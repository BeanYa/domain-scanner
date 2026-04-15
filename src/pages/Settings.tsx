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
  Plus,
  ExternalLink,
} from "lucide-react";
import { useGpuStore } from "../store/gpuStore";
import { TLD_LIST, TLDS_BY_CATEGORY, TLD_COUNT, type TldCategory } from "../data/tlds";

type SettingsTab = "llm" | "gpu" | "tld" | "general";

const categoryLabels: Record<TldCategory, string> = {
  gtld: "通用 gTLD",
  new_gtld: "新 gTLD",
  cctld: "国家 ccTLD",
  other: "其他",
};

export default function Settings() {
  const [activeTab, setActiveTab] = useState<SettingsTab>("llm");

  const tabs: { key: SettingsTab; label: string; icon: typeof SettingsIcon; color: string }[] = [
    { key: "llm", label: "LLM 配置", icon: Brain, color: "text-cyber-purple" },
    { key: "gpu", label: "GPU 设置", icon: Cpu, color: "text-cyber-green" },
    { key: "tld", label: "TLD 管理", icon: Globe, color: "text-cyber-orange" },
    { key: "general", label: "通用设置", icon: SettingsIcon, color: "text-cyber-blue" },
  ];

  return (
    <div className="space-y-6 max-w-5xl animate-fade-in">
      <div>
        <h1 className="text-2xl font-bold neon-text tracking-tight">设置</h1>
        <p className="text-sm text-cyber-muted mt-1">配置 LLM、GPU、TLD 后缀和应用参数</p>
      </div>

      <div className="grid grid-cols-[200px_1fr] gap-6">
        <div className="space-y-1">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`
                w-full flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-sm font-medium transition-all duration-200
                ${activeTab === tab.key
                  ? "bg-gradient-to-r from-cyber-green/10 to-transparent text-cyber-green border border-cyber-green/20 shadow-neon"
                  : "text-cyber-muted hover:text-cyber-text-secondary hover:bg-cyber-card/50 border border-transparent"
                }
              `}
            >
              <tab.icon className={`w-4 h-4 ${activeTab === tab.key ? "" : tab.color + "/60"}`} />
              <span>{tab.label}</span>
            </button>
          ))}
        </div>

        <div>
          {activeTab === "llm" && <LLMSettings />}
          {activeTab === "gpu" && <GPUSettings />}
          {activeTab === "tld" && <TLDSettings />}
          {activeTab === "general" && <GeneralSettings />}
        </div>
      </div>
    </div>
  );
}

function LLMSettings() {
  return (
    <div className="space-y-5 animate-fade-in">
      <div className="flex items-center justify-between">
        <h2 className="section-title m-0"><Brain className="w-4.5 h-4.5 text-cyber-purple" /> LLM 服务配置</h2>
        <button className="cyber-btn-primary cyber-btn-sm">
          <Plus className="w-3.5 h-3.5" /> 添加配置
        </button>
      </div>

      <div className="glass-panel p-8 text-center text-cyber-muted">
        <Brain className="w-10 h-10 mx-auto mb-3 opacity-30" />
        <p className="text-sm">暂无 LLM 配置</p>
        <p className="text-xs text-cyber-muted-dim mt-1">点击「添加配置」连接 LLM 服务用于智能域名生成</p>
      </div>
    </div>
  );
}

function GPUSettings() {
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

  const handleSave = () => {
    updateConfig({ backend: backend as any, device_id: deviceId, batch_size: batchSize });
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
        <div className={`flex items-start gap-3 p-4 rounded-xl border ${
          gpuStatus?.available && gpuStatus.backend !== "cpu"
            ? "bg-cyber-green/[0.03] border-cyber-green/18"
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

function TLDSettings() {
  return (
    <div className="space-y-5 animate-fade-in">
      <div className="flex items-center justify-between">
        <h2 className="section-title m-0"><Database className="w-4.5 h-4.5 text-cyber-orange" /> TLD 后缀管理</h2>
        <button className="cyber-btn-secondary cyber-btn-sm"><Globe className="w-3.5 h-3.5" /> 在线更新列表</button>
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

function GeneralSettings() {
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

        <button className="cyber-btn-primary cyber-btn-sm"><Save className="w-3.5 h-3.5" /> 保存所有设置</button>
      </div>
    </div>
  );
}
