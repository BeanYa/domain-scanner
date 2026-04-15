import { useState, useEffect, useCallback, useMemo } from "react";
import {
  ArrowLeft,
  Play,
  Square,
  RotateCcw,
  Download,
  Cpu,
  Filter,
  Terminal,
  ChevronDown,
  Table,
  Inbox,
  Trash2,
} from "lucide-react";
import { useNavigate, useParams } from "react-router-dom";
import { useTaskStore } from "../store/taskStore";
import { invokeCommand, listenEvent } from "../services/tauri";
import type { LogEntry, ScanItem, TaskRun } from "../types";

const logColors: Record<string, string> = {
  info: "text-cyber-green",
  warn: "text-cyber-orange",
  error: "text-cyber-red",
};

const resultStatusConfig: Record<ScanItem["status"], { label: string; className: string }> = {
  available: { label: "可用", className: "badge-green" },
  unavailable: { label: "已注册", className: "badge-neutral" },
  error: { label: "错误", className: "badge-red" },
  checking: { label: "检查中", className: "badge-blue" },
  pending: { label: "等待中", className: "badge-neutral" },
};

interface ScanProgress {
  task_id: string;
  completed_count: number;
  total_count: number;
  available_count: number;
  error_count: number;
  percent: number;
}

export default function TaskDetail() {
  const { id } = useParams();
  const navigate = useNavigate();
  const [showLogs, setShowLogs] = useState(true);
  const [logFilter, setLogFilter] = useState<string>("all");
  const [actionLoading, setActionLoading] = useState<"start" | "stop" | "delete" | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [runs, setRuns] = useState<TaskRun[]>([]);
  const [runsLoading, setRunsLoading] = useState(false);
  const [runsError, setRunsError] = useState<string | null>(null);
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const [results, setResults] = useState<ScanItem[]>([]);
  const [resultsLoading, setResultsLoading] = useState(false);
  const [resultsError, setResultsError] = useState<string | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [logsLoading, setLogsLoading] = useState(false);
  const [logsError, setLogsError] = useState<string | null>(null);
  const { tasks, fetchTasks, startTask, pauseTask, rerunTask, deleteTask } = useTaskStore();

  const fetchRuns = useCallback(async () => {
    if (!id) return;
    setRunsLoading(true);
    setRunsError(null);
    try {
      const result = await invokeCommand<string>("list_task_runs", {
        request: { task_id: id },
      });
      const parsed = JSON.parse(result) as TaskRun[];
      setRuns(parsed);
      setSelectedRunId((current) => {
        if (current && parsed.some((run) => run.id === current)) return current;
        return parsed[0]?.id ?? null;
      });
    } catch (e) {
      setRunsError(String(e));
    } finally {
      setRunsLoading(false);
    }
  }, [id]);

  const fetchResults = useCallback(async () => {
    if (!id || !selectedRunId) {
      setResults([]);
      return;
    }
    setResultsLoading(true);
    setResultsError(null);
    try {
      const result = await invokeCommand<string>("list_scan_items", {
        request: {
          task_id: id,
          run_id: selectedRunId,
          limit: 200,
          offset: 0,
        },
      });
      setResults(JSON.parse(result) as ScanItem[]);
    } catch (e) {
      setResultsError(String(e));
    } finally {
      setResultsLoading(false);
    }
  }, [id, selectedRunId]);

  const fetchLogs = useCallback(async () => {
    if (!id || !selectedRunId) {
      setLogs([]);
      return;
    }
    setLogsLoading(true);
    setLogsError(null);
    try {
      const result = await invokeCommand<string>("get_logs", {
        request: {
          task_id: id,
          run_id: selectedRunId,
          level: logFilter === "all" ? null : logFilter,
          limit: 200,
          offset: 0,
        },
      });
      setLogs(JSON.parse(result) as LogEntry[]);
    } catch (e) {
      setLogsError(String(e));
    } finally {
      setLogsLoading(false);
    }
  }, [id, selectedRunId, logFilter]);

  useEffect(() => {
    if (tasks.length === 0) fetchTasks();
  }, [tasks.length, fetchTasks]);

  useEffect(() => {
    fetchRuns();
  }, [fetchRuns]);

  useEffect(() => {
    fetchResults();
  }, [fetchResults]);

  useEffect(() => {
    fetchLogs();
  }, [fetchLogs]);

  // Listen for scan progress events
  useEffect(() => {
    const unlisten = listenEvent<ScanProgress>("scan-progress", (progress) => {
      if (progress.task_id === id) {
        fetchTasks();
        fetchRuns();
        fetchResults();
        fetchLogs();
      }
    });
    const unlistenComplete = listenEvent<ScanProgress>("scan-complete", (progress) => {
      if (progress.task_id === id) {
        fetchTasks();
        fetchRuns();
        fetchResults();
        fetchLogs();
      }
    });
    return () => { unlisten.then(fn => fn()); unlistenComplete.then(fn => fn()); };
  }, [id, fetchTasks, fetchRuns, fetchResults, fetchLogs]);

  useEffect(() => {
    const unlisten = listenEvent<LogEntry>("task-log-created", (log) => {
      if (log.task_id !== id) return;
      if (selectedRunId && log.run_id !== selectedRunId) return;
      if (logFilter !== "all" && log.level !== logFilter) return;
      setLogs((current) => [log, ...current.filter((entry) => entry.id !== log.id)].slice(0, 200));
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [id, selectedRunId, logFilter]);

  // Auto-refresh while task is running (fallback for missed events)
  useEffect(() => {
    const task = tasks.find((t) => t.id === id);
    if (!task || task.status !== "running") return;
    const interval = setInterval(() => {
      fetchTasks();
      fetchRuns();
      fetchResults();
      fetchLogs();
    }, 3000);
    return () => clearInterval(interval);
  }, [tasks, id, fetchTasks, fetchRuns, fetchResults, fetchLogs]);

  const task = tasks.find((t) => t.id === id);
  const selectedRun = useMemo(
    () => runs.find((run) => run.id === selectedRunId) ?? runs[0] ?? null,
    [runs, selectedRunId]
  );
  const availableResults = useMemo(
    () => results.filter((item) => item.status === "available"),
    [results]
  );

  const handleStart = async () => {
    if (!task) return;
    setActionError(null);
    setActionLoading("start");
    try {
      await startTask(task.id);
      await fetchTasks();
      await fetchRuns();
    } catch (e) {
      setActionError(String(e));
    } finally {
      setActionLoading(null);
    }
  };

  const handleStop = async () => {
    if (!task) return;
    setActionError(null);
    setActionLoading("stop");
    try {
      await pauseTask(task.id);
      await fetchTasks();
      await fetchRuns();
      await fetchResults();
      await fetchLogs();
    } catch (e) {
      setActionError(String(e));
    } finally {
      setActionLoading(null);
    }
  };

  const handleRerun = async () => {
    if (!task) return;
    const confirmed = window.confirm(`确定重新运行任务“${task.name}”吗？将创建新的运行记录并保留历史结果。`);
    if (!confirmed) return;

    setActionError(null);
    setActionLoading("start");
    try {
      const newRunId = await rerunTask(task.id);
      await fetchTasks();
      await fetchRuns();
      setSelectedRunId(newRunId);
    } catch (e) {
      setActionError(String(e));
    } finally {
      setActionLoading(null);
    }
  };

  const handleDelete = async () => {
    if (!task) return;
    const confirmed = window.confirm(`确定删除任务“${task.name}”吗？该操作会同时清理扫描结果和日志。`);
    if (!confirmed) return;

    setActionError(null);
    setActionLoading("delete");
    try {
      await deleteTask(task.id);
      navigate("/tasks");
    } catch (e) {
      setActionError(String(e));
      setActionLoading(null);
    }
  };

  if (!task) {
    return (
      <div className="space-y-6 animate-fade-in max-w-5xl">
        <div className="flex items-center gap-4">
          <button onClick={() => navigate("/tasks")} className="cyber-btn-icon cyber-btn-ghost">
            <ArrowLeft className="w-4.5 h-4.5" />
          </button>
          <h1 className="text-xl font-bold text-cyber-text">任务未找到</h1>
        </div>
        <div className="glass-panel p-12 text-center text-cyber-muted">
          <Inbox className="w-10 h-10 mx-auto mb-3 opacity-40" />
          <p className="text-sm">未找到 ID 为 {id} 的任务</p>
        </div>
      </div>
    );
  }

  const summary = selectedRun ?? {
    total_count: task.total_count,
    completed_count: task.completed_count,
    available_count: task.available_count,
    error_count: task.error_count,
  };
  const progress = summary.total_count > 0 ? Math.round((summary.completed_count / summary.total_count) * 100) : 0;

  return (
    <div className="space-y-6 animate-fade-in max-w-5xl">
      {/* Header */}
      <div className="flex items-center gap-4">
        <button
          onClick={() => navigate("/tasks")}
          className="cyber-btn-icon cyber-btn-ghost"
        >
          <ArrowLeft className="w-4.5 h-4.5" />
        </button>
        <div className="flex-1">
          <h1 className="text-xl font-bold text-cyber-text">{task.name}</h1>
          <div className="flex items-center gap-2 mt-1 flex-wrap">
            {task.tlds.map((tld) => (
              <span key={tld} className="badge-neutral">{tld}</span>
            ))}
            <span className="text-xs text-cyber-muted-dim font-mono">ID: {id}</span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {(task.status === "pending" || task.status === "paused") && (
            <button
              className="cyber-btn-primary cyber-btn-sm"
              onClick={handleStart}
              disabled={actionLoading !== null}
            >
              <Play className="w-3.5 h-3.5" /> {actionLoading === "start" ? "启动中..." : task.status === "pending" ? "启动" : "恢复"}
            </button>
          )}
          {task.status === "running" && (
            <button
              className="cyber-btn-secondary cyber-btn-sm"
              onClick={handleStop}
              disabled={actionLoading !== null}
            >
              <Square className="w-3.5 h-3.5" /> {actionLoading === "stop" ? "停止中..." : "停止"}
            </button>
          )}
          {task.status !== "running" && runs.length > 0 && (
            <button
              className="cyber-btn-secondary cyber-btn-sm"
              onClick={handleRerun}
              disabled={actionLoading !== null}
            >
              <RotateCcw className="w-3.5 h-3.5" /> {actionLoading === "start" ? "重新运行中..." : "Rerun"}
            </button>
          )}
          <button
            className="cyber-btn-secondary cyber-btn-sm text-cyber-red border-cyber-red/20 hover:border-cyber-red/40 hover:text-cyber-red"
            onClick={handleDelete}
            disabled={actionLoading !== null}
          >
            <Trash2 className="w-3.5 h-3.5" /> {actionLoading === "delete" ? "删除中..." : "删除"}
          </button>
          <button className="cyber-btn-secondary cyber-btn-sm"><Download className="w-3.5 h-3.5" /> 导出</button>
          <button className="cyber-btn-secondary cyber-btn-sm"><Cpu className="w-3.5 h-3.5" /> 向量化</button>
          <button className="cyber-btn-secondary cyber-btn-sm"><Filter className="w-3.5 h-3.5" /> 筛选</button>
        </div>
      </div>

      {actionError && (
        <div className="glass-panel px-4 py-3 text-sm text-cyber-red">
          {actionError}
        </div>
      )}

      <div className="glass-panel p-4 space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="section-title m-0">运行记录</h2>
          <div className="text-xs text-cyber-muted-dim">
            {runsLoading ? "加载中..." : `${runs.length} 次运行`}
          </div>
        </div>
        {runsError ? (
          <div className="text-sm text-cyber-red">{runsError}</div>
        ) : runs.length === 0 ? (
          <div className="text-sm text-cyber-muted">该任务还没有运行记录。</div>
        ) : (
          <div className="flex flex-wrap gap-2">
            {runs.map((run) => (
              <button
                key={run.id}
                onClick={() => setSelectedRunId(run.id)}
                className={`px-3 py-2 rounded-lg border text-left transition-colors ${
                  selectedRun?.id === run.id
                    ? "border-cyber-green/35 bg-cyber-green/[0.08] text-cyber-green"
                    : "border-cyber-border/25 bg-cyber-surface/50 text-cyber-text-secondary hover:border-cyber-border-light"
                }`}
              >
                <div className="text-xs font-semibold">Run #{run.run_number}</div>
                <div className="text-[10px] opacity-75">{formatDateTime(run.started_at)}</div>
                <div className="text-[10px] opacity-60 font-mono">{run.id.slice(0, 8)}</div>
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Progress Section */}
      <div className="grid grid-cols-[200px_1fr] gap-4 glass-panel p-5 items-center">
        <div className="relative w-32 h-32 mx-auto shrink-0">
          <svg className="w-full h-full -rotate-90" viewBox="0 0 128 128">
            <circle cx="64" cy="64" r="56" fill="none" stroke="#252D3A" strokeWidth="8" />
            <circle
              cx="64" cy="64" r="56" fill="none"
              stroke="url(#progressGrad)" strokeWidth="8"
              strokeLinecap="round"
              strokeDasharray={`${(progress / 100) * 351.86} 351.86`}
              style={{ transition: "stroke-dasharray 700ms ease-out" }}
            />
            <defs>
              <linearGradient id="progressGrad" x1="0%" y1="0%" x2="100%" y2="0%">
                <stop offset="0%" stopColor="#00E5A0" />
                <stop offset="100%" stopColor="#00C9DB" />
              </linearGradient>
            </defs>
          </svg>
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            <span className="text-2xl font-bold neon-text tabular-nums">{progress}%</span>
            <span className="text-[10px] text-cyber-muted">进度</span>
          </div>
        </div>

        <div className="grid grid-cols-3 gap-4">
          <div className="rounded-xl bg-cyber-bg-elevated/60 border border-cyber-border/20 p-4 space-y-1">
            <p className="text-xs text-cyber-muted">已完成</p>
            <p className="text-xl font-bold text-cyber-text tabular-nums">{summary.completed_count.toLocaleString()}</p>
            <p className="text-[10px] text-cyber-muted-dim font-mono tabular-nums">/ {summary.total_count.toLocaleString()}</p>
          </div>
          <div className="rounded-xl bg-cyber-bg-elevated/60 border border-cyber-border/20 p-4 space-y-1">
            <p className="text-xs text-cyber-muted">可用域名</p>
            <p className="text-xl font-bold text-cyber-green tabular-nums">{summary.available_count.toLocaleString()}</p>
            <p className="text-[10px] text-cyber-muted-dim">
              {summary.completed_count > 0 ? ((summary.available_count / summary.completed_count) * 100).toFixed(1) : "0.0"}% 可用率
            </p>
          </div>
          <div className="rounded-xl bg-cyber-bg-elevated/60 border border-cyber-border/20 p-4 space-y-1">
            <p className="text-xs text-cyber-muted">错误数</p>
            <p className="text-xl font-bold text-cyber-red tabular-nums">{summary.error_count}</p>
            <p className="text-[10px] text-cyber-muted-dim">
              {summary.completed_count > 0 ? ((summary.error_count / summary.completed_count) * 100).toFixed(2) : "0.00"}% 错误率
            </p>
          </div>
        </div>
      </div>

      {/* Results Table */}
      <div className="glass-panel overflow-hidden">
        <div className="px-5 py-3.5 border-b border-cyber-border/30 flex items-center justify-between">
          <h2 className="section-title m-0">
            <Table className="w-4 h-4 text-cyber-green" /> 扫描结果
          </h2>
          <div className="text-xs text-cyber-muted-dim">
            已加载 {results.length} 条
          </div>
        </div>
        {resultsError ? (
          <div className="px-5 py-6 text-sm text-cyber-red">{resultsError}</div>
        ) : resultsLoading && results.length === 0 ? (
          <div className="text-center py-12 text-cyber-muted">
            <Table className="w-10 h-10 mx-auto mb-3 opacity-40" />
            <p className="text-sm">正在加载扫描结果...</p>
          </div>
        ) : results.length === 0 ? (
          <div className="text-center py-12 text-cyber-muted">
            <Table className="w-10 h-10 mx-auto mb-3 opacity-40" />
            <p className="text-sm">暂无扫描结果</p>
            <p className="text-xs text-cyber-muted-dim mt-1">任务运行后结果将在此显示</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-cyber-bg-elevated/70 text-cyber-muted-dim">
                <tr className="text-left">
                  <th className="px-5 py-3 font-medium">域名</th>
                  <th className="px-3 py-3 font-medium">状态</th>
                  <th className="px-3 py-3 font-medium">方式</th>
                  <th className="px-3 py-3 font-medium">耗时</th>
                  <th className="px-3 py-3 font-medium">时间</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-cyber-border/15">
                {results.map((item) => {
                  const status = getResultStatus(item);
                  return (
                    <tr key={item.id} className="hover:bg-cyber-bg-elevated/30">
                      <td className="px-5 py-3">
                        <div className="font-mono text-cyber-text">{item.domain}</div>
                        {item.error_message && (
                          <div className="mt-1 text-xs text-cyber-red max-w-[520px] truncate" title={item.error_message}>
                            {item.error_message}
                          </div>
                        )}
                      </td>
                      <td className="px-3 py-3">
                        <span className={`${status.className} text-[11px]`}>{status.label}</span>
                      </td>
                      <td className="px-3 py-3 text-cyber-text-secondary uppercase">
                        {item.query_method ?? "-"}
                      </td>
                      <td className="px-3 py-3 text-cyber-text-secondary tabular-nums">
                        {item.response_time_ms !== null ? `${item.response_time_ms} ms` : "-"}
                      </td>
                      <td className="px-3 py-3 text-cyber-muted-dim text-xs">
                        {item.checked_at ? formatDateTime(item.checked_at) : "-"}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Log Panel */}
      <div className="glass-panel overflow-hidden">
        <div
          className="px-5 py-3.5 border-b border-cyber-border/30 flex items-center justify-between cursor-pointer hover:bg-cyber-card/20 transition-colors"
          onClick={() => setShowLogs(!showLogs)}
        >
          <h2 className="section-title m-0">
            <Terminal className="w-4 h-4 text-cyber-green" />
            实时日志
          </h2>
          <div className="flex items-center gap-3">
            <div className="hidden sm:flex bg-cyber-surface rounded-lg p-0.5 text-xs">
              {(["all", "info", "warn", "error"] as const).map((lvl) => (
                <button
                  key={lvl}
                  onClick={(e) => { e.stopPropagation(); setLogFilter(lvl); }}
                  className={`px-2.5 py-1 rounded-md transition-all ${
                    logFilter === lvl ? "bg-cyber-green/15 text-cyber-green" : "text-cyber-muted-dim hover:text-cyber-text-secondary"
                  }`}
                >
                  {lvl === "all" ? "全部" : lvl.toUpperCase()}
                </button>
              ))}
            </div>
            <ChevronDown className={`w-4 h-4 text-cyber-muted transition-transform duration-200 ${showLogs ? "" : "-rotate-90"}`} />
          </div>
        </div>
        {showLogs && (
          <div className="bg-cyber-bg-elevated/80 max-h-52 overflow-y-auto font-mono text-xs leading-relaxed">
            {logsError ? (
              <div className="px-5 py-4 text-sm text-cyber-red">{logsError}</div>
            ) : logsLoading && logs.length === 0 ? (
              <div className="text-center py-8 text-cyber-muted">
                <Terminal className="w-8 h-8 mx-auto mb-2 opacity-30" />
                <p className="text-xs">正在加载日志...</p>
              </div>
            ) : logs.length === 0 ? (
              <div className="text-center py-8 text-cyber-muted">
                <Terminal className="w-8 h-8 mx-auto mb-2 opacity-30" />
                <p className="text-xs">暂无运行日志</p>
              </div>
            ) : (
              <div className="divide-y divide-cyber-border/10">
                {logs.map((log) => (
                  <div key={log.id} className="px-5 py-3">
                    <div className="flex items-start gap-3">
                      <span className={`uppercase text-[10px] font-bold ${logColors[log.level] ?? "text-cyber-muted"}`}>
                        {log.level}
                      </span>
                      <span className="text-cyber-text-secondary text-[10px] shrink-0">
                        {formatDateTime(log.created_at)}
                      </span>
                      <span className="text-cyber-text whitespace-pre-wrap break-all">{log.message}</span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      {availableResults.length > 0 && (
        <div className="text-xs text-cyber-muted-dim px-1">
          当前已发现 {availableResults.length} 个可用域名，结果表默认展示最近加载到的 200 条记录。
        </div>
      )}
    </div>
  );
}

function formatDateTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

function getResultStatus(item: ScanItem) {
  if (item.error_message && item.status !== "error") {
    return { label: "降级", className: "badge-red" };
  }
  return resultStatusConfig[item.status] ?? resultStatusConfig.pending;
}
