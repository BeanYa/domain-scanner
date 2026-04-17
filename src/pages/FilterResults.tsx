import { useEffect, useMemo, useState } from "react";
import {
  AlertCircle,
  Brain,
  CheckCircle2,
  Filter,
  Inbox,
  Loader2,
  Regex,
  Search,
  SlidersHorizontal,
  Table,
  Type,
  type LucideIcon,
} from "lucide-react";
import { invokeCommand } from "../services/tauri";
import { useTaskStore } from "../store/taskStore";

type FilterMode = "exact" | "fuzzy" | "regex" | "semantic";

interface FilteredResult {
  id: number;
  task_id: string;
  domain: string;
  filter_type: string;
  filter_pattern: string | null;
  is_matched: boolean;
  score: number | null;
  embedding_id: number | null;
}

interface FilterResponse {
  items: FilteredResult[];
  total: number;
}

export default function FilterResults() {
  const [mode, setMode] = useState<FilterMode>("exact");
  const [query, setQuery] = useState("");
  const [threshold, setThreshold] = useState(0);
  const [selectedTaskId, setSelectedTaskId] = useState("");
  const [results, setResults] = useState<FilteredResult[]>([]);
  const [resultTotal, setResultTotal] = useState(0);
  const [filtering, setFiltering] = useState(false);
  const [filterError, setFilterError] = useState<string | null>(null);
  const { tasks, loading, error, fetchTasks } = useTaskStore();

  useEffect(() => {
    fetchTasks("completed");
  }, [fetchTasks]);

  const completedTasks = useMemo(
    () => tasks.filter((task) => task.status === "completed"),
    [tasks]
  );

  useEffect(() => {
    if (completedTasks.length === 0) {
      setSelectedTaskId("");
      return;
    }

    setSelectedTaskId((current) =>
      completedTasks.some((task) => task.id === current)
        ? current
        : completedTasks[0].id
    );
  }, [completedTasks]);

  useEffect(() => {
    setResults([]);
    setResultTotal(0);
    setFilterError(null);
  }, [selectedTaskId, mode]);

  const selectedTask = completedTasks.find((task) => task.id === selectedTaskId);
  const queryText = query.trim();

  const modes: { key: FilterMode; label: string; icon: LucideIcon }[] = [
    { key: "exact", label: "精确匹配", icon: Type },
    { key: "fuzzy", label: "模糊匹配", icon: Search },
    { key: "regex", label: "正则匹配", icon: Regex },
    { key: "semantic", label: "语义筛选", icon: Brain },
  ];

  const handleFilter = async () => {
    if (!selectedTaskId || !queryText) return;

    setFiltering(true);
    setFilterError(null);
    try {
      const command =
        mode === "exact"
          ? "filter_exact"
          : mode === "fuzzy"
          ? "filter_fuzzy"
          : mode === "regex"
          ? "filter_regex"
          : "filter_semantic";
      const request =
        mode === "semantic"
          ? {
              task_id: selectedTaskId,
              description: queryText,
              similarity_threshold: threshold,
              limit: 1000,
            }
          : {
              task_id: selectedTaskId,
              query: queryText,
            };
      const payload = await invokeCommand<string>(command, { request });
      const parsed = JSON.parse(payload) as FilterResponse;
      setResults(parsed.items);
      setResultTotal(parsed.total);
    } catch (e) {
      setFilterError(String(e));
      setResults([]);
      setResultTotal(0);
    } finally {
      setFiltering(false);
    }
  };

  return (
    <div className="page-shell max-w-5xl">
      <div>
        <div className="eyebrow mb-3">RESULT FILTER</div>
        <h1 className="page-heading">二次筛选</h1>
        <p className="page-subtitle">对扫描结果进行精确、模糊、正则或语义筛选，把可用域名收束成可决策列表。</p>
      </div>

      <div className="glass-panel p-5 space-y-4">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-sm font-semibold text-cyber-text flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4 text-cyber-green" />
            已完成任务
          </h2>
          <button
            onClick={() => fetchTasks("completed")}
            className="cyber-btn-secondary cyber-btn-sm"
            disabled={loading}
          >
            {loading ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Search className="w-3.5 h-3.5" />}
            刷新
          </button>
        </div>

        {error ? (
          <div className="flex items-center gap-2 text-sm text-cyber-red">
            <AlertCircle className="w-4 h-4" />
            {error}
          </div>
        ) : loading && completedTasks.length === 0 ? (
          <div className="text-center py-8 text-cyber-muted">
            <Loader2 className="w-10 h-10 mx-auto mb-3 opacity-50 animate-spin" />
            <p className="text-sm">正在加载已完成任务...</p>
          </div>
        ) : completedTasks.length === 0 ? (
          <div className="text-center py-8 text-cyber-muted">
            <Inbox className="w-10 h-10 mx-auto mb-3 opacity-40" />
            <p className="text-sm">暂无已完成任务</p>
            <p className="text-xs text-cyber-muted-dim mt-1">扫描完成后会在这里列出可用于二次筛选的任务</p>
          </div>
        ) : (
          <div className="grid gap-2">
            {completedTasks.map((task) => (
              <label
                key={task.id}
                className={`w-full overflow-hidden text-left rounded-lg border p-3 cursor-pointer transition-all ${
                  selectedTaskId === task.id
                    ? "bg-white/[0.06] border-white/20"
                    : "bg-cyber-bg/50 border-cyber-border hover:border-cyber-border-light"
                }`}
              >
                <div className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-start gap-3">
                  <input
                    type="radio"
                    name="filter-task"
                    value={task.id}
                    checked={selectedTaskId === task.id}
                    onChange={(e) => setSelectedTaskId(e.target.value)}
                    className="mt-1 accent-cyber-green"
                  />
                  <div className="min-w-0 flex-1">
                    <div className="min-w-0">
                      <span
                        className="block min-w-0 truncate text-sm font-semibold text-cyber-text"
                        title={task.name}
                      >
                        {task.name}
                      </span>
                    </div>
                    <div className="mt-2 flex max-h-7 flex-wrap items-center gap-1 overflow-hidden">
                      {task.tlds.slice(0, 4).map((tld) => (
                        <span key={tld} className="badge-neutral text-[11px]">{tld}</span>
                      ))}
                      {task.tlds.length > 4 && (
                        <span className="badge-blue text-[11px]">+{task.tlds.length - 4}</span>
                      )}
                    </div>
                    <div className="mt-2 grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
                      <Metric label="结果" value={task.completed_count} />
                      <Metric label="可用" value={task.available_count} className="text-cyber-green" />
                      <Metric label="错误" value={task.error_count} className={task.error_count > 0 ? "text-cyber-red" : undefined} />
                      <div>
                        <p className="text-cyber-muted-dim">更新时间</p>
                        <p className="text-cyber-text-secondary truncate">{formatDateTime(task.updated_at)}</p>
                      </div>
                    </div>
                  </div>
                  {selectedTaskId === task.id && (
                    <span className="badge-green text-[11px] shrink-0">已选择</span>
                  )}
                </div>
              </label>
            ))}
          </div>
        )}
      </div>

      <div className="glass-panel p-5 space-y-5">
        <div className="grid grid-cols-2 sm:grid-cols-4 bg-cyber-surface rounded-md p-1 border border-cyber-border gap-1">
          {modes.map((m) => (
            <button
              key={m.key}
              onClick={() => setMode(m.key)}
              className={`min-w-0 flex items-center justify-center gap-2 px-3 py-2 rounded-md text-sm font-medium transition-all ${
                mode === m.key
                  ? "bg-white/[0.08] text-white"
                  : "text-cyber-muted hover:text-cyber-text"
              }`}
            >
              <m.icon className="w-4 h-4 shrink-0" />
              <span className="min-w-0 truncate">{m.label}</span>
            </button>
          ))}
        </div>

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
                <span>使用默认 OpenAI 兼容 API 的 embeddings 对已向量化结果做语义召回</span>
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

        {selectedTask && (
          <div className="flex min-w-0 items-center gap-1 text-xs text-cyber-muted">
            <span className="shrink-0">当前源任务：</span>
            <span className="min-w-0 truncate text-cyber-text-secondary" title={selectedTask.name}>
              {selectedTask.name}
            </span>
          </div>
        )}

        {filterError && (
          <div className="flex items-center gap-2 text-sm text-cyber-red">
            <AlertCircle className="w-4 h-4" />
            {filterError}
          </div>
        )}

        <button
          className="cyber-btn-primary flex items-center gap-2"
          onClick={handleFilter}
          disabled={!selectedTaskId || !queryText || filtering}
        >
          {filtering ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <SlidersHorizontal className="w-4 h-4" />
          )}
          {filtering ? "筛选中..." : "执行筛选"}
        </button>
      </div>

      <div className="glass-panel overflow-hidden">
        <div className="px-5 py-3 border-b border-cyber-border/30 flex items-center justify-between">
          <h2 className="text-sm font-semibold text-cyber-text flex items-center gap-2">
            <Table className="w-4 h-4 text-cyber-green" />
            筛选结果
          </h2>
          <span className="text-xs text-cyber-muted-dim">共 {resultTotal.toLocaleString()} 条</span>
        </div>
        {results.length === 0 ? (
          <div className="text-center py-12 text-cyber-muted">
            <Inbox className="w-10 h-10 mx-auto mb-3 opacity-40" />
            <p className="text-sm">暂无筛选结果</p>
            <p className="text-xs text-cyber-muted-dim mt-1">选择已完成任务并执行筛选后，匹配结果会显示在这里</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-cyber-bg-elevated/70 text-cyber-muted-dim">
                <tr className="text-left">
                  <th className="px-5 py-3 font-medium">域名</th>
                  <th className="px-3 py-3 font-medium">筛选方式</th>
                  <th className="px-3 py-3 font-medium">匹配模式</th>
                  <th className="px-3 py-3 font-medium">得分</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-cyber-border/15">
                {results.map((item, index) => (
                  <tr key={`${item.filter_type}-${item.id}-${item.domain}-${index}`} className="hover:bg-cyber-bg-elevated/30">
                    <td className="px-5 py-3 font-mono text-cyber-text">{item.domain}</td>
                    <td className="px-3 py-3">
                      <span className="badge-blue text-[11px]">{getModeLabel(item.filter_type)}</span>
                    </td>
                    <td className="px-3 py-3 text-cyber-text-secondary font-mono max-w-[280px] truncate">
                      {item.filter_pattern ?? "-"}
                    </td>
                    <td className="px-3 py-3 text-cyber-text-secondary tabular-nums">
                      {item.score === null ? "-" : item.score.toFixed(3)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}

function Metric({ label, value, className }: { label: string; value: number; className?: string }) {
  return (
    <div>
      <p className="text-cyber-muted-dim">{label}</p>
      <p className={`font-mono text-cyber-text-secondary tabular-nums ${className ?? ""}`}>
        {value.toLocaleString()}
      </p>
    </div>
  );
}

function formatDateTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

function getModeLabel(value: string) {
  switch (value) {
    case "exact":
      return "精确";
    case "fuzzy":
      return "模糊";
    case "regex":
      return "正则";
    case "semantic":
      return "语义";
    default:
      return value;
  }
}
