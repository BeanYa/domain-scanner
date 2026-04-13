import { useState } from "react";
import { Zap, Regex, Type, Brain, List, ChevronRight, AlertCircle, CheckCircle } from "lucide-react";
import { useNavigate } from "react-router-dom";

type ScanTab = "regex" | "llm" | "manual";

const tldOptions = [
  { value: ".com", label: ".com", count: 456976 },
  { value: ".net", label: ".net", count: 456976 },
  { value: ".org", label: ".org", count: 456976 },
  { value: ".io", label: ".io", count: 0 },
  { value: ".ai", label: ".ai", count: 0 },
  { value: ".dev", label: ".dev", count: 0 },
  { value: ".co", label: ".co", count: 0 },
  { value: ".app", label: ".app", count: 0 },
];

export default function NewTask() {
  const [activeTab, setActiveTab] = useState<ScanTab>("regex");
  const [pattern, setPattern] = useState("^[a-z]{4}$");
  const [llmPrompt, setLlmPrompt] = useState("");
  const [manualDomains, setManualDomains] = useState("");
  const [selectedTlds, setSelectedTlds] = useState<string[]>([".com"]);
  const [taskName, setTaskName] = useState("");
  const [created, setCreated] = useState(false);
  const navigate = useNavigate();

  const tabs: { key: ScanTab; label: string; icon: typeof Regex }[] = [
    { key: "regex", label: "正则/通配符", icon: Regex },
    { key: "llm", label: "LLM 生成", icon: Brain },
    { key: "manual", label: "手动输入", icon: List },
  ];

  const toggleTld = (tld: string) => {
    setSelectedTlds((prev) =>
      prev.includes(tld) ? prev.filter((t) => t !== tld) : [...prev, tld]
    );
  };

  const estimateCount = () => {
    if (activeTab === "regex") {
      if (pattern === "^[a-z]{4}$") return 456976;
      if (pattern === "^[a-z]{3}$") return 17576;
      if (pattern === "^[a-z]{2}$") return 676;
      return 0;
    }
    if (activeTab === "manual") {
      return manualDomains.split("\n").filter((d) => d.trim()).length;
    }
    return 0;
  };

  const handleCreate = () => {
    setCreated(true);
    setTimeout(() => navigate("/tasks"), 1500);
  };

  if (created) {
    return (
      <div className="p-6 flex items-center justify-center min-h-[80vh]">
        <div className="text-center space-y-4 animate-in fade-in">
          <div className="w-16 h-16 mx-auto rounded-full bg-cyber-green/20 flex items-center justify-center">
            <CheckCircle className="w-8 h-8 text-cyber-green" />
          </div>
          <h2 className="text-xl font-bold text-cyber-text">任务创建成功！</h2>
          <p className="text-sm text-cyber-muted">
            创建了 {selectedTlds.length} 个任务，跳过了 0 个已存在
          </p>
          <p className="text-xs text-cyber-muted">正在跳转到任务列表...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6 max-w-4xl">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold neon-text">新建任务</h1>
        <p className="text-sm text-cyber-muted mt-1">配置扫描参数，支持多 TLD 批量创建</p>
      </div>

      {/* Task Name */}
      <div className="glass-panel p-5 space-y-4">
        <label className="block text-sm font-medium text-cyber-text">任务名称</label>
        <input
          type="text"
          placeholder="例如：4字母域名扫描"
          value={taskName}
          onChange={(e) => setTaskName(e.target.value)}
          className="cyber-input w-full"
        />
      </div>

      {/* Scan Mode */}
      <div className="glass-panel p-5 space-y-4">
        <label className="block text-sm font-medium text-cyber-text">扫描模式</label>
        <div className="flex bg-cyber-surface rounded-lg p-1 border border-cyber-border/50">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all ${
                activeTab === tab.key
                  ? "bg-cyber-green/20 text-cyber-green"
                  : "text-cyber-muted hover:text-cyber-text"
              }`}
            >
              <tab.icon className="w-4 h-4" />
              {tab.label}
            </button>
          ))}
        </div>

        {activeTab === "regex" && (
          <div className="space-y-3">
            <input
              type="text"
              placeholder="正则表达式，如 ^[a-z]{4}$"
              value={pattern}
              onChange={(e) => setPattern(e.target.value)}
              className="cyber-input w-full font-mono text-sm"
            />
            <div className="flex items-center gap-2 text-xs text-cyber-muted">
              <Type className="w-3.5 h-3.5" />
              <span>通配符 ? 等同于正则 . ，* 等同于 .*</span>
            </div>
            {estimateCount() > 0 && (
              <div className="flex items-center gap-2 p-3 rounded-lg bg-cyber-green/5 border border-cyber-green/20">
                <AlertCircle className="w-4 h-4 text-cyber-green" />
                <span className="text-sm text-cyber-green">
                  预估每个 TLD 生成 <strong>{estimateCount().toLocaleString()}</strong> 个域名
                </span>
              </div>
            )}
          </div>
        )}

        {activeTab === "llm" && (
          <div className="space-y-3">
            <textarea
              placeholder="描述你想要的域名类型，如：AI 技术相关的简短品牌名"
              value={llmPrompt}
              onChange={(e) => setLlmPrompt(e.target.value)}
              className="cyber-input w-full h-24 resize-none"
            />
            <div className="flex items-center gap-2 text-xs text-cyber-muted">
              <Brain className="w-3.5 h-3.5" />
              <span>LLM 将根据描述生成候选域名列表</span>
            </div>
          </div>
        )}

        {activeTab === "manual" && (
          <div className="space-y-3">
            <textarea
              placeholder="每行一个域名前缀，如：&#10;techworld&#10;codehub&#10;devstream"
              value={manualDomains}
              onChange={(e) => setManualDomains(e.target.value)}
              className="cyber-input w-full h-32 resize-none font-mono text-sm"
            />
            <p className="text-xs text-cyber-muted">
              已输入 {manualDomains.split("\n").filter((d) => d.trim()).length} 个域名
            </p>
          </div>
        )}
      </div>

      {/* TLD Selector */}
      <div className="glass-panel p-5 space-y-4">
        <label className="block text-sm font-medium text-cyber-text">选择 TLD</label>
        <div className="grid grid-cols-4 gap-2">
          {tldOptions.map((tld) => {
            const isSelected = selectedTlds.includes(tld.value);
            return (
              <button
                key={tld.value}
                onClick={() => toggleTld(tld.value)}
                className={`p-3 rounded-lg border text-sm font-medium transition-all ${
                  isSelected
                    ? "bg-cyber-green/10 border-cyber-green/40 text-cyber-green shadow-neon"
                    : "bg-cyber-surface border-cyber-border/40 text-cyber-muted hover:border-cyber-green/20 hover:text-cyber-text"
                }`}
              >
                <span className="text-base">{tld.label}</span>
                {tld.count > 0 && activeTab === "regex" && (
                  <p className="text-[10px] mt-1 opacity-60">{tld.count.toLocaleString()}</p>
                )}
              </button>
            );
          })}
        </div>
        {selectedTlds.length > 1 && (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-cyber-orange/5 border border-cyber-orange/20">
            <AlertCircle className="w-4 h-4 text-cyber-orange" />
            <span className="text-sm text-cyber-orange">
              将为每个 TLD 创建独立任务（共 {selectedTlds.length} 个），并归入同一批次
            </span>
          </div>
        )}
      </div>

      {/* Submit */}
      <div className="flex items-center justify-between">
        <p className="text-sm text-cyber-muted">
          将创建 {selectedTlds.length} 个任务
        </p>
        <button onClick={handleCreate} className="cyber-btn-primary flex items-center gap-2">
          <Zap className="w-4 h-4" />
          创建任务
          <ChevronRight className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
