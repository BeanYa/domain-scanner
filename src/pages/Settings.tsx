import { useState } from "react";
import {
  Brain,
  Cpu,
  Globe,
  Settings as SettingsIcon,
  Save,
  TestTube2,
  Monitor,
  Database,
  Key,
  Plus,
  ExternalLink,
} from "lucide-react";

type SettingsTab = "llm" | "gpu" | "tld" | "general";

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
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold neon-text tracking-tight">设置</h1>
        <p className="text-sm text-cyber-muted mt-1">配置 LLM、GPU、TLD 后缀和应用参数</p>
      </div>

      <div className="grid grid-cols-[200px_1fr] gap-6">
        {/* Sidebar Tabs */}
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

        {/* Content */}
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
  const configs = [
    { id: "glm-4", name: "GLM-4 (智谱)", baseUrl: "https://open.bigmodel.cn/api/paas/v4/", model: "glm-4-flash", isDefault: true, status: "connected" as const },
    { id: "minimax", name: "MiniMax Hailuo", baseUrl: "https://api.minimax.chat/v1/", model: "abab6.5s-chat", isDefault: false, status: "disconnected" as const },
  ];

  return (
    <div className="space-y-5 animate-fade-in">
      <div className="flex items-center justify-between">
        <h2 className="section-title m-0"><Brain className="w-4.5 h-4.5 text-cyber-purple" /> LLM 服务配置</h2>
        <button className="cyber-btn-primary cyber-btn-sm">
          <Plus className="w-3.5 h-3.5" /> 添加配置
        </button>
      </div>

      <div className="space-y-3">
        {configs.map((config) => (
          <div key={config.id} className="glass-panel p-5 space-y-3 group">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className={`w-9 h-9 rounded-xl flex items-center justify-center ${
                  config.status === "connected"
                    ? "bg-cyber-purple/10 text-cyber-purple"
                    : "bg-cyber-surface text-cyber-muted-dim"
                }`}>
                  <Brain className="w-4.5 h-4.5" />
                </div>
                <div>
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-semibold text-cyber-text">{config.name}</span>
                    {config.isDefault && <span className="badge-green text-[10px]">默认</span>}
                    <span className={`text-[10px] px-1.5 py-0.5 rounded-md ${
                      config.status === "connected" ? "bg-cyber-green/8 text-cyber-green" : "bg-cyber-red/8 text-cyber-red/70"
                    }`}>
                      {config.status === "connected" ? "已连接" : "未连接"}
                    </span>
                  </div>
                </div>
              </div>
              <div className="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                <button className="cyber-btn-secondary cyber-btn-sm"><TestTube2 className="w-3 h-3" /> 测试</button>
                <button className="cyber-btn-secondary cyber-btn-sm">编辑</button>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-3 text-xs pt-1">
              <div className="rounded-lg bg-cyber-bg-elevated/60 p-2.5 border border-cyber-border/15">
                <span className="text-cyber-muted-dim block mb-0.5">Base URL</span>
                <span className="text-cyber-text-secondary font-mono text-[11px] break-all">{config.baseUrl}</span>
              </div>
              <div className="rounded-lg bg-cyber-bg-elevated/60 p-2.5 border border-cyber-border/15">
                <span className="text-cyber-muted-dim block mb-0.5">Model</span>
                <span className="font-mono text-cyber-text-secondary">{config.model}</span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function GPUSettings() {
  return (
    <div className="space-y-5 animate-fade-in">
      <h2 className="section-title m-0"><Monitor className="w-4.5 h-4.5 text-cyber-green" /> GPU 加速配置</h2>

      <div className="glass-panel p-5 space-y-5">
        {/* Backend selector */}
        <div>
          <label className="block text-xs font-medium text-cyber-muted mb-1.5 uppercase tracking-wider">推理后端</label>
          <select className="cyber-input text-sm cursor-pointer">
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
            <input type="number" defaultValue={0} className="cyber-input w-full text-sm" />
          </div>
          <div>
            <label className="block text-xs font-medium text-cyber-muted mb-1.5 uppercase tracking-wider">批处理大小</label>
            <input type="number" defaultValue={500} className="cyber-input w-full text-sm" />
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
        <div className="flex items-start gap-3 p-4 rounded-xl bg-cyber-orange/[0.03] border border-cyber-orange/18">
          <Cpu className="w-5 h-5 text-cyber-orange shrink-0 mt-0.5" />
          <div>
            <p className="text-sm font-semibold text-cyber-orange">当前：CPU 模式运行中</p>
            <p className="text-xs text-cyber-muted-dim mt-1 leading-relaxed">
              未检测到可用 GPU。如需 GPU 加速，请重新构建时添加 <code className="font-mono text-cyber-orange bg-cyber-orange/8 px-1.5 py-0.5 rounded">--features gpu-directml</code> 参数。
              你的 AMD 5700XT 可通过 DirectML 获得显著的向量化性能提升。
            </p>
          </div>
        </div>

        <div className="pt-2">
          <button className="cyber-btn-primary cyber-btn-sm"><Save className="w-3.5 h-3.5" /> 保存 GPU 配置</button>
        </div>
      </div>
    </div>
  );
}

function TLDSettings() {
  const tlds = [
    { tld: ".com", popular: true }, { tld: ".net", popular: true },
    { tld: ".org", popular: true }, { tld: ".io", popular: true },
    { tld: ".ai", popular: false }, { tld: ".dev", popular: true },
    { tld: ".co", popular: false }, { tld: ".app", popular: false },
    { tld: ".info", popular: false }, { tld: ".biz", popular: false },
    { tld: ".xyz", popular: true }, { tld: ".me", popular: false },
  ];

  return (
    <div className="space-y-5 animate-fade-in">
      <div className="flex items-center justify-between">
        <h2 className="section-title m-0"><Database className="w-4.5 h-4.5 text-cyber-orange" /> TLD 后缀管理</h2>
        <button className="cyber-btn-secondary cyber-btn-sm"><Globe className="w-3.5 h-3.5" /> 在线更新列表</button>
      </div>

      <div className="glass-panel p-5 space-y-4">
        <p className="text-sm text-cyber-muted">
          当前内置 <strong className="text-cyber-text">{tlds.length}</strong> 个 TLD 后缀。点击标签可编辑或禁用。
        </p>

        <div className="flex flex-wrap gap-2">
          {tlds.map(({ tld, popular }) => (
            <span
              key={tld}
              className="group relative inline-flex items-center gap-1.5 px-3 py-2 rounded-xl 
                       bg-cyber-surface border border-cyber-border/30 text-sm text-cyber-text
                       hover:border-cyber-green/25 hover:bg-cyber-green/[0.04]
                       transition-all duration-200 cursor-pointer"
            >
              <Globe className="w-3.5 h-3.5 text-cyber-muted-dim group-hover:text-cyber-green transition-colors" />
              {tld}
              {popular && <span className="text-[9px] px-1 py-px rounded bg-cyber-orange/10 text-cyber-orange">HOT</span>}
            </span>
          ))}
        </div>

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
