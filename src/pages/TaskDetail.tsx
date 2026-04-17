import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import {
  ArrowLeft,
  Pause,
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
  Gauge,
  Save,
  Settings,
  Shield,
  Server,
} from "lucide-react";
import { useNavigate, useParams } from "react-router-dom";
import { useTaskStore } from "../store/taskStore";
import { useProxyStore } from "../store/proxyStore";
import { invokeCommand, listenEvent } from "../services/tauri";
import type { ListScanBatchesResponse, LogEntry, PaginatedResult, ScanBatchSummary, ScanItem, TaskRun } from "../types";
import ActionNotice, { type ActionNoticeState } from "../components/ActionNotice";
import { ProxySelect } from "../components/ProxySelect";

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
  run_id: string;
  completed_count: number;
  total_count: number;
  available_count: number;
  error_count: number;
  percent: number;
}

interface ScanResultsUpdated {
  task_id: string;
  run_id: string;
  flushed_count: number;
  completed_count: number;
}

type LogType = "task" | "request";

export default function TaskDetail() {
  const { id } = useParams();
  const navigate = useNavigate();
  const [showLogs, setShowLogs] = useState(true);
  const [logType, setLogType] = useState<LogType>("task");
  const [logFilter, setLogFilter] = useState<string>("all");
  const [resultFilter, setResultFilter] = useState<"all" | "available" | "unavailable" | "error">("available");
  const [actionLoading, setActionLoading] = useState<"start" | "pause" | "stop" | "delete" | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [actionNotice, setActionNotice] = useState<ActionNoticeState | null>(null);
  const [runs, setRuns] = useState<TaskRun[]>([]);
  const [runsLoading, setRunsLoading] = useState(false);
  const [runsError, setRunsError] = useState<string | null>(null);
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const [resultPage, setResultPage] = useState(1);
  const [resultTotal, setResultTotal] = useState(0);
  const [results, setResults] = useState<ScanItem[]>([]);
  const [resultsLoading, setResultsLoading] = useState(false);
  const [resultsError, setResultsError] = useState<string | null>(null);
  const [selectedResultIds, setSelectedResultIds] = useState<number[]>([]);
  const [retryingResults, setRetryingResults] = useState(false);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [batchSummary, setBatchSummary] = useState<ScanBatchSummary | null>(null);
  const [batchesLoading, setBatchesLoading] = useState(false);
  const [batchesExpanded, setBatchesExpanded] = useState(false);
  const [logsLoading, setLogsLoading] = useState(false);
  const [logsError, setLogsError] = useState<string | null>(null);
  const [settingsConcurrency, setSettingsConcurrency] = useState(50);
  const [settingsProxyId, setSettingsProxyId] = useState<number | undefined>(undefined);
  const [settingsSaving, setSettingsSaving] = useState(false);
  const [settingsError, setSettingsError] = useState<string | null>(null);
  const [liveProgress, setLiveProgress] = useState<ScanProgress | null>(null);
  const [tldsExpanded, setTldsExpanded] = useState(false);
  const resultRefreshTimerRef = useRef<number | null>(null);
  const { tasks, fetchTasks, startTask, pauseTask, stopTask, updateTaskSettings, rerunTask, deleteTask } = useTaskStore();
  const { proxies, fetchProxies } = useProxyStore();
  const effectiveRunId = selectedRunId ?? runs[0]?.id ?? null;

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
      const nextRunId = parsed[0]?.id ?? null;
      setSelectedRunId((current) => {
        if (current && parsed.some((run) => run.id === current)) return current;
        return nextRunId;
      });
    } catch (e) {
      setRunsError(String(e));
    } finally {
      setRunsLoading(false);
    }
  }, [id]);

  const fetchResults = useCallback(async () => {
    if (!id || !effectiveRunId) {
      setResults([]);
      setResultTotal(0);
      return;
    }
    setResultsLoading(true);
    setResultsError(null);
    try {
      const result = await invokeCommand<string>("list_scan_items", {
        request: {
          task_id: id,
          run_id: effectiveRunId,
          status: resultFilter === "all" ? null : resultFilter,
          limit: 10,
          offset: (resultPage - 1) * 10,
        },
      });
      const payload = JSON.parse(result) as PaginatedResult<ScanItem>;
      setResults(payload.items);
      setResultTotal(payload.total);
    } catch (e) {
      setResultsError(String(e));
    } finally {
      setResultsLoading(false);
    }
  }, [id, effectiveRunId, resultFilter, resultPage]);

  const fetchLogs = useCallback(async () => {
    if (!id || (logType === "request" && !effectiveRunId)) {
      setLogs([]);
      return;
    }
    setLogsLoading(true);
    setLogsError(null);
    try {
      const result = await invokeCommand<string>("get_logs", {
        request: {
          task_id: id,
          run_id: effectiveRunId,
          log_type: logType,
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
  }, [id, effectiveRunId, logFilter, logType]);

  const fetchBatchSummary = useCallback(async () => {
    if (!id || !effectiveRunId) {
      setBatchSummary(null);
      return;
    }
    setBatchesLoading(true);
    try {
      const result = await invokeCommand<string>("list_scan_batches", {
        request: {
          task_id: id,
          run_id: effectiveRunId,
          limit: 0,
          offset: 0,
        },
      });
      const payload = JSON.parse(result) as ListScanBatchesResponse;
      setBatchSummary(payload.summary);
    } catch {
      setBatchSummary(null);
    } finally {
      setBatchesLoading(false);
    }
  }, [id, effectiveRunId]);

  useEffect(() => {
    if (tasks.length === 0) fetchTasks();
  }, [tasks.length, fetchTasks]);

  useEffect(() => {
    fetchProxies();
  }, [fetchProxies]);

  useEffect(() => {
    fetchRuns();
  }, [fetchRuns]);

  useEffect(() => {
    setResultPage(1);
    setLiveProgress(null);
    setSelectedResultIds([]);
  }, [selectedRunId]);

  useEffect(() => {
    setResultPage(1);
    setSelectedResultIds([]);
  }, [resultFilter]);

  useEffect(() => {
    fetchResults();
  }, [fetchResults]);

  useEffect(() => {
    fetchLogs();
  }, [fetchLogs]);

  useEffect(() => {
    fetchBatchSummary();
  }, [fetchBatchSummary]);

  // Listen for scan progress events
  useEffect(() => {
    const unlisten = listenEvent<ScanProgress>("scan-progress", (progress) => {
      if (progress.task_id !== id) return;
      setLiveProgress(progress);
      setRuns((current) =>
        current.map((run) =>
          run.id === progress.run_id
            ? {
                ...run,
                completed_count: progress.completed_count,
                total_count: progress.total_count,
                available_count: progress.available_count,
                error_count: progress.error_count,
              }
            : run
        )
      );
    });
    const unlistenResults = listenEvent<ScanResultsUpdated>("scan-results-updated", (payload) => {
      if (payload.task_id !== id) return;
      if (payload.run_id !== effectiveRunId || resultPage !== 1) return;
      setResultTotal((current) => Math.max(current, payload.completed_count));
      if (resultRefreshTimerRef.current) {
        window.clearTimeout(resultRefreshTimerRef.current);
      }
      resultRefreshTimerRef.current = window.setTimeout(() => {
        fetchResults();
      }, 100);
    });
    const unlistenComplete = listenEvent<ScanProgress>("scan-complete", (progress) => {
      if (progress.task_id === id) {
        fetchTasks();
        fetchRuns();
        fetchResults();
        fetchLogs();
        fetchBatchSummary();
      }
    });
    return () => {
      unlisten.then(fn => fn());
      unlistenResults.then(fn => fn());
      unlistenComplete.then(fn => fn());
      if (resultRefreshTimerRef.current) {
        window.clearTimeout(resultRefreshTimerRef.current);
      }
    };
  }, [id, effectiveRunId, resultPage, fetchTasks, fetchRuns, fetchResults, fetchLogs, fetchBatchSummary]);

  useEffect(() => {
    const unlisten = listenEvent<LogEntry>("task-log-created", (log) => {
      if (log.task_id !== id) return;
      if (log.log_type !== logType) return;
      if (logType === "request" && effectiveRunId && log.run_id !== effectiveRunId) return;
      if (logType === "task" && log.run_id && effectiveRunId && log.run_id !== effectiveRunId) return;
      if (logFilter !== "all" && log.level !== logFilter) return;
      setLogs((current) => [log, ...current.filter((entry) => entry.id !== log.id)].slice(0, 200));
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [id, effectiveRunId, logFilter, logType]);

  // Auto-refresh while task is running (fallback for missed events)
  useEffect(() => {
    const task = tasks.find((t) => t.id === id);
    if (!task || task.status !== "running") return;
    const interval = setInterval(() => {
      fetchRuns();
      fetchBatchSummary();
      if (resultPage === 1) {
        fetchResults();
      }
      if (showLogs) {
        fetchLogs();
      }
    }, 5000);
    return () => clearInterval(interval);
  }, [tasks, id, resultPage, showLogs, fetchRuns, fetchResults, fetchLogs, fetchBatchSummary]);

  const task = tasks.find((t) => t.id === id);
  const selectedRun = useMemo(
    () => runs.find((run) => run.id === selectedRunId) ?? runs[0] ?? null,
    [runs, selectedRunId]
  );
  const allCurrentPageSelected = useMemo(
    () => results.length > 0 && results.every((item) => selectedResultIds.includes(item.id)),
    [results, selectedResultIds]
  );
  const visibleTlds = useMemo(() => {
    if (!task) return [];
    return tldsExpanded || task.tlds.length <= 10 ? task.tlds : task.tlds.slice(0, 10);
  }, [task, tldsExpanded]);
  const hasHiddenTlds = (task?.tlds.length ?? 0) > 10;

  useEffect(() => {
    setSelectedResultIds((current) => current.filter((id) => results.some((item) => item.id === id)));
  }, [results]);

  useEffect(() => {
    setTldsExpanded(false);
  }, [task?.id]);

  useEffect(() => {
    if (!task) return;
    setSettingsConcurrency(task.concurrency);
    setSettingsProxyId(task.proxy_id ?? undefined);
    setSettingsError(null);
  }, [task?.id, task?.concurrency, task?.proxy_id]);

  const canEditSettings = task?.status === "pending" || task?.status === "paused";
  const settingsDirty = Boolean(
    task &&
      canEditSettings &&
      (settingsConcurrency !== task.concurrency || (settingsProxyId ?? null) !== task.proxy_id)
  );

  const handleStart = async () => {
    if (!task) return;
    setActionError(null);
    setActionLoading("start");
    try {
      await startTask(task.id);
      await fetchTasks();
      await fetchRuns();
      await fetchBatchSummary();
    } catch (e) {
      setActionError(String(e));
    } finally {
      setActionLoading(null);
    }
  };

  const handleSaveSettings = async () => {
    if (!task || !canEditSettings) return;
    setSettingsSaving(true);
    setSettingsError(null);
    try {
      await updateTaskSettings(task.id, settingsConcurrency, settingsProxyId ?? null);
      await fetchTasks();
      await fetchLogs();
      setActionNotice({
        tone: "info",
        title: "任务设置已保存",
        message: `并发量 ${settingsConcurrency}，代理 ${settingsProxyId ? "已选择" : "直连"}`,
      });
    } catch (e) {
      setSettingsError(String(e));
    } finally {
      setSettingsSaving(false);
    }
  };

  const handlePause = async () => {
    if (!task) return;
    setActionError(null);
    setActionLoading("pause");
    try {
      await pauseTask(task.id);
      await fetchTasks();
      await fetchRuns();
      await fetchResults();
      await fetchLogs();
      await fetchBatchSummary();
    } catch (e) {
      setActionError(String(e));
    } finally {
      setActionLoading(null);
    }
  };

  const handleStop = async () => {
    if (!task) return;
    const confirmed = window.confirm(`确定停止任务“${task.name}”吗？停止后不能断点续传，只能重新开始。`);
    if (!confirmed) return;

    setActionError(null);
    setActionLoading("stop");
    try {
      await stopTask(task.id);
      await fetchTasks();
      await fetchRuns();
      await fetchResults();
      await fetchLogs();
      await fetchBatchSummary();
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
      await fetchBatchSummary();
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

  const handleRetrySelected = async () => {
    if (!id || !effectiveRunId || selectedResultIds.length === 0) return;
    const confirmed = window.confirm(`确定重试已勾选的 ${selectedResultIds.length} 条结果吗？`);
    if (!confirmed) return;

    setRetryingResults(true);
    setResultsError(null);
    try {
      await invokeCommand<string>("retry_scan_items", {
        request: {
          task_id: id,
          run_id: effectiveRunId,
          item_ids: selectedResultIds,
        },
      });
      setSelectedResultIds([]);
      await Promise.all([fetchResults(), fetchLogs(), fetchRuns(), fetchTasks(), fetchBatchSummary()]);
    } catch (e) {
      setResultsError(String(e));
    } finally {
      setRetryingResults(false);
    }
  };

  const showActionNotice = (notice: ActionNoticeState, action: string) => {
    console.info(`[ui-action] task-detail-${action}`, notice);
    setActionNotice(notice);
  };

  if (!task) {
    return (
      <div className="page-shell max-w-5xl">
        <div className="flex items-center gap-4">
          <button onClick={() => navigate("/tasks")} className="cyber-btn-icon cyber-btn-ghost">
            <ArrowLeft className="w-4.5 h-4.5" />
          </button>
          <h1 className="text-xl font-normal text-cyber-text">任务未找到</h1>
        </div>
        <div className="glass-panel p-12 text-center text-cyber-muted">
          <Inbox className="w-10 h-10 mx-auto mb-3 opacity-40" />
          <p className="text-sm">未找到 ID 为 {id} 的任务</p>
        </div>
      </div>
    );
  }

  const summary =
    liveProgress && liveProgress.run_id === effectiveRunId
      ? liveProgress
      : selectedRun ?? {
    total_count: task.total_count,
    completed_count: task.completed_count,
    available_count: task.available_count,
    error_count: task.error_count,
  };
  const progress = summary.total_count > 0 ? Math.round((summary.completed_count / summary.total_count) * 100) : 0;
  const totalPages = Math.max(1, Math.ceil(resultTotal / 10));
  const resultFilterOptions: Array<{ key: "all" | "available" | "unavailable" | "error"; label: string }> = [
    { key: "all", label: "全部" },
    { key: "available", label: "可用" },
    { key: "unavailable", label: "已注册" },
    { key: "error", label: "错误" },
  ];

  return (
    <div className="page-shell max-w-5xl">
      <div className="flex items-start gap-4">
        <button
          onClick={() => navigate("/tasks")}
          className="cyber-btn-icon cyber-btn-ghost"
        >
          <ArrowLeft className="w-4.5 h-4.5" />
        </button>
        <div className="min-w-0 flex-1">
          <div className="eyebrow mb-2">TASK DETAIL</div>
          <h1 className="truncate text-3xl font-normal leading-none text-cyber-text" title={task.name}>
            {task.name}
          </h1>
          <div className="mt-2 text-xs text-cyber-muted-dim font-mono">ID: {id}</div>
          <div
            className={`mt-2 flex items-start gap-2 overflow-hidden transition-[height] duration-200 ${
              tldsExpanded ? "min-h-[4.25rem] flex-wrap" : "h-[4.25rem] flex-wrap"
            }`}
          >
            {visibleTlds.map((tld) => (
              <span key={tld} className="badge-neutral shrink-0">{tld}</span>
            ))}
            {hasHiddenTlds && (
              <button
                type="button"
                className="badge-neutral inline-flex min-w-[6.5rem] shrink-0 items-center justify-center gap-1 hover:border-cyber-green/35 hover:text-cyber-text-secondary"
                onClick={() => setTldsExpanded((expanded) => !expanded)}
                aria-expanded={tldsExpanded}
              >
                {tldsExpanded ? "收起" : `展开 +${task.tlds.length - 10}`}
                <ChevronDown className={`h-3 w-3 transition-transform ${tldsExpanded ? "rotate-180" : ""}`} />
              </button>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2 flex-wrap justify-end">
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
              onClick={handlePause}
              disabled={actionLoading !== null}
            >
              <Pause className="w-3.5 h-3.5" /> {actionLoading === "pause" ? "暂停中..." : "暂停任务"}
            </button>
          )}
          {(task.status === "running" || task.status === "paused") && (
            <button
              className="cyber-btn-secondary cyber-btn-sm text-cyber-red border-cyber-red/25 hover:border-cyber-red/45 hover:text-cyber-red"
              onClick={handleStop}
              disabled={actionLoading !== null}
            >
              <Square className="w-3.5 h-3.5" /> {actionLoading === "stop" ? "停止中..." : "停止任务"}
            </button>
          )}
          {task.status !== "running" && runs.length > 0 && (
            <button
              className="cyber-btn-secondary cyber-btn-sm"
              onClick={handleRerun}
              disabled={actionLoading !== null}
            >
              <RotateCcw className="w-3.5 h-3.5" /> {actionLoading === "start" ? "重新开始中..." : "重新开始"}
            </button>
          )}
          <button
            className="cyber-btn-secondary cyber-btn-sm text-cyber-red border-cyber-red/25 hover:border-cyber-red/45 hover:text-cyber-red"
            onClick={handleDelete}
            disabled={actionLoading !== null}
          >
            <Trash2 className="w-3.5 h-3.5" /> {actionLoading === "delete" ? "删除中..." : "删除"}
          </button>
          <button
            className="cyber-btn-secondary cyber-btn-sm"
            onClick={() =>
              showActionNotice(
                {
                  tone: "warning",
                  title: "导出需要选择输出路径",
                  message: "后端 export_results 已存在，但当前前端还没有接入文件保存对话框，因此没有直接写文件。请先使用结果表查看和重试结果。",
                },
                "export"
              )
            }
          >
            <Download className="w-3.5 h-3.5" /> 导出
          </button>
          <button
            className="cyber-btn-secondary cyber-btn-sm"
            onClick={() => {
              console.info("[ui-action] task-detail-vectorize", { task_id: task.id });
              navigate("/vectorize");
            }}
          >
            <Cpu className="w-3.5 h-3.5" /> 向量化
          </button>
          <button
            className="cyber-btn-secondary cyber-btn-sm"
            onClick={() => {
              console.info("[ui-action] task-detail-filter", { task_id: task.id });
              navigate("/filter");
            }}
          >
            <Filter className="w-3.5 h-3.5" /> 筛选
          </button>
        </div>
      </div>

      {actionError && (
        <div className="glass-panel px-4 py-3 text-sm text-cyber-red">
          {actionError}
        </div>
      )}
      {actionNotice && <ActionNotice notice={actionNotice} onClose={() => setActionNotice(null)} />}

      {canEditSettings && (
        <div className="glass-panel p-4 space-y-4">
          <div className="flex items-center justify-between gap-3">
            <h2 className="section-title m-0">
              <Settings className="w-4 h-4 text-cyber-text-secondary" />
              任务设置
            </h2>
            <button
              className="cyber-btn-primary cyber-btn-sm"
              disabled={!settingsDirty || settingsSaving}
              onClick={handleSaveSettings}
            >
              <Save className="w-3.5 h-3.5" />
              {settingsSaving ? "保存中..." : "保存设置"}
            </button>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
            <div>
              <label className="flex items-center gap-2 text-xs text-cyber-muted mb-2">
                <Gauge className="w-3.5 h-3.5" />
                并发量
                <span className="text-cyber-green font-mono font-bold">{settingsConcurrency}</span>
              </label>
              <div className="flex items-center gap-3">
                <input
                  type="range"
                  min={1}
                  max={500}
                  value={settingsConcurrency}
                  onChange={(event) => setSettingsConcurrency(Number(event.target.value))}
                  className="w-full accent-cyber-green"
                />
                <input
                  type="number"
                  min={1}
                  max={500}
                  value={settingsConcurrency}
                  onChange={(event) =>
                    setSettingsConcurrency(Math.min(500, Math.max(1, Number(event.target.value) || 1)))
                  }
                  className="cyber-input h-9 w-24 text-sm"
                />
              </div>
            </div>
            <div>
              <label className="flex items-center gap-2 text-xs text-cyber-muted mb-2">
                <Shield className="w-3.5 h-3.5" />
                代理
              </label>
              <ProxySelect
                proxies={proxies}
                selectedProxyId={settingsProxyId}
                onChange={setSettingsProxyId}
                hasWarning={Boolean(
                  settingsProxyId &&
                    proxies.find((proxy) => proxy.id === settingsProxyId)?.status !== "available"
                )}
              />
            </div>
          </div>
          {settingsError && <div className="text-sm text-cyber-red">{settingsError}</div>}
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

      <div className="grid grid-cols-[200px_1fr] gap-4 glass-panel p-5 items-center">
        <div className="relative w-32 h-32 mx-auto shrink-0">
          <svg className="w-full h-full -rotate-90" viewBox="0 0 128 128">
            <circle cx="64" cy="64" r="56" fill="none" stroke="#252D3A" strokeWidth="8" />
            <circle
              cx="64" cy="64" r="56" fill="none"
              stroke="#ffffff" strokeWidth="8"
              strokeLinecap="round"
              strokeDasharray={`${(progress / 100) * 351.86} 351.86`}
              style={{ transition: "stroke-dasharray 700ms ease-out" }}
            />
          </svg>
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            <span className="text-2xl font-normal text-white tabular-nums">{progress}%</span>
            <span className="text-[10px] text-cyber-muted">进度</span>
          </div>
        </div>

        <div className="grid grid-cols-3 gap-4">
          <div className="metric-tile space-y-1">
            <p className="text-xs text-cyber-muted">已完成</p>
            <p className="text-xl font-normal text-cyber-text tabular-nums">{summary.completed_count.toLocaleString()}</p>
            <p className="text-[10px] text-cyber-muted-dim font-mono tabular-nums">/ {summary.total_count.toLocaleString()}</p>
          </div>
          <div className="metric-tile space-y-1">
            <p className="text-xs text-cyber-muted">可用域名</p>
            <p className="text-xl font-normal text-cyber-green tabular-nums">{summary.available_count.toLocaleString()}</p>
            <p className="text-[10px] text-cyber-muted-dim">
              {summary.completed_count > 0 ? ((summary.available_count / summary.completed_count) * 100).toFixed(1) : "0.0"}% 可用率
            </p>
          </div>
          <div className="metric-tile space-y-1">
            <p className="text-xs text-cyber-muted">错误数</p>
            <p className="text-xl font-normal text-cyber-red tabular-nums">{summary.error_count}</p>
            <p className="text-[10px] text-cyber-muted-dim">
              {summary.completed_count > 0 ? ((summary.error_count / summary.completed_count) * 100).toFixed(2) : "0.00"}% 错误率
            </p>
          </div>
        </div>
      </div>

      {batchSummary && batchSummary.total > 0 && (
        <div className="glass-panel overflow-hidden">
          <button
            type="button"
            className="w-full px-5 py-3.5 border-b border-cyber-border/30 flex items-center justify-between text-left hover:bg-cyber-card/20 transition-colors"
            onClick={() => setBatchesExpanded((expanded) => !expanded)}
            aria-expanded={batchesExpanded}
          >
            <h2 className="section-title m-0">
              <Server className="w-4 h-4 text-cyber-blue" />
              Batch 调度摘要
            </h2>
            <div className="flex items-center gap-3 text-xs text-cyber-muted-dim">
              <span>{batchesLoading ? "刷新中..." : `${batchSummary.total} 个 batch`}</span>
              <ChevronDown className={`w-4 h-4 transition-transform ${batchesExpanded ? "rotate-180" : ""}`} />
            </div>
          </button>
          <div className="grid grid-cols-4 gap-3 p-4">
            <div className="metric-tile space-y-1">
              <p className="text-xs text-cyber-muted">运行中</p>
              <p className="text-xl font-normal text-cyber-blue tabular-nums">
                {(batchSummary.running + batchSummary.assigned + batchSummary.retrying).toLocaleString()}
              </p>
              <p className="text-[10px] text-cyber-muted-dim">queued {batchSummary.queued}</p>
            </div>
            <div className="metric-tile space-y-1">
              <p className="text-xs text-cyber-muted">已完成 batch</p>
              <p className="text-xl font-normal text-cyber-green tabular-nums">{batchSummary.succeeded.toLocaleString()}</p>
              <p className="text-[10px] text-cyber-muted-dim">/ {batchSummary.total.toLocaleString()}</p>
            </div>
            <div className="metric-tile space-y-1">
              <p className="text-xs text-cyber-muted">异常 batch</p>
              <p className="text-xl font-normal text-cyber-red tabular-nums">
                {(batchSummary.failed + batchSummary.expired + batchSummary.cancelled).toLocaleString()}
              </p>
              <p className="text-[10px] text-cyber-muted-dim">paused {batchSummary.paused}</p>
            </div>
            <div className="metric-tile space-y-1">
              <p className="text-xs text-cyber-muted">参与 worker</p>
              <p className="text-xl font-normal text-cyber-text tabular-nums">{batchSummary.worker_count}</p>
              <p className="text-[10px] text-cyber-muted-dim">结果 {batchSummary.completed_count.toLocaleString()}</p>
            </div>
          </div>
          {batchesExpanded && (
            <div className="border-t border-cyber-border/20 px-5 py-3 text-xs text-cyber-muted-dim grid grid-cols-4 gap-3">
              <span>可用：{batchSummary.available_count.toLocaleString()}</span>
              <span>错误：{batchSummary.error_count.toLocaleString()}</span>
              <span>重试：{batchSummary.retrying.toLocaleString()}</span>
              <span>过期：{batchSummary.expired.toLocaleString()}</span>
            </div>
          )}
        </div>
      )}

      <div className="glass-panel overflow-hidden">
        <div className="px-5 py-3.5 border-b border-cyber-border/30 flex items-center justify-between">
          <h2 className="section-title m-0">
            <Table className="w-4 h-4 text-cyber-green" /> 扫描结果
          </h2>
          <div className="flex items-center gap-3">
            <div className="flex bg-cyber-surface rounded-md p-0.5 text-xs">
              {resultFilterOptions.map((option) => (
                <button
                  key={option.key}
                  onClick={() => setResultFilter(option.key)}
                  className={`px-2.5 py-1 rounded-md transition-all ${
                    resultFilter === option.key
                      ? "bg-white/[0.08] text-white"
                      : "text-cyber-muted-dim hover:text-cyber-text-secondary"
                  }`}
                >
                  {option.label}
                </button>
              ))}
            </div>
            <button
              className="cyber-btn-secondary cyber-btn-sm"
              disabled={selectedResultIds.length === 0 || retryingResults}
              onClick={handleRetrySelected}
            >
              <RotateCcw className={`w-3.5 h-3.5 ${retryingResults ? "animate-spin" : ""}`} />
              {retryingResults ? "重试中..." : `重试选中${selectedResultIds.length > 0 ? ` (${selectedResultIds.length})` : ""}`}
            </button>
            <div className="grid w-[30ch] shrink-0 grid-cols-[16ch_12ch] gap-2 text-right text-xs tabular-nums text-cyber-muted-dim">
              <span className="whitespace-nowrap">第 {resultPage}/{totalPages} 页</span>
              <span className="whitespace-nowrap">共 {resultTotal} 条</span>
            </div>
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
                  <th className="px-3 py-3 font-medium w-10">
                    <input
                      type="checkbox"
                      checked={allCurrentPageSelected}
                      onChange={(e) => {
                        if (e.target.checked) {
                          setSelectedResultIds(results.map((item) => item.id));
                        } else {
                          setSelectedResultIds([]);
                        }
                      }}
                    />
                  </th>
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
                      <td className="px-3 py-3">
                        <input
                          type="checkbox"
                          checked={selectedResultIds.includes(item.id)}
                          onChange={(e) => {
                            setSelectedResultIds((current) =>
                              e.target.checked
                                ? [...current, item.id]
                                : current.filter((id) => id !== item.id)
                            );
                          }}
                        />
                      </td>
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
        {resultTotal > 0 && (
          <div className="px-5 py-3 border-t border-cyber-border/20 flex flex-col gap-3 text-xs text-cyber-muted-dim sm:flex-row sm:items-center sm:justify-between">
            <span>默认显示可用结果；支持全部 / 可用 / 已注册 / 错误筛选，每页 10 条</span>
            <div className="grid w-[22rem] shrink-0 grid-cols-[5.5rem_10rem_5.5rem] items-center gap-2">
              <button
                className="cyber-btn-secondary cyber-btn-sm w-full justify-center"
                disabled={resultPage <= 1}
                onClick={() => setResultPage((page) => Math.max(1, page - 1))}
              >
                上一页
              </button>
              <span
                className="inline-flex h-8 items-center justify-center rounded-md border border-cyber-border/30 bg-cyber-bg-elevated/40 px-2 font-mono tabular-nums text-cyber-text-secondary"
                aria-label={`第 ${resultPage} 页，共 ${totalPages} 页`}
              >
                <span className="inline-block w-[5ch] text-right">{resultPage}</span>
                <span className="px-1 text-cyber-muted-dim">/</span>
                <span className="inline-block w-[5ch] text-left">{totalPages}</span>
              </span>
              <button
                className="cyber-btn-secondary cyber-btn-sm w-full justify-center"
                disabled={resultPage >= totalPages}
                onClick={() => setResultPage((page) => Math.min(totalPages, page + 1))}
              >
                下一页
              </button>
            </div>
          </div>
        )}
      </div>

      <div className="glass-panel overflow-hidden">
        <div
          className="px-5 py-3.5 border-b border-cyber-border/30 flex items-center justify-between cursor-pointer hover:bg-cyber-card/20 transition-colors"
          onClick={() => setShowLogs(!showLogs)}
        >
          <h2 className="section-title m-0">
            <Terminal className="w-4 h-4 text-cyber-green" />
            日志
          </h2>
          <div className="flex items-center gap-3">
            <div className="hidden sm:flex bg-cyber-surface rounded-md p-0.5 text-xs">
              {([
                ["task", "任务日志"],
                ["request", "请求日志"],
              ] as const).map(([type, label]) => (
                <button
                  key={type}
                  onClick={(e) => {
                    e.stopPropagation();
                    setLogType(type);
                  }}
                  className={`px-2.5 py-1 rounded-md transition-all ${
                    logType === type
                      ? "bg-white/[0.08] text-white"
                      : "text-cyber-muted-dim hover:text-cyber-text-secondary"
                  }`}
                >
                  {label}
                </button>
              ))}
            </div>
            <div className="hidden sm:flex bg-cyber-surface rounded-md p-0.5 text-xs">
              {(["all", "info", "warn", "error"] as const).map((lvl) => (
                <button
                  key={lvl}
                  onClick={(e) => { e.stopPropagation(); setLogFilter(lvl); }}
                  className={`px-2.5 py-1 rounded-md transition-all ${
                    logFilter === lvl ? "bg-white/[0.08] text-white" : "text-cyber-muted-dim hover:text-cyber-text-secondary"
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
                <p className="text-xs">暂无{logType === "task" ? "任务日志" : "请求日志"}</p>
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

      {summary.available_count > 0 && (
        <div className="text-xs text-cyber-muted-dim px-1">
          当前运行累计已确认 {summary.available_count} 个可用域名；勾选多条结果后可发起重试并刷新结果。
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
