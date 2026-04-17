import { useState, useEffect, useMemo } from "react";
import {
  Cpu,
  Play,
  HardDrive,
  Gauge,
  Clock,
  Inbox,
  Terminal,
  Database,
  RefreshCcw,
  Trash2,
  XCircle,
} from "lucide-react";
import { useGpuStore } from "../store/gpuStore";
import { useTaskStore } from "../store/taskStore";
import { invokeCommand } from "../services/tauri";
import ActionNotice, { type ActionNoticeState } from "../components/ActionNotice";
import { useTaskLogs } from "../hooks/useTaskLogs";
import { useVectorProgress } from "../hooks/useVectorProgress";
import type { TaskRun, VectorRecord, VectorStats } from "../types";

type VectorLogType = "task" | "request";

const logColors: Record<string, string> = {
  debug: "text-cyber-muted",
  info: "text-cyber-cyan",
  warn: "text-cyber-orange",
  error: "text-cyber-red",
};

const VECTOR_PAGE_SIZE = 50;

export default function VectorizePage() {
  const [selectedTaskIds, setSelectedTaskIds] = useState<string[]>([]);
  const [activeTaskId, setActiveTaskId] = useState("");
  const [latestRunsByTask, setLatestRunsByTask] = useState<Record<string, TaskRun | null>>({});
  const [isRunning, setIsRunning] = useState(false);
  const [actionLoading, setActionLoading] = useState(false);
  const [stopLoading, setStopLoading] = useState(false);
  const [vectorLoading, setVectorLoading] = useState(false);
  const [vectorActionId, setVectorActionId] = useState<number | "all" | null>(null);
  const [logType, setLogType] = useState<VectorLogType>("task");
  const [vectorStats, setVectorStats] = useState<VectorStats | null>(null);
  const [vectors, setVectors] = useState<VectorRecord[]>([]);
  const [vectorTotal, setVectorTotal] = useState(0);
  const [vectorPage, setVectorPage] = useState(0);
  const [notice, setNotice] = useState<ActionNoticeState | null>(null);
  const { status: gpuStatus, fetchStatus: fetchGpuStatus } = useGpuStore();
  const { tasks, fetchTasks } = useTaskStore();

  useEffect(() => {
    fetchGpuStatus();
    fetchTasks();
  }, []);

  const completedTasks = useMemo(
    () => tasks.filter(t => t.status === "completed" && t.available_count > 0),
    [tasks]
  );
  const activeTaskInfo = completedTasks.find((task) => task.id === activeTaskId);
  const { progress, fetchProgress } = useVectorProgress(activeTaskId || null);
  const { logs, loading: logsLoading } = useTaskLogs({
    taskId: activeTaskId,
    logType,
    pageSize: 100,
    autoRefresh: true,
    enabled: Boolean(activeTaskId),
  });

  useEffect(() => {
    if (completedTasks.length === 0) {
      setSelectedTaskIds([]);
      setActiveTaskId("");
      return;
    }
    setSelectedTaskIds((current) => {
      const validIds = current.filter((id) => completedTasks.some((task) => task.id === id));
      return validIds.length > 0 ? validIds : [completedTasks[0].id];
    });
  }, [completedTasks]);

  useEffect(() => {
    if (selectedTaskIds.length === 0) {
      setActiveTaskId("");
      return;
    }
    setActiveTaskId((current) => (selectedTaskIds.includes(current) ? current : selectedTaskIds[0]));
  }, [selectedTaskIds]);

  useEffect(() => {
    let cancelled = false;
    const fetchLatestRuns = async () => {
      if (completedTasks.length === 0) {
        setLatestRunsByTask({});
        return;
      }
      const entries = await Promise.all(
        completedTasks.map(async (task) => {
          try {
            const result = await invokeCommand<string>("list_task_runs", {
              request: { task_id: task.id },
            });
            const runs = JSON.parse(result) as TaskRun[];
            return [task.id, runs[0] ?? null] as const;
          } catch (e) {
            console.error("Failed to fetch task runs:", e);
            return [task.id, null] as const;
          }
        })
      );
      if (!cancelled) {
        setLatestRunsByTask(Object.fromEntries(entries));
      }
    };
    fetchLatestRuns();
    return () => {
      cancelled = true;
    };
  }, [completedTasks]);

  const fetchVectorData = async (page = vectorPage) => {
    if (!activeTaskId) {
      setVectorStats(null);
      setVectors([]);
      setVectorTotal(0);
      return;
    }
    setVectorLoading(true);
    try {
      const [statsResult, listResult] = await Promise.all([
        invokeCommand<string>("get_vector_stats", { request: { task_id: activeTaskId } }),
        invokeCommand<string>("list_vectors", {
          request: { task_id: activeTaskId, limit: VECTOR_PAGE_SIZE, offset: page * VECTOR_PAGE_SIZE },
        }),
      ]);
      const stats = JSON.parse(statsResult) as VectorStats;
      const list = JSON.parse(listResult) as { items: VectorRecord[]; total: number };
      setVectorStats(stats);
      setVectors(list.items);
      setVectorTotal(list.total);
    } catch (e) {
      console.error("Failed to fetch vector data:", e);
    } finally {
      setVectorLoading(false);
    }
  };

  useEffect(() => {
    setVectorPage(0);
  }, [activeTaskId]);

  useEffect(() => {
    fetchVectorData();
  }, [activeTaskId, vectorPage]);

  useEffect(() => {
    if (progress?.status === "completed" || progress?.status === "failed" || progress?.status === "cancelled") {
      setIsRunning(false);
      fetchVectorData();
    }
  }, [progress?.status]);

  const showNotice = (action: string, nextNotice: ActionNoticeState) => {
    console.info(`[ui-action] vectorize-${action}`, nextNotice);
    setNotice(nextNotice);
  };

  const toggleTaskSelection = (taskId: string) => {
    setSelectedTaskIds((current) => {
      if (current.includes(taskId)) {
        const next = current.filter((id) => id !== taskId);
        return next;
      }
      return [...current, taskId];
    });
    setActiveTaskId(taskId);
  };

  const handleVectorize = async () => {
    if (selectedTaskIds.length === 0) return;

    setActionLoading(true);
    setIsRunning(true);
    showNotice("start", {
      tone: "running",
      title: "正在提交向量化任务",
      message: `正在提交 ${selectedTaskIds.length} 个已选任务结果。`,
    });
    try {
      const results = await Promise.allSettled(
        selectedTaskIds.map((taskId) =>
          invokeCommand<string>("start_vectorize", {
            request: {
              task_id: taskId,
              backend: "remote",
              batch_size: 500,
            },
          })
        )
      );
      await fetchProgress();
      await fetchVectorData();
      const successes = results
        .filter((result): result is PromiseFulfilledResult<string> => result.status === "fulfilled")
        .map((result) => JSON.parse(result.value) as {
          skipped_existing: number;
          pending: number;
          total: number;
          embedding_dim: number;
        });
      const failures = results.filter((result) => result.status === "rejected") as PromiseRejectedResult[];
      if (successes.length === 0) {
        throw new Error(failures.map((failure) => String(failure.reason)).join("；") || "没有任务成功启动。");
      }
      const total = successes.reduce((sum, item) => sum + item.total, 0);
      const pending = successes.reduce((sum, item) => sum + item.pending, 0);
      const skipped = successes.reduce((sum, item) => sum + item.skipped_existing, 0);
      showNotice("start-success", {
        tone: failures.length > 0 ? "warning" : "running",
        title: failures.length > 0 ? "部分向量化任务已启动" : "向量化已在后台启动",
        message: `已启动 ${successes.length}/${selectedTaskIds.length} 个任务；总计 ${total} 条，待处理 ${pending} 条，已存在 ${skipped} 条。${failures.length > 0 ? `失败 ${failures.length} 个。` : ""}`,
      });
    } catch (e) {
      showNotice("start-error", {
        tone: "error",
        title: "向量化启动失败",
        message: String(e),
      });
    } finally {
      setIsRunning(false);
      setActionLoading(false);
    }
  };

  const handleStopVectorize = async () => {
    if (selectedTaskIds.length === 0) return;
    setStopLoading(true);
    try {
      const results = await Promise.allSettled(
        selectedTaskIds.map((taskId) =>
          invokeCommand<string>("stop_vectorize", {
            request: { task_id: taskId },
          })
        )
      );
      const cancelledCount = results.reduce((count, result) => {
        if (result.status !== "fulfilled") return count;
        const response = JSON.parse(result.value) as { cancelled: boolean };
        return response.cancelled ? count + 1 : count;
      }, 0);
      showNotice("stop", {
        tone: cancelledCount > 0 ? "info" : "error",
        title: cancelledCount > 0 ? "已请求取消向量化" : "没有正在运行的向量化任务",
        message: cancelledCount > 0 ? `已对 ${cancelledCount} 个任务发出取消请求，后端会在当前 embedding 请求结束后停止写入后续批次。` : "当前已选任务没有可取消的向量化进程。",
      });
    } catch (e) {
      showNotice("stop-error", {
        tone: "error",
        title: "取消向量化失败",
        message: String(e),
      });
    } finally {
      setStopLoading(false);
    }
  };

  const handleDeleteVector = async (domainId: number) => {
    setVectorActionId(domainId);
    try {
      await invokeCommand("delete_vector", { request: { domain_id: domainId } });
      await fetchProgress();
      if (vectors.length === 1 && vectorPage > 0) {
        setVectorPage((page) => Math.max(0, page - 1));
      } else {
        await fetchVectorData();
      }
    } catch (e) {
      showNotice("delete-vector", {
        tone: "error",
        title: "删除向量失败",
        message: String(e),
      });
    } finally {
      setVectorActionId(null);
    }
  };

  const handleRevectorizeItem = async (domainId: number) => {
    setVectorActionId(domainId);
    try {
      await invokeCommand("revectorize_item", { request: { domain_id: domainId } });
      await fetchProgress();
      await fetchVectorData();
    } catch (e) {
      showNotice("revectorize-item", {
        tone: "error",
        title: "重建向量失败",
        message: String(e),
      });
    } finally {
      setVectorActionId(null);
    }
  };

  const handleDeleteTaskVectors = async () => {
    if (!activeTaskId) return;
    const ok = window.confirm("确认清空当前查看任务的全部向量？清空后语义筛选需要重新向量化。");
    if (!ok) return;
    setVectorActionId("all");
    try {
      await invokeCommand("delete_task_vectors", { request: { task_id: activeTaskId } });
      setVectorPage(0);
      await fetchProgress();
      await fetchVectorData();
    } catch (e) {
      showNotice("delete-task-vectors", {
        tone: "error",
        title: "清空向量库失败",
        message: String(e),
      });
    } finally {
      setVectorActionId(null);
    }
  };

  const gpuBackendLabel = gpuStatus?.available
    ? gpuStatus.backend === "cuda" ? "CUDA"
    : gpuStatus.backend === "directml" ? "DirectML"
    : gpuStatus.backend === "cpu" ? "CPU"
    : gpuStatus.backend
    : "CPU";
  const gpuDeviceName = gpuStatus?.device_name || "CPU Only";
  const total = progress?.total ?? activeTaskInfo?.available_count ?? 0;
  const processed = progress?.processed ?? 0;
  const percentage = progress?.percentage ?? 0;
  const progressMessage = progress?.message || getProgressStatusLabel(progress?.status, isRunning);
  const isVectorizing = isRunning || progress?.status === "running" || vectorStats?.running;
  const vectorPageCount = Math.max(1, Math.ceil(vectorTotal / VECTOR_PAGE_SIZE));
  const canPrevVectors = vectorPage > 0;
  const canNextVectors = (vectorPage + 1) * VECTOR_PAGE_SIZE < vectorTotal;
  const lastRun = vectorStats?.last_run ?? null;

  return (
    <div className="page-shell max-w-4xl">
      <div>
        <div className="eyebrow mb-3">VECTOR PIPELINE</div>
        <h1 className="page-heading">向量化处理</h1>
        <p className="page-subtitle">将域名文本转化为向量，为语义搜索与智能筛选准备可召回的数据层。</p>
      </div>
      {notice && <ActionNotice notice={notice} onClose={() => setNotice(null)} />}

      <div className="glass-panel p-5">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-sm font-semibold text-cyber-text flex items-center gap-2">
            <Cpu className="w-4 h-4 text-cyber-green" />
            GPU 状态
          </h2>
          <span className={`text-xs px-2 py-0.5 rounded border ${
            gpuStatus?.available && gpuStatus.backend !== "cpu"
              ? "bg-white/[0.06] text-cyber-green border-white/12"
              : "bg-cyber-orange/10 text-cyber-orange border-cyber-orange/25"
          }`}>
            {gpuStatus?.available && gpuStatus.backend !== "cpu" ? "GPU 已启用" : "CPU 模式"}
          </span>
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
          <div className="metric-tile">
            <p className="text-[10px] text-cyber-muted mb-1">当前后端</p>
            <p className="text-sm font-semibold text-cyber-text">{gpuBackendLabel}</p>
          </div>
          <div className="metric-tile">
            <p className="text-[10px] text-cyber-muted mb-1">设备名称</p>
            <p className="text-sm font-semibold text-cyber-text">{gpuDeviceName}</p>
          </div>
          <div className="metric-tile">
            <p className="text-[10px] text-cyber-muted mb-1">模型</p>
            <p className="text-sm font-semibold text-cyber-text">MiniLM-L6-v2</p>
          </div>
          <div className="metric-tile">
            <p className="text-[10px] text-cyber-muted mb-1">维度</p>
            <p className="text-sm font-semibold text-cyber-text">384</p>
          </div>
        </div>
        {(!gpuStatus?.available || gpuStatus.backend === "cpu") && (
          <div className="mt-3 flex items-center gap-2 text-xs text-cyber-muted">
            <Cpu className="w-3.5 h-3.5 text-cyber-orange" />
            <span>未检测到 GPU，将使用 CPU 推理。可在设置中配置 GPU 后端加速。</span>
          </div>
        )}
      </div>

      <div className="glass-panel p-5 space-y-4">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-sm font-semibold text-cyber-text">选择任务结果</h2>
            <p className="mt-1 text-xs text-cyber-muted-dim">
              勾选多个任务结果后可一次性提交向量化；点击任务名称可切换下方进度、日志和向量库视图。
            </p>
          </div>
          {completedTasks.length > 0 && (
            <div className="flex shrink-0 items-center gap-2 text-xs">
              <span className="text-cyber-muted-dim">已选 {selectedTaskIds.length}</span>
              <button
                type="button"
                className="cyber-btn-secondary cyber-btn-sm"
                onClick={() => {
                  setSelectedTaskIds(completedTasks.map((task) => task.id));
                  setActiveTaskId(completedTasks[0]?.id ?? "");
                }}
              >
                全选
              </button>
              <button
                type="button"
                className="cyber-btn-secondary cyber-btn-sm"
                onClick={() => setSelectedTaskIds([])}
              >
                清空
              </button>
            </div>
          )}
        </div>
        {completedTasks.length === 0 ? (
          <div className="text-center py-8 text-cyber-muted">
            <Inbox className="w-10 h-10 mx-auto mb-3 opacity-40" />
            <p className="text-sm">暂无已完成且含可用域名的任务</p>
            <p className="text-xs text-cyber-muted-dim mt-1">请先完成扫描任务</p>
          </div>
        ) : (
          <div className="space-y-2">
            {completedTasks.map((task, index) => {
              const checked = selectedTaskIds.includes(task.id);
              const active = activeTaskId === task.id;
              const latestRun = latestRunsByTask[task.id];
              return (
                <div
                  key={task.id}
                  className={`grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 overflow-hidden rounded-lg border p-3 transition-all ${
                    active
                      ? "border-white/25 bg-white/[0.07]"
                      : checked
                        ? "border-cyber-green/30 bg-cyber-green/[0.05]"
                        : "border-cyber-border bg-cyber-bg/50 hover:border-cyber-border-light"
                  }`}
                >
                  <input
                    type="checkbox"
                    aria-label={`选择 ${task.name}`}
                    checked={checked}
                    onChange={() => toggleTaskSelection(task.id)}
                    className="h-4 w-4 accent-cyber-green"
                  />
                  <button
                    type="button"
                    className="min-w-0 text-left"
                    onClick={() => setActiveTaskId(task.id)}
                  >
                    <span className="block truncate text-sm font-medium text-cyber-text" title={task.name}>
                      {task.name}
                    </span>
                    <span className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-[11px] text-cyber-muted-dim">
                      <span className="font-mono">
                        {latestRun ? `Run #${latestRun.run_number}` : `Task #${index + 1}`}
                      </span>
                      <span>{formatDateTime(latestRun?.started_at ?? task.created_at)}</span>
                      <span className="font-mono">{task.id.slice(0, 8)}</span>
                    </span>
                  </button>
                  <div className="flex shrink-0 items-center gap-3">
                    {active && (
                      <span className="rounded border border-white/15 bg-white/[0.06] px-2 py-0.5 text-[11px] text-cyber-text-secondary">
                        当前查看
                      </span>
                    )}
                    <span className="text-xs text-cyber-muted">{task.available_count} 个可用域名</span>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      <div className="glass-panel p-5 space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-semibold text-cyber-text flex items-center gap-2">
            <Gauge className="w-4 h-4 text-cyber-cyan" />
            向量化进度
          </h2>
          {isRunning && (
            <span className="text-xs text-cyber-green animate-pulse">处理中...</span>
          )}
        </div>

        {activeTaskId ? (
          <div className="space-y-3">
            <div className="h-2 rounded-sm bg-cyber-surface overflow-hidden">
              <div
                className={`h-full rounded-sm bg-white transition-all duration-300 ${isVectorizing ? "animate-pulse" : ""}`}
                style={{ width: `${Math.max(0, Math.min(100, percentage))}%` }}
              />
            </div>
            <div className="grid grid-cols-3 gap-4 text-xs">
              <div className="flex items-center gap-2">
                <HardDrive className="w-3.5 h-3.5 text-cyber-muted" />
                <span className="text-cyber-muted">已处理:</span>
                <span className="text-cyber-text font-medium">{processed} / {total}</span>
              </div>
              <div className="flex items-center gap-2">
                <Gauge className="w-3.5 h-3.5 text-cyber-muted" />
                <span className="text-cyber-muted">速度:</span>
                <span className="text-cyber-text font-medium">{formatSpeed(progress?.speed_per_sec)}</span>
              </div>
              <div className="flex items-center gap-2">
                <Clock className="w-3.5 h-3.5 text-cyber-muted" />
                <span className="text-cyber-muted">预计剩余:</span>
                <span className="text-cyber-text font-medium">{formatDuration(progress?.estimated_remaining_secs)}</span>
              </div>
            </div>
            <div className="flex items-center justify-between text-xs text-cyber-muted">
              <span>{progressMessage}</span>
              <span>{percentage.toFixed(1)}%</span>
            </div>
          </div>
        ) : (
          <div className="text-center py-8 text-cyber-muted">
            <Cpu className="w-10 h-10 mx-auto mb-2 opacity-40" />
            <p className="text-sm">选择任务后点击开始</p>
          </div>
        )}
      </div>

      <div className="glass-panel overflow-hidden">
        <div className="flex items-center justify-between p-5 border-b border-cyber-border">
          <h2 className="text-sm font-semibold text-cyber-text flex items-center gap-2">
            <Database className="w-4 h-4 text-cyber-green" />
            向量库
          </h2>
          <div className="flex items-center gap-2">
            <button
              onClick={() => fetchVectorData()}
              className="cyber-btn-secondary cyber-btn-sm"
              disabled={!activeTaskId || vectorLoading}
            >
              <RefreshCcw className={`w-3.5 h-3.5 ${vectorLoading ? "animate-spin" : ""}`} />
              刷新
            </button>
            <button
              onClick={handleDeleteTaskVectors}
              className="cyber-btn-secondary cyber-btn-sm text-cyber-red"
              disabled={!activeTaskId || isVectorizing || vectorStats?.vector_count === 0 || vectorActionId === "all"}
            >
              <Trash2 className="w-3.5 h-3.5" />
              清空
            </button>
          </div>
        </div>

        {!activeTaskId ? (
          <div className="text-center py-8 text-cyber-muted">
            <Database className="w-10 h-10 mx-auto mb-3 opacity-40" />
            <p className="text-sm">选择任务后查看向量库</p>
          </div>
        ) : (
          <div className="p-5 space-y-4">
            <div className="grid grid-cols-2 lg:grid-cols-6 gap-3">
              <VectorMetric label="表" value={vectorStats?.table_name ?? "domain_vectors"} />
              <VectorMetric label="维度" value={String(vectorStats?.embedding_dim ?? 384)} />
              <VectorMetric label="可用域名" value={formatNumber(vectorStats?.total_available)} />
              <VectorMetric label="已入库" value={formatNumber(vectorStats?.vector_count)} />
              <VectorMetric label="覆盖率" value={`${(vectorStats?.coverage ?? 0).toFixed(1)}%`} />
              <VectorMetric label="上次运行" value={lastRun ? getProgressStatusLabel(lastRun.status, false) : "暂无"} />
            </div>

            {lastRun && (
              <div className="grid gap-2 rounded-lg border border-cyber-border bg-cyber-bg/40 p-3 text-xs sm:grid-cols-3">
                <div className="min-w-0">
                  <p className="text-cyber-muted-dim">运行 ID</p>
                  <p className="truncate font-mono text-cyber-text-secondary" title={lastRun.id}>{lastRun.id}</p>
                </div>
                <div>
                  <p className="text-cyber-muted-dim">运行进度</p>
                  <p className="font-mono text-cyber-text-secondary">
                    {lastRun.processed_count.toLocaleString()} / {lastRun.total_count.toLocaleString()}
                  </p>
                </div>
                <div>
                  <p className="text-cyber-muted-dim">更新时间</p>
                  <p className="text-cyber-text-secondary">{formatDateTime(lastRun.updated_at)}</p>
                </div>
              </div>
            )}

            <div className="h-2 rounded-sm bg-cyber-surface overflow-hidden">
              <div
                className="h-full rounded-sm bg-cyber-green transition-all duration-300"
                style={{ width: `${Math.max(0, Math.min(100, vectorStats?.coverage ?? 0))}%` }}
              />
            </div>

            <div className="rounded-lg border border-cyber-border overflow-hidden">
              <div className="grid grid-cols-[minmax(0,1fr)_80px_80px_140px] gap-3 px-4 py-2 bg-cyber-bg-elevated/70 text-[11px] text-cyber-muted-dim">
                <span>域名</span>
                <span>TLD</span>
                <span>维度</span>
                <span className="text-right">操作</span>
              </div>
              {vectors.length === 0 ? (
                <div className="text-center py-8 text-cyber-muted">
                  <Inbox className="w-8 h-8 mx-auto mb-2 opacity-30" />
                  <p className="text-xs">暂无向量记录</p>
                </div>
              ) : (
                <div className="divide-y divide-cyber-border/10">
                  {vectors.map((record) => (
                    <div
                      key={record.domain_id}
                      className="grid grid-cols-[minmax(0,1fr)_80px_80px_140px] items-center gap-3 px-4 py-3 text-xs"
                    >
                      <span className="min-w-0 truncate font-mono text-cyber-text" title={record.domain}>
                        {record.domain}
                      </span>
                      <span className="text-cyber-text-secondary">{record.tld}</span>
                      <span className="font-mono text-cyber-text-secondary">{record.vector_dim}</span>
                      <div className="flex justify-end gap-2">
                        <button
                          onClick={() => handleRevectorizeItem(record.domain_id)}
                          className="grid h-8 w-8 place-items-center rounded border border-cyber-border bg-cyber-bg/60 text-cyber-muted hover:text-cyber-text disabled:opacity-50"
                          title="重建向量"
                          disabled={isVectorizing || vectorActionId === record.domain_id}
                        >
                          <RefreshCcw className={`w-4 h-4 ${vectorActionId === record.domain_id ? "animate-spin" : ""}`} />
                        </button>
                        <button
                          onClick={() => handleDeleteVector(record.domain_id)}
                          className="grid h-8 w-8 place-items-center rounded border border-cyber-border bg-cyber-bg/60 text-cyber-red hover:border-cyber-red/40 disabled:opacity-50"
                          title="删除向量"
                          disabled={isVectorizing || vectorActionId === record.domain_id}
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
            <div className="flex flex-col gap-3 text-xs text-cyber-muted-dim sm:flex-row sm:items-center sm:justify-between">
              <span>
                第 {(vectorPage + 1).toLocaleString()} / {vectorPageCount.toLocaleString()} 页，当前显示 {vectors.length.toLocaleString()} 条，共 {vectorTotal.toLocaleString()} 条。向量保存在本地 SQLite 的 sqlite-vec 表中。
              </span>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setVectorPage((page) => Math.max(0, page - 1))}
                  className="cyber-btn-secondary cyber-btn-sm"
                  disabled={!canPrevVectors || vectorLoading}
                >
                  上一页
                </button>
                <button
                  onClick={() => setVectorPage((page) => page + 1)}
                  className="cyber-btn-secondary cyber-btn-sm"
                  disabled={!canNextVectors || vectorLoading}
                >
                  下一页
                </button>
              </div>
            </div>
          </div>
        )}
      </div>

      <div className="glass-panel overflow-hidden">
        <div className="flex items-center justify-between p-5 border-b border-cyber-border">
          <h2 className="text-sm font-semibold text-cyber-text flex items-center gap-2">
            <Terminal className="w-4 h-4 text-cyber-green" />
            向量化日志
          </h2>
          <div className="flex items-center gap-1 rounded-md border border-cyber-border bg-cyber-bg/50 p-1 text-xs">
            {(["task", "request"] as const).map((type) => (
              <button
                key={type}
                onClick={() => setLogType(type)}
                className={`px-2.5 py-1 rounded transition-all ${
                  logType === type ? "bg-white/[0.08] text-white" : "text-cyber-muted hover:text-cyber-text"
                }`}
              >
                {type === "task" ? "任务日志" : "请求日志"}
              </button>
            ))}
          </div>
        </div>
        <div className="bg-cyber-bg-elevated/80 max-h-56 overflow-y-auto font-mono text-xs leading-relaxed">
          {!activeTaskId ? (
            <div className="text-center py-8 text-cyber-muted">
              <Terminal className="w-8 h-8 mx-auto mb-2 opacity-30" />
              <p className="text-xs">选择任务后查看日志</p>
            </div>
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
      </div>

      <div className="flex justify-end gap-3">
        {isVectorizing && (
          <button
            onClick={handleStopVectorize}
            className="cyber-btn-secondary flex items-center gap-2"
            disabled={stopLoading}
          >
            <XCircle className="w-4 h-4" />
            {stopLoading ? "取消中..." : "取消向量化"}
          </button>
        )}
        <button
          onClick={handleVectorize}
          className="cyber-btn-primary flex items-center gap-2"
          disabled={selectedTaskIds.length === 0 || actionLoading || isVectorizing}
        >
          <Play className="w-4 h-4" />
          {actionLoading || isVectorizing ? "向量化中..." : `开始向量化${selectedTaskIds.length > 1 ? ` (${selectedTaskIds.length})` : ""}`}
        </button>
      </div>
    </div>
  );
}

function formatSpeed(speed: number | null | undefined) {
  if (!speed || speed <= 0) return "-";
  return `${speed >= 10 ? speed.toFixed(0) : speed.toFixed(1)} 个/秒`;
}

function VectorMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric-tile">
      <p className="text-[10px] text-cyber-muted mb-1">{label}</p>
      <p className="text-sm font-semibold text-cyber-text truncate" title={value}>{value}</p>
    </div>
  );
}

function formatNumber(value: number | null | undefined) {
  return (value ?? 0).toLocaleString();
}

function formatDuration(seconds: number | null | undefined) {
  if (seconds === null || seconds === undefined || !Number.isFinite(seconds)) return "-";
  if (seconds <= 1) return "< 1 秒";
  const rounded = Math.round(seconds);
  const minutes = Math.floor(rounded / 60);
  const restSeconds = rounded % 60;
  if (minutes <= 0) return `${restSeconds} 秒`;
  return `${minutes} 分 ${restSeconds} 秒`;
}

function getProgressStatusLabel(status: string | undefined, isRunning: boolean) {
  if (isRunning || status === "running") return "向量化处理中";
  if (status === "completed") return "向量化已完成";
  if (status === "cancelled") return "向量化已取消";
  if (status === "interrupted") return "向量化已中断";
  if (status === "failed") return "向量化失败";
  return "等待开始";
}

function formatDateTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}
