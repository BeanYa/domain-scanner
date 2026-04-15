import { useState, Fragment } from "react";
import { Zap, Regex, Type, Brain, List, ChevronRight, CheckCircle, AlertTriangle, Sparkles, Globe, Search } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { TLD_LIST, POPULAR_TLDS, TLDS_BY_CATEGORY, type TldCategory } from "../data/tlds";

type ScanTab = "regex" | "llm" | "manual";
type Step = 1 | 2 | 3;
type TldView = "popular" | "category";

const categoryLabels: Record<TldCategory, string> = {
  gtld: "通用 gTLD",
  new_gtld: "新 gTLD",
  cctld: "国家 ccTLD",
  other: "其他",
};

export default function NewTask() {
  const [activeTab, setActiveTab] = useState<ScanTab>("regex");
  const [pattern, setPattern] = useState("^[a-z]{4}$");
  const [llmPrompt, setLlmPrompt] = useState("");
  const [manualDomains, setManualDomains] = useState("");
  const [selectedTlds, setSelectedTlds] = useState<string[]>([".com"]);
  const [taskName, setTaskName] = useState("");
  const [created, setCreated] = useState(false);
  const [tldView, setTldView] = useState<TldView>("popular");
  const [tldSearch, setTldSearch] = useState("");
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

  const selectAllVisible = (tlds: string[]) => {
    setSelectedTlds((prev) => {
      const set = new Set(prev);
      for (const t of tlds) set.add(t);
      return [...set];
    });
  };

  const deselectAllVisible = (tlds: string[]) => {
    setSelectedTlds((prev) => {
      const set = new Set(tlds);
      return prev.filter((t) => !set.has(t));
    });
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

  // Filter TLDs by search
  const filteredTlds = tldSearch
    ? TLD_LIST.filter((t) => t.tld.includes(tldSearch.toLowerCase()))
    : null;

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

  // Get visible TLD list based on view mode and search
  const getVisibleTlds = (): string[] => {
    if (filteredTlds) return filteredTlds.map((t) => t.tld);
    if (tldView === "popular") return POPULAR_TLDS.map((t) => t.tld);
    return TLD_LIST.map((t) => t.tld);
  };
  const visibleTlds = getVisibleTlds();

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
        <div className="flex items-center justify-between">
          <label className="flex items-center gap-2 text-sm font-semibold text-cyber-text">
            <Globe className="w-4 h-4 text-cyber-cyan-bright" /> 选择 TLD 后缀
            <span className="text-xs text-cyber-muted-dim font-normal">({TLD_LIST.length} 个可选)</span>
          </label>
          <div className="flex items-center gap-2">
            <button
              onClick={() => {
                if (selectedTlds.length >= visibleTlds.length) deselectAllVisible(visibleTlds);
                else selectAllVisible(visibleTlds);
              }}
              className="text-[11px] text-cyber-muted hover:text-cyber-cyan transition-colors"
            >
              {visibleTlds.every((t) => selectedTlds.includes(t)) ? "取消全选" : "全选当前"}
            </button>
          </div>
        </div>

        {/* Search + View Toggle */}
        <div className="flex items-center gap-3">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-cyber-muted-dim pointer-events-none" />
            <input
              type="text"
              placeholder="搜索 TLD..."
              value={tldSearch}
              onChange={(e) => setTldSearch(e.target.value)}
              className="cyber-input pl-9 text-sm"
            />
          </div>
          {!tldSearch && (
            <div className="flex bg-cyber-surface rounded-lg p-0.5 text-xs border border-cyber-border/30">
              <button
                onClick={() => setTldView("popular")}
                className={`px-3 py-1.5 rounded-md transition-all ${tldView === "popular" ? "bg-cyber-green/15 text-cyber-green" : "text-cyber-muted-dim hover:text-cyber-text"}`}
              >
                热门
              </button>
              <button
                onClick={() => setTldView("category")}
                className={`px-3 py-1.5 rounded-md transition-all ${tldView === "category" ? "bg-cyber-green/15 text-cyber-green" : "text-cyber-muted-dim hover:text-cyber-text"}`}
              >
                按分类
              </button>
            </div>
          )}
        </div>

        {/* Selected count */}
        {selectedTlds.length > 0 && (
          <div className="flex items-center gap-2 text-xs text-cyber-green">
            <CheckCircle className="w-3.5 h-3.5" />
            已选择 <strong>{selectedTlds.length}</strong> 个 TLD
          </div>
        )}

        {/* TLD Grid */}
        {tldSearch ? (
          /* Search results */
          <div className="grid grid-cols-6 gap-1.5">
            {(filteredTlds || []).map((t) => (
              <TldButton key={t.tld} tld={t.tld} selected={selectedTlds.includes(t.tld)} onClick={() => toggleTld(t.tld)} />
            ))}
            {(filteredTlds || []).length === 0 && (
              <p className="col-span-full text-center text-sm text-cyber-muted py-4">未找到匹配的 TLD</p>
            )}
          </div>
        ) : tldView === "popular" ? (
          /* Popular TLDs grid */
          <div className="grid grid-cols-6 gap-1.5">
            {POPULAR_TLDS.map((t) => (
              <TldButton key={t.tld} tld={t.tld} selected={selectedTlds.includes(t.tld)} onClick={() => toggleTld(t.tld)} highlight />
            ))}
          </div>
        ) : (
          /* Category view */
          <div className="space-y-4">
            {(Object.entries(TLDS_BY_CATEGORY) as [TldCategory, typeof TLD_LIST][]).map(([cat, tlds]) => (
              <div key={cat}>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-xs font-semibold text-cyber-muted uppercase tracking-wider">{categoryLabels[cat]} ({tlds.length})</span>
                  <button
                    onClick={() => {
                      if (tlds.every((t) => selectedTlds.includes(t.tld))) deselectAllVisible(tlds.map((t) => t.tld));
                      else selectAllVisible(tlds.map((t) => t.tld));
                    }}
                    className="text-[10px] text-cyber-muted-dim hover:text-cyber-cyan"
                  >
                    {tlds.every((t) => selectedTlds.includes(t.tld)) ? "取消" : "全选"}
                  </button>
                </div>
                <div className="grid grid-cols-8 gap-1">
                  {tlds.map((t) => (
                    <TldButton key={t.tld} tld={t.tld} selected={selectedTlds.includes(t.tld)} onClick={() => toggleTld(t.tld)} compact />
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}

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

function TldButton({ tld, selected, onClick, highlight, compact }: {
  tld: string; selected: boolean; onClick: () => void; highlight?: boolean; compact?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      className={`
        relative rounded-lg border text-center transition-all duration-150
        ${compact ? "px-2 py-1.5 text-xs" : "px-3 py-2.5 text-sm font-semibold"}
        ${selected
          ? "bg-cyber-green/[0.08] border-cyber-green/35 text-cyber-green shadow-neon"
          : highlight
            ? "bg-cyber-surface border-cyber-border/30 text-cyber-text-secondary hover:border-cyber-border-light"
            : "bg-cyber-surface/50 border-cyber-border/20 text-cyber-muted hover:border-cyber-border-light hover:text-cyber-text-secondary"
        }
      `}
    >
      {tld}
      {selected && (
        <span className="absolute -top-1 -right-1 w-3.5 h-3.5 rounded-full bg-cyber-green flex items-center justify-center">
          <CheckCircle className="w-2.5 h-2.5 text-white" />
        </span>
      )}
    </button>
  );
}
