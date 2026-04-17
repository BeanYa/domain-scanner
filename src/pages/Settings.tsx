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
} from "lucide-react";
import { useGpuStore } from "../store/gpuStore";
import { useLlmStore } from "../store/llmStore";
import { TLD_LIST, TLDS_BY_CATEGORY, TLD_COUNT, type TldCategory } from "../data/tlds";
import ActionNotice, { type ActionNoticeState } from "../components/ActionNotice";
import type { LlmConfig } from "../types";

type SettingsTab = "llm" | "gpu" | "tld" | "general";
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
