import { useState, Fragment } from "react";
import { Zap, Regex, Type, Brain, List, ChevronRight, CheckCircle, AlertTriangle, Sparkles } from "lucide-react";
import { useNavigate } from "react-router-dom";

type ScanTab = "regex" | "llm" | "manual";
type Step = 1 | 2 | 3;

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

  const tabs: { key: ScanTab; label: string; icon: typeof Regex; desc: string }[] = [
    { key: "regex", label: "正则 / 通配符", icon: Regex, desc: "用正则表达式或通配符生成域名前缀列表" },
    { key: "llm", label: "LLM 智能生成", icon: Brain, desc: "描述需求，AI 自动生成候选域名前缀" },
    { key: "manual", label: "手动输入", icon: List, desc: "逐行输入自定义的域名前缀" },
  ];

  const toggleTld = (tld: string) => {
    setSelectedTlds((prev) =>
      prev.includes(tld) ? prev.filter((t) => t !== tld) : [...prev, tld]
    );
  };

  const estimateCountPerTld = () => {
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

  // Success state
  if (created) {
    return (
      <div className="flex items-center justify-center min-h-[70vh] animate-fade-in">
        <div className="text-center space-y-5 scale-in max-w-sm">
          <div className="w-20 h-20 mx-auto rounded-2xl bg-cyber-green/10 border border-cyber-green/20 flex items-center justify-center shadow-neon">
            <CheckCircle className="w-10 h-10 text-cyber-green" />
          </div>
          <h2 className="text-xl font-bold text-cyber-text">任务创建成功</h2>
          <p className="text-sm text-cyber-muted leading-relaxed">
            已创建 <span className="text-cyber-green font-semibold">1 个任务</span>，
            覆盖 <span className="text-cyber-blue font-semibold">{selectedTlds.length} 个 TLD 后缀</span>
          </p>
          <p className="text-xs text-cyber-muted-dim">正在跳转到任务列表...</p>
        </div>
      </div>
    );
  }

  const totalEstimate = estimateCountPerTld() * selectedTlds.length;

  return (
    <div className="space-y-6 max-w-4xl animate-fade-in">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold neon-text tracking-tight">新建扫描任务</h1>
        <p className="text-sm text-cyber-muted mt-1">配置扫描参数，支持多前缀 x 多后缀笛卡尔积组合</p>
      </div>

      {/* Step Indicator */}
      <div className="flex items-center gap-3 px-1">
        {[1, 2, 3].map((step) => (
          <Fragment key={step}>
            {step > 1 && <div className={`h-[2px] flex-1 rounded-full ${step <= 3 ? "bg-cyber-green/30" : "bg-cyber-border/40"}`} />}
            <div className={`flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium transition-colors ${
              step <= 3 ? "bg-cyber-green/10 text-cyber-green border border-cyber-green/20" : "bg-cyber-surface text-cyber-muted-dim"
            }`}>
              步骤 {step}
            </div>
          </Fragment>
        ))}
      </div>

      {/* Task Name Card */}
      <section className="glass-panel p-5 space-y-3">
        <label className="flex items-center gap-2 text-sm font-semibold text-cyber-text">
          <Sparkles className="w-4 h-4 text-cyber-orange" />
          任务名称
        </label>
        <input
          type="text"
          placeholder="例如：4字母 .com/.net 扫描"
          value={taskName}
          onChange={(e) => setTaskName(e.target.value)}
          className="cyber-input text-base"
        />
        {!taskName && (
          <p className="text-[11px] text-cyber-muted-dim flex items-center gap-1.5">
            <AlertTriangle className="w-3 h-3" />
            留空将自动根据模式与 TLD 生成名称
          </p>
        )}
      </section>

      {/* Scan Mode Card */}
      <section className="glass-panel p-5 space-y-4">
        <label className="block text-sm font-semibold text-cyber-text">扫描模式</label>

        {/* Tab Switcher */}
        <div className="grid grid-cols-3 gap-2 p-1 bg-cyber-bg-elevated rounded-xl border border-cyber-border/30">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`
                relative flex flex-col items-center gap-1.5 px-3 py-3 rounded-lg text-sm transition-all duration-200
                ${activeTab === tab.key
                  ? "bg-gradient-to-b from-cyber-green/15 to-cyber-green/5 text-cyber-green border border-cyber-green/25 shadow-neon"
                  : "text-cyber-muted hover:text-cyber-text-secondary hover:bg-cyber-card/50 border border-transparent"
                }
              `}
            >
              <tab.icon className={`w-5 h-5 ${activeTab === tab.key ? "" : "opacity-50"}`} />
              <span className="font-medium">{tab.label}</span>
              <span className="text-[10px] opacity-60 hidden sm:block">{tab.desc}</span>
            </button>
          ))}
        </div>

        {/* Tab Content */}
        <div className="animate-fade-in min-h-[100px]">
          {activeTab === "regex" && (
            <div className="space-y-3">
              <div className="cyber-input-icon">
                <Regex />
                <input
                  type="text"
                  placeholder="正则表达式，如 ^[a-z]{4}$"
                  value={pattern}
                  onChange={(e) => setPattern(e.target.value)}
                  className="cyber-input w-full font-mono text-sm"
                />
              </div>
              <div className="flex items-start gap-2 text-xs text-cyber-muted-dim bg-cyber-bg-elevated/60 p-3 rounded-lg border border-cyber-border/20">
                <Type className="w-3.5 h-3.5 mt-0.5 shrink-0 text-cyber-muted-dim/60" />
                <span>通配符 <code className="font-mono text-cyber-orange">?</code> 等同于正则 <code className="font-mono text-cyber-orange">.</code>，
                  <code className="font-mono text-cyber-orange">*</code> 等同于 <code className="font-mono text-cyber-orange">.*</code></span>
              </div>
            </div>
          )}

          {activeTab === "llm" && (
            <div className="space-y-3">
              <textarea
                placeholder="描述你想要的域名类型，例如：&#10;• AI 技术相关的简短品牌名（3-5个字母）&#10;• Web3 / Crypto 方向的创新词汇&#10;• 适合 SaaS 产品的现代英文词"
                value={llmPrompt}
                onChange={(e) => setLlmPrompt(e.target.value)}
                className="cyber-input w-full h-28 resize-none text-sm leading-relaxed"
              />
              <div className="flex items-start gap-2 text-xs text-cyber-muted-dim bg-cyber-bg-elevated/60 p-3 rounded-lg border border-cyber-border/20">
                <Brain className="w-3.5 h-3.5 mt-0.5 shrink-0 text-cyber-purple/60" />
                <span>LLM 将根据你的描述智能生成候选域名前缀，支持中文描述</span>
              </div>
            </div>
          )}

          {activeTab === "manual" && (
            <div className="space-y-3">
              <textarea
                placeholder={"每行一个域名前缀，例如：\ntechworld\ncodehub\ndevstream\npixelcraft\n"}
                value={manualDomains}
                onChange={(e) => setManualDomains(e.target.value)}
                className="cyber-input w-full h-32 resize-none font-mono text-sm leading-relaxed"
              />
              <div className="flex items-center justify-between text-xs">
                <span className="text-cyber-muted-dim">
                  已输入 <strong className="text-cyber-text">{manualDomains.split("\n").filter((d) => d.trim()).length}</strong> 个前缀
                </span>
              </div>
            </div>
          )}
        </div>
      </section>

      {/* TLD Selector Card */}
      <section className="glass-panel p-5 space-y-4">
        <label className="flex items-center justify-between">
          <span className="flex items-center gap-2 text-sm font-semibold text-cyber-text">
            <Globe className="w-4 h-4 text-cyber-cyan-bright" /> 选择 TLD 后缀
          </span>
          <button
            onClick={() => setSelectedTlds(selectedTlds.length === tldOptions.length ? [] : tldOptions.map((t) => t.value))}
            className="text-[11px] text-cyber-muted hover:text-cyber-cyan transition-colors"
          >
            {selectedTlds.length === tldOptions.length ? "取消全选" : "全选"}
          </button>
        </label>

        <div className="grid grid-cols-4 gap-2">
          {tldOptions.map((tld) => {
            const isSelected = selectedTlds.includes(tld.value);
            return (
              <button
                key={tld.value}
                onClick={() => toggleTld(tld.value)}
                className={`
                  relative p-3.5 rounded-xl border text-sm font-semibold transition-all duration-200 group
                  ${isSelected
                    ? "bg-cyber-green/[0.08] border-cyber-green/35 text-cyber-green shadow-neon"
                    : "bg-cyber-surface border-cyber-border/30 text-cyber-muted hover:border-cyber-border-light hover:text-cyber-text-secondary"
                  }
                `}
              >
                <span className="text-base">{tld.label}</span>
                {tld.count > 0 && activeTab === "regex" && (
                  <p className={`text-[10px] mt-1 font-mono ${isSelected ? "text-cyber-green/70" : "text-cyber-muted-dim"}`}>
                    {(tld.count / 1000).toFixed(0)}k
                  </p>
                )}
                {isSelected && (
                  <span className="absolute top-1.5 right-1.5 w-4 h-4 rounded-full bg-cyber-green/20 flex items-center justify-center">
                    <CheckCircle className="w-3 h-3 text-cyber-green" />
                  </span>
                )}
              </button>
            );
          })}
        </div>

        {/* Cartesian product info */}
        {selectedTlds.length > 1 && totalEstimate > 0 && (
          <div className="flex items-start gap-2.5 p-3.5 rounded-xl bg-cyber-blue/[0.04] border border-cyber-blue/18">
            <AlertTriangle className="w-4 h-4 text-cyber-blue/80 shrink-0 mt-0.5" />
            <div>
              <p className="text-sm text-cyber-blue">
                笛卡尔积：{estimateCountPerTld().toLocaleString()} 前缀 × {selectedTlds.length} TLDs ={" "}
                <strong>{totalEstimate.toLocaleString()}</strong> 组合
              </p>
              <p className="text-[11px] text-cyber-muted-dim mt-1">
                所有 TLD 将合并为单个任务，统一管理与进度追踪
              </p>
            </div>
          </div>
        )}
      </section>

      {/* Submit Bar */}
      <div className="glass-panel p-5 flex items-center justify-between sticky bottom-6">
        <div className="space-y-0.5">
          <p className="text-sm text-cyber-text-secondary">
            将创建 <strong className="text-cyber-green">1</strong> 个任务 · 覆盖{" "}
            <strong className="text-cyber-blue">{selectedTlds.length}</strong> 个 TLD
          </p>
          {totalEstimate > 0 && (
            <p className="text-xs text-cyber-muted-dim">
              预计扫描 <strong className="font-mono">{totalEstimate.toLocaleString()}</strong> 个域名组合
            </p>
          )}
        </div>
        <button
          onClick={handleCreate}
          disabled={selectedTlds.length === 0 || (activeTab === "regex" && !pattern)}
          className="cyber-btn-primary cyber-btn-lg disabled:opacity-40 disabled:hover:translate-y-0 disabled:hover:shadow-none"
        >
          <Zap className="w-4.5 h-4.5" />
          创建并启动
          <ChevronRight className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}

/* Quick import for Globe icon used inline */
function Globe({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/>
      <path d="M2 12h20"/><path d="M12 2c2.5 2.8 4 6.4 4 10s-1.5 7.2-4 10c-2.5-2.8-4-6.4-4-10s1.5-7.2 4-10z"/>
    </svg>
  );
}
