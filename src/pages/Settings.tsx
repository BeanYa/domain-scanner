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
} from "lucide-react";

type SettingsTab = "llm" | "gpu" | "tld" | "general";

export default function Settings() {
  const [activeTab, setActiveTab] = useState<SettingsTab>("llm");

  const tabs: { key: SettingsTab; label: string; icon: typeof SettingsIcon }[] = [
    { key: "llm", label: "LLM 配置", icon: Brain },
    { key: "gpu", label: "GPU 设置", icon: Cpu },
    { key: "tld", label: "TLD 管理", icon: Globe },
    { key: "general", label: "通用设置", icon: SettingsIcon },
  ];

  return (
    <div className="p-6 space-y-6 max-w-5xl">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold neon-text">设置</h1>
        <p className="text-sm text-cyber-muted mt-1">配置 LLM、GPU、TLD 和应用选项</p>
      </div>

      <div className="grid grid-cols-4 gap-6">
        {/* Sidebar */}
        <div className="space-y-1">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`w-full flex items-center gap-3 px-4 py-2.5 rounded-lg text-sm font-medium transition-all ${
                activeTab === tab.key
                  ? "bg-cyber-green/10 text-cyber-green border border-cyber-green/20"
                  : "text-cyber-muted hover:text-cyber-text hover:bg-cyber-card/60"
              }`}
            >
              <tab.icon className="w-4 h-4" />
              {tab.label}
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="col-span-3">
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
    { id: "glm-4", name: "GLM-4", baseUrl: "https://open.bigmodel.cn/api/paas/v4/", model: "glm-4", isDefault: true },
    { id: "minimax", name: "MiniMax", baseUrl: "https://api.minimax.chat/v1/", model: "abab6.5s-chat", isDefault: false },
  ];

  return (
    <div className="space-y-5">
      <div className="flex items-center justify-between">
        <h2 className="text-base font-semibold text-cyber-text">LLM 配置</h2>
        <button className="cyber-btn-primary text-sm flex items-center gap-1">
          <Key className="w-3.5 h-3.5" /> 添加配置
        </button>
      </div>

      {configs.map((config) => (
        <div key={config.id} className="glass-panel p-5 space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Brain className="w-4 h-4 text-cyber-green" />
              <span className="text-sm font-semibold text-cyber-text">{config.name}</span>
              {config.isDefault && (
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-cyber-green/10 text-cyber-green">
                  默认
                </span>
              )}
            </div>
            <div className="flex gap-2">
              <button className="cyber-btn-secondary text-xs px-2 py-1 flex items-center gap-1">
                <TestTube2 className="w-3 h-3" /> 测试
              </button>
              <button className="cyber-btn-secondary text-xs px-2 py-1">编辑</button>
            </div>
          </div>
          <div className="grid grid-cols-2 gap-3 text-xs">
            <div>
              <span className="text-cyber-muted">Base URL:</span>{" "}
              <span className="text-cyber-text font-mono">{config.baseUrl}</span>
            </div>
            <div>
              <span className="text-cyber-muted">Model:</span>{" "}
              <span className="text-cyber-text font-mono">{config.model}</span>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

function GPUSettings() {
  return (
    <div className="space-y-5">
      <h2 className="text-base font-semibold text-cyber-text flex items-center gap-2">
        <Monitor className="w-4 h-4 text-cyber-cyan" />
        GPU 配置
      </h2>

      <div className="glass-panel p-5 space-y-4">
        <div>
          <label className="block text-xs text-cyber-muted mb-1">后端选择</label>
          <select className="cyber-input w-full text-sm">
            <option value="auto">自动检测</option>
            <option value="cuda">CUDA (NVIDIA)</option>
            <option value="directml">DirectML (Windows)</option>
            <option value="rocm">ROCm (AMD Linux)</option>
            <option value="coreml">CoreML (Apple Silicon)</option>
            <option value="cpu">CPU 仅限</option>
            <option value="remote">远程 API</option>
          </select>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-xs text-cyber-muted mb-1">设备 ID</label>
            <input type="number" defaultValue={0} className="cyber-input w-full text-sm" />
          </div>
          <div>
            <label className="block text-xs text-cyber-muted mb-1">批处理大小</label>
            <input type="number" defaultValue={500} className="cyber-input w-full text-sm" />
          </div>
        </div>

        <div>
          <label className="block text-xs text-cyber-muted mb-1">本地模型路径</label>
          <input type="text" placeholder="留空使用内置模型" className="cyber-input w-full text-sm font-mono" />
          <p className="text-[10px] text-cyber-muted mt-1">支持 all-MiniLM-L6-v2 ONNX 模型（384 维）</p>
        </div>

        <div className="flex items-center gap-2 p-3 rounded-lg bg-cyber-green/5 border border-cyber-green/20 text-xs">
          <Cpu className="w-4 h-4 text-cyber-green" />
          <span className="text-cyber-green">当前检测: CPU 模式（无 GPU 可用）</span>
        </div>

        <button className="cyber-btn-primary text-sm flex items-center gap-2">
          <Save className="w-3.5 h-3.5" /> 保存配置
        </button>
      </div>
    </div>
  );
}

function TLDSettings() {
  const tlds = [".com", ".net", ".org", ".io", ".ai", ".dev", ".co", ".app", ".info", ".biz"];

  return (
    <div className="space-y-5">
      <div className="flex items-center justify-between">
        <h2 className="text-base font-semibold text-cyber-text flex items-center gap-2">
          <Database className="w-4 h-4 text-cyber-orange" />
          TLD 管理
        </h2>
        <button className="cyber-btn-secondary text-sm flex items-center gap-1">
          <Globe className="w-3.5 h-3.5" /> 在线更新
        </button>
      </div>

      <div className="glass-panel p-5">
        <p className="text-sm text-cyber-muted mb-4">当前内置 {tlds.length} 个 TLD，点击可编辑</p>
        <div className="flex flex-wrap gap-2">
          {tlds.map((tld) => (
            <span
              key={tld}
              className="px-3 py-1.5 rounded-lg bg-cyber-surface border border-cyber-border/30 text-sm text-cyber-text hover:border-cyber-green/30 hover:bg-cyber-green/5 transition-all cursor-pointer"
            >
              {tld}
            </span>
          ))}
        </div>
      </div>
    </div>
  );
}

function GeneralSettings() {
  return (
    <div className="space-y-5">
      <h2 className="text-base font-semibold text-cyber-text">通用设置</h2>

      <div className="glass-panel p-5 space-y-4">
        <div>
          <label className="block text-xs text-cyber-muted mb-1">最大并发数</label>
          <input type="number" defaultValue={50} className="cyber-input w-full text-sm" />
          <p className="text-[10px] text-cyber-muted mt-1">建议 10-100，过高可能触发限流</p>
        </div>

        <div>
          <label className="block text-xs text-cyber-muted mb-1">RDAP 查询超时 (秒)</label>
          <input type="number" defaultValue={10} className="cyber-input w-full text-sm" />
        </div>

        <div>
          <label className="block text-xs text-cyber-muted mb-1">请求间隔范围 (ms)</label>
          <div className="grid grid-cols-2 gap-4">
            <input type="number" defaultValue={50} placeholder="最小" className="cyber-input w-full text-sm" />
            <input type="number" defaultValue={200} placeholder="最大" className="cyber-input w-full text-sm" />
          </div>
        </div>

        <div>
          <label className="block text-xs text-cyber-muted mb-1">重试次数</label>
          <input type="number" defaultValue={3} className="cyber-input w-full text-sm" />
        </div>

        <div>
          <label className="block text-xs text-cyber-muted mb-1">数据存储路径</label>
          <input type="text" defaultValue="~/.domain-scanner" className="cyber-input w-full text-sm font-mono" />
        </div>

        <button className="cyber-btn-primary text-sm flex items-center gap-2">
          <Save className="w-3.5 h-3.5" /> 保存设置
        </button>
      </div>
    </div>
  );
}
