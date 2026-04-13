import { useState } from "react";
import { Search, Filter, Brain, Regex, Type, CheckCircle, XCircle, SlidersHorizontal } from "lucide-react";

type FilterMode = "exact" | "fuzzy" | "regex" | "semantic";

const mockItems = [
  { domain: "techworld.com", matched: true, score: 1.0 },
  { domain: "codehub.com", matched: true, score: 0.98 },
  { domain: "devstream.com", matched: true, score: 0.95 },
  { domain: "aifusion.com", matched: true, score: 0.92 },
  { domain: "neuralops.com", matched: false, score: 0.45 },
  { domain: "dataforge.com", matched: false, score: 0.38 },
];

export default function FilterResults() {
  const [mode, setMode] = useState<FilterMode>("exact");
  const [query, setQuery] = useState("");
  const [threshold, setThreshold] = useState(0.8);

  const modes: { key: FilterMode; label: string; icon: typeof Filter }[] = [
    { key: "exact", label: "精确匹配", icon: Type },
    { key: "fuzzy", label: "模糊匹配", icon: Search },
    { key: "regex", label: "正则匹配", icon: Regex },
    { key: "semantic", label: "语义筛选", icon: Brain },
  ];

  return (
    <div className="p-6 space-y-6 max-w-5xl">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold neon-text">二次筛选</h1>
        <p className="text-sm text-cyber-muted mt-1">对扫描结果进行精确/模糊/正则/语义筛选</p>
      </div>

      {/* Filter Panel */}
      <div className="glass-panel p-5 space-y-5">
        {/* Mode Tabs */}
        <div className="flex bg-cyber-surface rounded-lg p-1 border border-cyber-border/50">
          {modes.map((m) => (
            <button
              key={m.key}
              onClick={() => setMode(m.key)}
              className={`flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all ${
                mode === m.key
                  ? "bg-cyber-green/20 text-cyber-green"
                  : "text-cyber-muted hover:text-cyber-text"
              }`}
            >
              <m.icon className="w-4 h-4" />
              {m.label}
            </button>
          ))}
        </div>

        {/* Query Input */}
        <div className="space-y-3">
          {mode === "semantic" ? (
            <>
              <textarea
                placeholder="描述你想要的域名特征，如：技术相关的简短品牌名"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                className="cyber-input w-full h-20 resize-none"
              />
              <div className="flex items-center gap-4">
                <label className="text-xs text-cyber-muted">相似度阈值</label>
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.05"
                  value={threshold}
                  onChange={(e) => setThreshold(parseFloat(e.target.value))}
                  className="flex-1 accent-cyber-green"
                />
                <span className="text-sm text-cyber-green font-mono w-10 text-right">
                  {threshold.toFixed(2)}
                </span>
              </div>
              <div className="flex items-center gap-2 text-xs text-cyber-muted">
                <Brain className="w-3.5 h-3.5" />
                <span>使用本地 ONNX 模型或远程 API 生成 embedding</span>
              </div>
            </>
          ) : (
            <input
              type="text"
              placeholder={
                mode === "exact"
                  ? "输入精确域名，如 techworld"
                  : mode === "fuzzy"
                  ? "输入模糊关键词，如 tech"
                  : "输入正则表达式，如 ^tech.*"
              }
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              className={`cyber-input w-full ${mode === "regex" ? "font-mono text-sm" : ""}`}
            />
          )}
        </div>

        <button className="cyber-btn-primary flex items-center gap-2">
          <SlidersHorizontal className="w-4 h-4" />
          执行筛选
        </button>
      </div>

      {/* Results */}
      <div className="glass-panel overflow-hidden">
        <div className="px-5 py-3 border-b border-cyber-border/30 flex items-center justify-between">
          <h2 className="text-sm font-semibold text-cyber-text">筛选结果</h2>
          <div className="flex items-center gap-3 text-xs text-cyber-muted">
            <span>匹配 4 / 6</span>
          </div>
        </div>
        <table className="w-full">
          <thead>
            <tr className="text-xs text-cyber-muted border-b border-cyber-border/20">
              <th className="text-left px-5 py-2 font-medium">域名</th>
              <th className="text-center px-5 py-2 font-medium">匹配状态</th>
              {mode === "semantic" && (
                <th className="text-right px-5 py-2 font-medium">相似度</th>
              )}
            </tr>
          </thead>
          <tbody>
            {mockItems.map((item, i) => (
              <tr key={i} className="border-b border-cyber-border/10 hover:bg-cyber-card/30 transition-colors">
                <td className="px-5 py-2.5 text-sm font-mono text-cyber-text">{item.domain}</td>
                <td className="px-5 py-2.5 text-center">
                  {item.matched ? (
                    <span className="inline-flex items-center gap-1 text-xs text-cyber-green">
                      <CheckCircle className="w-3.5 h-3.5" /> 匹配
                    </span>
                  ) : (
                    <span className="inline-flex items-center gap-1 text-xs text-cyber-muted">
                      <XCircle className="w-3.5 h-3.5" /> 不匹配
                    </span>
                  )}
                </td>
                {mode === "semantic" && (
                  <td className="px-5 py-2.5 text-right">
                    <span className={`text-xs font-mono ${item.matched ? "text-cyber-green" : "text-cyber-muted"}`}>
                      {item.score.toFixed(2)}
                    </span>
                  </td>
                )}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
