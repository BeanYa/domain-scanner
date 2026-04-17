import { useState, useEffect } from "react";
import {
  Brain,
  Cpu,
  Globe,
  Settings as SettingsIcon,
  Save,
  TestTube2,
  Monitor,
  Database,
  ExternalLink,
  Loader2,
  Server,
  Copy,
  RefreshCw,
  Trash2,
  Power,
  PowerOff,
  KeyRound,
  Clock,
  Activity,
} from "lucide-react";
import { useGpuStore } from "../store/gpuStore";
import { useLlmStore } from "../store/llmStore";
import { useClusterStore } from "../store/clusterStore";
import { TLD_LIST, TLDS_BY_CATEGORY, TLD_COUNT, type TldCategory } from "../data/tlds";
import ActionNotice, { type ActionNoticeState } from "../components/ActionNotice";
import type { ClusterWorker, ClusterWorkerStatus, LlmConfig } from "../types";

type SettingsTab = "llm" | "gpu" | "cluster" | "tld" | "general";
type SettingsNotify = (action: string, notice: ActionNoticeState) => void;

const categoryLabels: Record<TldCategory, string> = {
  gtld: "通用 gTLD",
  new_gtld: "新 gTLD",
  cctld: "国家 ccTLD",
  other: "其他",
};

export default function Settings() {
  const [activeTab, setActiveTab] = useState<SettingsTab>("llm");
  const [notice, setNotice] = useState<ActionNoticeState | null>(null);

  const notify: SettingsNotify = (action, nextNotice) => {
    console.info(`[ui-action] settings-${action}`, nextNotice);
    setNotice(nextNotice);
  };

  const tabs: { key: SettingsTab; label: string; icon: typeof SettingsIcon; color: string }[] = [
    { key: "llm", label: "Embedding", icon: Brain, color: "text-cyber-purple" },
    { key: "gpu", label: "GPU 设置", icon: Cpu, color: "text-cyber-green" },
    { key: "cluster", label: "集群节点", icon: Server, color: "text-cyber-blue" },
    { key: "tld", label: "TLD 管理", icon: Globe, color: "text-cyber-orange" },
    { key: "general", label: "通用设置", icon: SettingsIcon, color: "text-cyber-blue" },
  ];

  return (
    <div className="page-shell max-w-5xl">
      <div>
        <div className="eyebrow mb-3">APPLICATION SETTINGS</div>
        <h1 className="page-heading">设置</h1>
        <p className="page-subtitle">配置 Embedding、GPU、TLD 后缀和应用参数。</p>
      </div>

      {notice && <ActionNotice notice={notice} onClose={() => setNotice(null)} />}

      <div className="grid grid-cols-[200px_1fr] gap-6">
        <div className="space-y-1">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`
                w-full flex items-center gap-3 px-3.5 py-2.5 rounded-md text-sm font-medium transition-colors duration-150
                ${activeTab === tab.key
                  ? "bg-white/[0.06] text-white border border-white/15"
                  : "text-cyber-muted hover:text-cyber-text-secondary hover:bg-cyber-card border border-transparent"
                }
              `}
            >
              <tab.icon className={`w-4 h-4 ${activeTab === tab.key ? "" : tab.color + "/60"}`} />
              <span>{tab.label}</span>
            </button>
          ))}
        </div>

        <div>
          {activeTab === "llm" && <LLMSettings notify={notify} />}
          {activeTab === "gpu" && <GPUSettings notify={notify} />}
          {activeTab === "cluster" && <ClusterSettings notify={notify} />}
          {activeTab === "tld" && <TLDSettings notify={notify} />}
          {activeTab === "general" && <GeneralSettings notify={notify} />}
        </div>
      </div>
    </div>
  );
}

function LLMSettings({ notify }: { notify: SettingsNotify }) {
  const { configs, loading, testing, error, fetchConfigs, saveConfig, testConfig } = useLlmStore();
  const [selectedTemplateId, setSelectedTemplateId] = useState("openrouter-free-embedding");
  const [form, setForm] = useState({
    id: "",
    name: "OpenRouter Free Embedding",
    base_url: "https://openrouter.ai/api/v1/",
    api_key: "",
    model: "",
    embedding_model: "nvidia/llama-nemotron-embed-vl-1b-v2:free",
    embedding_dim: 384,
    is_default: true,
  });

  useEffect(() => {
    fetchConfigs();
  }, [fetchConfigs]);

  const templates = configs.filter((config) => config.is_template);
  const savedConfigs = configs.filter((config) => !config.is_template);
  const defaultConfig = savedConfigs.find((config) => config.is_default);
  const selectedTemplate = templates.find((config) => config.id === selectedTemplateId) ?? templates[0];

  const applyTemplate = (template: LlmConfig) => {
    setSelectedTemplateId(template.id);
    setForm((current) => ({
      ...current,
      id: "",
      name: template.name,
      base_url: template.base_url,
      model: template.model ?? "",
      embedding_model: template.embedding_model ?? "",
      embedding_dim: template.embedding_dim || 384,
    }));
    notify("llm-template-apply", {
      tone: "success",
      title: `已套用 ${template.name}`,
      message: "该预设已填入 embedding 模型，可用于当前 384 维向量化流程。",
    });
  };

  const handleTemplateChange = (templateId: string) => {
    const template = templates.find((item) => item.id === templateId);
    if (template) {
      applyTemplate(template);
    } else {
      setSelectedTemplateId(templateId);
    }
  };

  const handleSave = async () => {
    if (!form.name.trim() || !form.base_url.trim() || !form.api_key.trim() || !form.embedding_model.trim()) {
      notify("llm-save-validation", {
        tone: "warning",
        title: "配置不完整",
        message: "请填写名称、Base URL、API Key 和 Embedding Model。",
      });
      return;
    }
    try {
      await saveConfig({
        id: form.id || undefined,
        name: form.name.trim(),
        base_url: form.base_url.trim(),
        api_key: form.api_key.trim(),
        model: form.model.trim() || form.embedding_model.trim(),
        embedding_model: form.embedding_model.trim() || null,
        embedding_dim: form.embedding_dim,
        is_default: form.is_default,
      });
      await fetchConfigs();
      notify("llm-save", {
        tone: "success",
        title: "Embedding API 配置已保存",
        message: "后续向量化和语义筛选会使用默认配置的 embeddings 接口。请确保 embedding 维度为 384。",
      });
    } catch (e) {
      notify("llm-save-error", {
        tone: "error",
        title: "保存 Embedding 配置失败",
        message: String(e),
      });
    }
  };

  const handleTestDefault = async () => {
    const target = defaultConfig ?? savedConfigs[0];
    if (!target) {
      notify("llm-test-missing", {
        tone: "warning",
        title: "没有可测试的配置",
        message: "请先保存一个 Embedding API 配置。",
      });
      return;
    }
    notify("llm-test-start", {
      tone: "running",
      title: "正在测试 Embedding 连接",
      message: `正在调用 ${target.name} 的 embeddings 接口。`,
    });
    const ok = await testConfig(target.id);
    notify("llm-test-finish", {
      tone: ok ? "success" : "error",
      title: ok ? "Embedding 连接测试通过" : "Embedding 连接测试失败",
      message: ok ? "默认配置可以访问 OpenAI 兼容 embeddings 接口，且返回维度匹配。" : useLlmStore.getState().error ?? "接口没有返回成功结果。",
    });
  };

  return (
    <div className="space-y-5 animate-fade-in">
      <div className="flex items-center justify-between">
        <h2 className="section-title m-0"><Brain className="w-4.5 h-4.5 text-cyber-purple" /> Embedding API 配置</h2>
        <button
          className="cyber-btn-secondary cyber-btn-sm"
          onClick={handleTestDefault}
          disabled={testing || savedConfigs.length === 0}
        >
          {testing ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <TestTube2 className="w-3.5 h-3.5" />}
          测试 Embedding
        </button>
      </div>

      <div className="glass-panel p-5 space-y-4">
        <div className="rounded-md border border-cyber-border bg-cyber-surface p-3.5 space-y-3">
          <div className="flex items-start justify-between gap-3">
            <label className="space-y-1.5 flex-1 min-w-0">
              <span className="block text-xs font-medium text-cyber-muted uppercase tracking-wider">Embedding 预设</span>
              <select
                className="cyber-input w-full text-sm cursor-pointer"
                value={selectedTemplate?.id ?? selectedTemplateId}
                onChange={(e) => handleTemplateChange(e.target.value)}
              >
                {templates.length === 0 && <option value="openai-compatible">正在加载预设...</option>}
                {templates.map((template) => (
                  <option key={template.id} value={template.id}>
                    {template.name} · {template.region ?? "Custom"} · {template.category ?? "API"}
                  </option>
                ))}
              </select>
            </label>
            <button
              className="cyber-btn-secondary cyber-btn-sm mt-6"
              onClick={() => selectedTemplate && applyTemplate(selectedTemplate)}
              disabled={!selectedTemplate}
            >
              <Brain className="w-3.5 h-3.5" />
              套用
            </button>
          </div>

          {selectedTemplate && (
            <div className="flex flex-wrap items-center gap-2 text-xs">
              <span className="px-2 py-1 rounded border border-cyber-border bg-cyber-bg-elevated text-cyber-text-secondary">
                {selectedTemplate.base_url}
              </span>
              <span className="px-2 py-1 rounded border border-cyber-border bg-cyber-bg-elevated text-cyber-muted">
                Embedding: {selectedTemplate.embedding_model ?? "-"}
              </span>
              <span className="px-2 py-1 rounded border border-cyber-green/25 bg-cyber-green/[0.05] text-cyber-green">
                可用于向量化
              </span>
            </div>
          )}

          {selectedTemplate?.notes && (
            <p className="text-xs text-cyber-muted leading-5">{selectedTemplate.notes}</p>
          )}
        </div>

        <div className="grid grid-cols-2 gap-4">
          <label className="space-y-1.5">
            <span className="block text-xs font-medium text-cyber-muted uppercase tracking-wider">配置名称</span>
            <input className="cyber-input w-full text-sm" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
          </label>
          <label className="space-y-1.5">
            <span className="block text-xs font-medium text-cyber-muted uppercase tracking-wider">Chat Model（可选）</span>
            <input className="cyber-input w-full text-sm font-mono" placeholder="仅 LLM 生成候选域名时需要" value={form.model} onChange={(e) => setForm({ ...form, model: e.target.value })} />
          </label>
          <label className="col-span-2 space-y-1.5">
            <span className="block text-xs font-medium text-cyber-muted uppercase tracking-wider">Base URL</span>
            <input className="cyber-input w-full text-sm font-mono" placeholder="https://api.openai.com/v1/" value={form.base_url} onChange={(e) => setForm({ ...form, base_url: e.target.value })} />
          </label>
          <label className="col-span-2 space-y-1.5">
            <span className="block text-xs font-medium text-cyber-muted uppercase tracking-wider">API Key</span>
            <input className="cyber-input w-full text-sm font-mono" type="password" value={form.api_key} onChange={(e) => setForm({ ...form, api_key: e.target.value })} />
          </label>
          <label className="space-y-1.5">
            <span className="block text-xs font-medium text-cyber-muted uppercase tracking-wider">Embedding Model</span>
            <input className="cyber-input w-full text-sm font-mono" value={form.embedding_model} onChange={(e) => setForm({ ...form, embedding_model: e.target.value })} />
          </label>
          <label className="space-y-1.5">
            <span className="block text-xs font-medium text-cyber-muted uppercase tracking-wider">Embedding 维度</span>
            <input className="cyber-input w-full text-sm" type="number" min={1} value={form.embedding_dim} onChange={(e) => setForm({ ...form, embedding_dim: Number(e.target.value) })} />
          </label>
        </div>

        <label className="inline-flex items-center gap-2 text-xs text-cyber-muted">
          <input
            type="checkbox"
            checked={form.is_default}
            onChange={(e) => setForm({ ...form, is_default: e.target.checked })}
            className="accent-cyber-green"
          />
          设为默认配置，用于向量化和语义筛选
        </label>

        <div className="rounded-md border border-cyber-border bg-cyber-surface p-3 text-xs text-cyber-muted leading-5">
          当前核心流程只需要 embeddings：向量化会把可用域名写入 384 维向量表，语义筛选会对查询文本取 embedding 后做向量召回。Chat Model 仅在使用 LLM 智能生成候选域名前缀时才需要。
        </div>

        {error && <div className="text-sm text-cyber-red">{error}</div>}

        <div className="flex items-center justify-between gap-3">
          <div className="text-xs text-cyber-muted">
            {defaultConfig ? `默认配置：${defaultConfig.name} (${defaultConfig.embedding_model ?? "未配置 embedding"})` : "尚未保存默认配置"}
          </div>
          <button className="cyber-btn-primary cyber-btn-sm" onClick={handleSave} disabled={loading}>
            {loading ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Save className="w-3.5 h-3.5" />}
            保存 Embedding 配置
          </button>
        </div>
      </div>
    </div>
  );
}

function GPUSettings({ notify }: { notify: SettingsNotify }) {
  const { status: gpuStatus, config: gpuConfig, fetchStatus, updateConfig } = useGpuStore();
  const [backend, setBackend] = useState(gpuConfig?.backend || "auto");
  const [deviceId, setDeviceId] = useState(gpuConfig?.device_id ?? 0);
  const [batchSize, setBatchSize] = useState(gpuConfig?.batch_size ?? 500);

  useEffect(() => {
    fetchStatus();
  }, []);

  useEffect(() => {
    if (gpuConfig) {
      setBackend(gpuConfig.backend);
      setDeviceId(gpuConfig.device_id);
      setBatchSize(gpuConfig.batch_size);
    }
  }, [gpuConfig]);

  const handleSave = async () => {
    await updateConfig({ backend: backend as any, device_id: deviceId, batch_size: batchSize });
    notify("gpu-save", {
      tone: "success",
      title: "GPU 配置已提交",
      message: `已提交后端 ${backend}、设备 ${deviceId}、批处理 ${batchSize}。如后端返回错误，会显示在 GPU 状态区。`,
    });
  };

  const gpuBackendLabel = gpuStatus?.available
    ? gpuStatus.backend === "cuda" ? "CUDA"
    : gpuStatus.backend === "directml" ? "DirectML"
    : gpuStatus.backend === "cpu" ? "CPU"
    : gpuStatus.backend
    : "检测中...";
  const gpuDeviceName = gpuStatus?.device_name || "-";

  return (
    <div className="space-y-5 animate-fade-in">
      <h2 className="section-title m-0"><Monitor className="w-4.5 h-4.5 text-cyber-green" /> GPU 加速配置</h2>

      <div className="glass-panel p-5 space-y-5">
        <div>
          <label className="block text-xs font-medium text-cyber-muted mb-1.5 uppercase tracking-wider">推理后端</label>
          <select
            value={backend}
            onChange={(e) => setBackend(e.target.value as any)}
            className="cyber-input text-sm cursor-pointer"
          >
            <option value="auto">自动检测（推荐）</option>
            <option value="cuda">CUDA (NVIDIA)</option>
            <option value="directml">DirectML (AMD Windows)</option>
            <option value="rocm">ROCm (AMD Linux)</option>
            <option value="coreml">CoreML (Apple Silicon)</option>
            <option value="cpu">CPU 仅限</option>
            <option value="remote">远程 Embedding API</option>
          </select>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-xs font-medium text-cyber-muted mb-1.5 uppercase tracking-wider">设备 ID</label>
            <input type="number" value={deviceId} onChange={(e) => setDeviceId(Number(e.target.value))} className="cyber-input w-full text-sm" />
          </div>
          <div>
            <label className="block text-xs font-medium text-cyber-muted mb-1.5 uppercase tracking-wider">批处理大小</label>
            <input type="number" value={batchSize} onChange={(e) => setBatchSize(Number(e.target.value))} className="cyber-input w-full text-sm" />
          </div>
        </div>

        <div>
          <label className="block text-xs font-medium text-cyber-muted mb-1.5 uppercase tracking-wider">本地模型路径</label>
          <input type="text" placeholder="留空使用内置 MiniLM-L6-v2 模型" className="cyber-input w-full text-sm font-mono" />
          <p className="text-[11px] text-cyber-muted-dim mt-1.5 flex items-center gap-1.5">
            <Cpu className="w-3 h-3" /> 支持 ONNX 格式的 all-MiniLM-L6-v2 模型（384 维向量）
          </p>
        </div>

        {/* GPU Status Banner */}
        <div className={`flex items-start gap-3 p-4 rounded-md border ${
          gpuStatus?.available && gpuStatus.backend !== "cpu"
            ? "bg-white/[0.04] border-white/12"
            : "bg-cyber-orange/[0.03] border-cyber-orange/18"
        }`}>
          <Cpu className={`w-5 h-5 shrink-0 mt-0.5 ${gpuStatus?.available && gpuStatus.backend !== "cpu" ? "text-cyber-green" : "text-cyber-orange"}`} />
          <div>
            <p className={`text-sm font-semibold ${gpuStatus?.available && gpuStatus.backend !== "cpu" ? "text-cyber-green" : "text-cyber-orange"}`}>
              {gpuStatus?.available && gpuStatus.backend !== "cpu"
                ? `GPU 已启用：${gpuBackendLabel}`
                : "当前：CPU 模式运行中"}
            </p>
            <p className="text-xs text-cyber-muted-dim mt-1 leading-relaxed">
              {gpuStatus?.device_name
                ? `检测到设备：${gpuDeviceName}`
                : "未检测到可用 GPU。如需 GPU 加速，请安装对应版本的 CUDA/DirectML 驱动。"}
            </p>
          </div>
        </div>

        <div className="pt-2">
          <button onClick={handleSave} className="cyber-btn-primary cyber-btn-sm"><Save className="w-3.5 h-3.5" /> 保存 GPU 配置</button>
        </div>
      </div>
    </div>
  );
}

const workerStatusLabels: Record<ClusterWorkerStatus, { label: string; className: string }> = {
  pending: { label: "等待注册", className: "badge-blue" },
  available: { label: "在线", className: "badge-green" },
  unavailable: { label: "离线", className: "badge-neutral" },
  error: { label: "错误", className: "badge-red" },
  expired: { label: "已过期", className: "badge-red" },
  disabled: { label: "已禁用", className: "badge-neutral" },
};

function ClusterSettings({ notify }: { notify: SettingsNotify }) {
  const {
    workers,
    loading,
    registering,
    testingWorkerId,
    error,
    lastRegistration,
    fetchWorkers,
    createWorkerRegistration,
    pollWorkerRegistration,
    testWorker,
    enableWorker,
    disableWorker,
    deleteWorker,
  } = useClusterStore();
  const [form, setForm] = useState({
    name: "",
    base_url: "http://127.0.0.1:8731",
    script_url: "https://example.com/domain-scanner/worker_install.sh",
    port: 8731,
    timeout_seconds: 600,
  });

  useEffect(() => {
    fetchWorkers();
  }, [fetchWorkers]);

  const remoteWorkers = workers.filter((worker) => worker.worker_type === "remote");
  const availableCount = workers.filter((worker) => worker.status === "available").length;

  const handleCreateRegistration = async () => {
    const registration = await createWorkerRegistration({
      name: form.name.trim() || undefined,
      base_url: form.base_url.trim(),
      script_url: form.script_url.trim(),
      port: form.port,
      timeout_seconds: form.timeout_seconds,
    });
    if (!registration) {
      notify("cluster-register-error", {
        tone: "error",
        title: "创建 worker 注册失败",
        message: useClusterStore.getState().error ?? "后端没有返回注册信息。",
      });
      return;
    }
    notify("cluster-register", {
      tone: "success",
      title: "安装命令已生成",
      message: "该命令包含一次性 token，请只在受信任的服务器终端中使用。",
    });
  };

  const handleCopyCommand = async (command: string) => {
    try {
      await navigator.clipboard.writeText(command);
      notify("cluster-copy-command", {
        tone: "success",
        title: "安装命令已复制",
        message: "命令包含敏感 token，避免粘贴到日志、聊天或公开文档中。",
      });
    } catch (e) {
      notify("cluster-copy-command-error", {
        tone: "error",
        title: "复制失败",
        message: String(e),
      });
    }
  };

  const handleProbe = async (worker: ClusterWorker) => {
    const result =
      worker.status === "pending"
        ? await pollWorkerRegistration(worker.id)
        : await testWorker(worker.id);
    notify("cluster-probe", {
      tone: result?.success ? "success" : "warning",
      title: result?.success ? "worker 探测通过" : "worker 探测未通过",
      message: result?.message ?? useClusterStore.getState().error ?? "没有返回探测结果。",
    });
  };

  const handleToggleWorker = async (worker: ClusterWorker) => {
    if (worker.enabled && worker.status !== "disabled") {
      await disableWorker(worker.id);
      notify("cluster-disable", {
        tone: "info",
        title: "worker 已禁用",
        message: `${worker.name ?? worker.id} 不会参与后续 batch 调度。`,
      });
    } else {
      await enableWorker(worker.id);
      notify("cluster-enable", {
        tone: "info",
        title: "worker 已启用",
        message: `${worker.name ?? worker.id} 已恢复为可探测状态。`,
      });
    }
  };

  const handleDeleteWorker = async (worker: ClusterWorker) => {
    const confirmed = window.confirm(`确定删除 worker“${worker.name ?? worker.id}”吗？`);
    if (!confirmed) return;
    await deleteWorker(worker.id);
    const storeError = useClusterStore.getState().error;
    notify("cluster-delete", {
      tone: storeError ? "error" : "info",
      title: storeError ? "删除 worker 失败" : "worker 已删除",
      message: storeError ?? "该远端 worker 记录已从本地移除。",
    });
  };

  return (
    <div className="space-y-5 animate-fade-in">
      <div className="flex items-center justify-between">
        <h2 className="section-title m-0">
          <Server className="w-4.5 h-4.5 text-cyber-blue" /> 集群节点
        </h2>
        <button className="cyber-btn-secondary cyber-btn-sm" onClick={fetchWorkers} disabled={loading}>
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? "animate-spin" : ""}`} />
          刷新
        </button>
      </div>

      <div className="grid grid-cols-3 gap-3">
        <div className="metric-tile space-y-1">
          <p className="text-xs text-cyber-muted">节点总数</p>
          <p className="text-xl font-normal text-cyber-text tabular-nums">{workers.length}</p>
          <p className="text-[10px] text-cyber-muted-dim">{remoteWorkers.length} 个远端</p>
        </div>
        <div className="metric-tile space-y-1">
          <p className="text-xs text-cyber-muted">在线节点</p>
          <p className="text-xl font-normal text-cyber-green tabular-nums">{availableCount}</p>
          <p className="text-[10px] text-cyber-muted-dim">包含本地 worker</p>
        </div>
        <div className="metric-tile space-y-1">
          <p className="text-xs text-cyber-muted">待注册</p>
          <p className="text-xl font-normal text-cyber-orange tabular-nums">
            {workers.filter((worker) => worker.status === "pending").length}
          </p>
          <p className="text-[10px] text-cyber-muted-dim">可轮询 /health</p>
        </div>
      </div>

      <div className="glass-panel p-5 space-y-4">
        <div className="flex items-center justify-between gap-3">
          <h3 className="text-sm font-semibold text-cyber-text flex items-center gap-2">
            <KeyRound className="w-4 h-4 text-cyber-green" />
            添加远端 worker
          </h3>
          <button className="cyber-btn-primary cyber-btn-sm" onClick={handleCreateRegistration} disabled={registering}>
            {registering ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Server className="w-3.5 h-3.5" />}
            生成安装命令
          </button>
        </div>
        <div className="grid grid-cols-2 gap-4">
          <label className="space-y-1.5">
            <span className="block text-xs font-medium text-cyber-muted uppercase tracking-wider">节点名称</span>
            <input
              className="cyber-input w-full text-sm"
              placeholder="worker-01"
              value={form.name}
              onChange={(event) => setForm({ ...form, name: event.target.value })}
            />
          </label>
          <label className="space-y-1.5">
            <span className="block text-xs font-medium text-cyber-muted uppercase tracking-wider">Worker URL</span>
            <input
              className="cyber-input w-full text-sm font-mono"
              value={form.base_url}
              onChange={(event) => setForm({ ...form, base_url: event.target.value })}
            />
          </label>
          <label className="space-y-1.5">
            <span className="block text-xs font-medium text-cyber-muted uppercase tracking-wider">安装脚本 URL</span>
            <input
              className="cyber-input w-full text-sm font-mono"
              value={form.script_url}
              onChange={(event) => setForm({ ...form, script_url: event.target.value })}
            />
          </label>
          <div className="grid grid-cols-2 gap-3">
            <label className="space-y-1.5">
              <span className="block text-xs font-medium text-cyber-muted uppercase tracking-wider">端口</span>
              <input
                className="cyber-input w-full text-sm"
                type="number"
                min={1}
                value={form.port}
                onChange={(event) => setForm({ ...form, port: Number(event.target.value) || 8731 })}
              />
            </label>
            <label className="space-y-1.5">
              <span className="block text-xs font-medium text-cyber-muted uppercase tracking-wider">有效期秒</span>
              <input
                className="cyber-input w-full text-sm"
                type="number"
                min={30}
                value={form.timeout_seconds}
                onChange={(event) => setForm({ ...form, timeout_seconds: Number(event.target.value) || 600 })}
              />
            </label>
          </div>
        </div>

        {lastRegistration && (
          <div className="rounded-md border border-cyber-orange/25 bg-cyber-orange/[0.04] p-3 space-y-2">
            <div className="flex items-center justify-between gap-3">
              <div className="min-w-0">
                <div className="text-xs font-semibold text-cyber-orange">敏感安装命令</div>
                <div className="mt-1 text-[11px] text-cyber-muted-dim">
                  过期时间：{formatDateTime(lastRegistration.expires_at)}
                </div>
              </div>
              <button
                className="cyber-btn-secondary cyber-btn-sm shrink-0"
                onClick={() => handleCopyCommand(lastRegistration.install_command)}
              >
                <Copy className="w-3.5 h-3.5" />
                复制
              </button>
            </div>
            <code className="block rounded bg-cyber-bg-elevated/70 border border-cyber-border/30 p-3 text-xs text-cyber-text-secondary break-all">
              {lastRegistration.install_command}
            </code>
          </div>
        )}

        {error && <div className="text-sm text-cyber-red">{error}</div>}
      </div>

      <div className="space-y-3">
        {workers.map((worker) => {
          const status = workerStatusLabels[worker.status] ?? workerStatusLabels.unavailable;
          const isBusy = testingWorkerId === worker.id;
          const capabilities = formatCapabilities(worker);
          return (
            <div key={worker.id} className="glass-panel p-4 space-y-3">
              <div className="flex items-start justify-between gap-4">
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <Server className={`w-4 h-4 ${worker.worker_type === "local" ? "text-cyber-green" : "text-cyber-blue"}`} />
                    <h3 className="text-sm font-semibold text-cyber-text truncate">
                      {worker.name ?? (worker.worker_type === "local" ? "本地内置 Worker" : worker.id)}
                    </h3>
                    <span className={`${status.className} text-[10px]`}>{status.label}</span>
                  </div>
                  <div className="mt-1 text-xs text-cyber-muted-dim font-mono break-all">
                    {worker.base_url ?? worker.id}
                  </div>
                </div>
                <div className="flex items-center gap-2 shrink-0">
                  <button className="cyber-btn-secondary cyber-btn-sm" onClick={() => handleProbe(worker)} disabled={isBusy}>
                    {isBusy ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Activity className="w-3.5 h-3.5" />}
                    探测
                  </button>
                  <button className="cyber-btn-secondary cyber-btn-sm" onClick={() => handleToggleWorker(worker)}>
                    {worker.enabled && worker.status !== "disabled" ? <PowerOff className="w-3.5 h-3.5" /> : <Power className="w-3.5 h-3.5" />}
                    {worker.enabled && worker.status !== "disabled" ? "禁用" : "启用"}
                  </button>
                  {worker.worker_type === "remote" && (
                    <button
                      className="cyber-btn-secondary cyber-btn-sm text-cyber-red border-cyber-red/25 hover:border-cyber-red/45 hover:text-cyber-red"
                      onClick={() => handleDeleteWorker(worker)}
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                      删除
                    </button>
                  )}
                </div>
              </div>
              <div className="grid grid-cols-3 gap-3 text-xs">
                <div className="rounded-md border border-cyber-border/25 bg-cyber-surface/50 p-3">
                  <div className="text-cyber-muted-dim">能力</div>
                  <div className="mt-1 font-mono text-cyber-text-secondary">{capabilities}</div>
                </div>
                <div className="rounded-md border border-cyber-border/25 bg-cyber-surface/50 p-3">
                  <div className="text-cyber-muted-dim">版本</div>
                  <div className="mt-1 font-mono text-cyber-text-secondary">{worker.version ?? "-"}</div>
                </div>
                <div className="rounded-md border border-cyber-border/25 bg-cyber-surface/50 p-3">
                  <div className="text-cyber-muted-dim flex items-center gap-1">
                    <Clock className="w-3 h-3" />
                    最近检查
                  </div>
                  <div className="mt-1 text-cyber-text-secondary">
                    {worker.last_checked_at ? formatDateTime(worker.last_checked_at) : "-"}
                  </div>
                </div>
              </div>
              {worker.last_error && (
                <div className="text-xs text-cyber-red break-all">{worker.last_error}</div>
              )}
            </div>
          );
        })}

        {!loading && workers.length === 0 && (
          <div className="glass-panel p-8 text-center text-sm text-cyber-muted">
            还没有集群节点。
          </div>
        )}
      </div>
    </div>
  );
}

function formatCapabilities(worker: ClusterWorker) {
  const running = worker.max_running_batches ?? "-";
  const total = worker.max_total_concurrency ?? "-";
  const batch = worker.max_batch_concurrency ?? "-";
  return `${running} batches / ${total} total / ${batch} per batch`;
}

function TLDSettings({ notify }: { notify: SettingsNotify }) {
  return (
    <div className="space-y-5 animate-fade-in">
      <div className="flex items-center justify-between">
        <h2 className="section-title m-0"><Database className="w-4.5 h-4.5 text-cyber-orange" /> TLD 后缀管理</h2>
        <button
          className="cyber-btn-secondary cyber-btn-sm"
          onClick={() =>
            notify("tld-online-update", {
              tone: "warning",
              title: "在线更新尚未接入",
              message: "当前 TLD 列表来自内置数据；在线拉取 IANA 注册表和持久化自定义扩展还未接入。",
            })
          }
        >
          <Globe className="w-3.5 h-3.5" /> 在线更新列表
        </button>
      </div>

      <div className="glass-panel p-5 space-y-4">
        <p className="text-sm text-cyber-muted">
          当前内置 <strong className="text-cyber-text">{TLD_COUNT}</strong> 个 TLD 后缀，涵盖通用、新顶级域名、国家代码等分类。
        </p>

        {(Object.entries(TLDS_BY_CATEGORY) as [TldCategory, typeof TLD_LIST][]).map(([cat, tlds]) => (
          <div key={cat}>
            <p className="text-xs font-semibold text-cyber-muted uppercase tracking-wider mb-2">
              {categoryLabels[cat]} ({tlds.length})
            </p>
            <div className="flex flex-wrap gap-1.5">
              {tlds.map(({ tld, popular }) => (
                <span
                  key={tld}
                  className="group relative inline-flex items-center gap-1 px-2.5 py-1.5 rounded-lg
                           bg-cyber-surface border border-cyber-border/30 text-xs text-cyber-text
                           hover:border-cyber-green/25 hover:bg-cyber-green/[0.04]
                           transition-all duration-200 cursor-pointer"
                >
                  {tld}
                  {popular && <span className="text-[8px] px-0.5 rounded bg-cyber-orange/10 text-cyber-orange">HOT</span>}
                </span>
              ))}
            </div>
          </div>
        ))}

        <div className="divider my-2" />

        <div className="flex items-center gap-2 text-xs text-cyber-muted-dim">
          <ExternalLink className="w-3.5 h-3.5" />
          <span>TLD 数据来源于 IANA 注册表，可自定义扩展</span>
        </div>
      </div>
    </div>
  );
}

function GeneralSettings({ notify }: { notify: SettingsNotify }) {
  return (
    <div className="space-y-5 animate-fade-in">
      <h2 className="section-title m-0">通用应用设置</h2>

      <div className="glass-panel p-5 space-y-5">
        <div>
          <label className="block text-xs font-medium text-cyber-muted mb-1.5 uppercase tracking-wider">最大并发查询数</label>
          <input type="number" defaultValue={50} className="cyber-input w-full text-sm" />
          <p className="text-[11px] text-cyber-muted-dim mt-1">建议 10~100，过高可能触发目标 RDAP/DNS 限流策略</p>
        </div>

        <div>
          <label className="block text-xs font-medium text-cyber-muted mb-1.5 uppercase tracking-wider">RDAP 查询超时 (秒)</label>
          <input type="number" defaultValue={10} min={1} max={60} className="cyber-input w-full text-sm" />
        </div>

        <div>
          <label className="block text-xs font-medium text-cyber-muted mb-1.5 uppercase tracking-wider">请求间隔范围 (ms)</label>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <input type="number" defaultValue={50} placeholder="最小延迟" className="cyber-input w-full text-sm" />
            </div>
            <div>
              <input type="number" defaultValue={200} placeholder="最大延迟" className="cyber-input w-full text-sm" />
            </div>
          </div>
        </div>

        <div>
          <label className="block text-xs font-medium text-cyber-muted mb-1.5 uppercase tracking-wider">失败重试次数</label>
          <input type="number" defaultValue={3} min={0} max={10} className="cyber-input w-full text-sm" />
        </div>

        <div>
          <label className="block text-xs font-medium text-cyber-muted mb-1.5 uppercase tracking-wider">数据存储目录</label>
          <input type="text" defaultValue="~/.domain-scanner" className="cyber-input w-full text-sm font-mono" />
        </div>

        <div className="divider my-1" />

        <button
          className="cyber-btn-primary cyber-btn-sm"
          onClick={() =>
            notify("general-save", {
              tone: "warning",
              title: "通用设置尚未持久化",
              message: "这些表单项目前是界面占位，尚未接入后端配置存储；点击已被记录，避免静默无响应。",
            })
          }
        >
          <Save className="w-3.5 h-3.5" /> 保存所有设置
        </button>
      </div>
    </div>
  );
}

function formatDateTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}
