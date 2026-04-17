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
import type { VectorRecord, VectorStats } from "../types";

type VectorLogType = "task" | "request";

const logColors: Record<string, string> = {
  debug: "text-cyber-muted",
  info: "text-cyber-cyan",
  warn: "text-cyber-orange",
  error: "text-cyber-red",
};

const VECTOR_PAGE_SIZE = 50;

export default function VectorizePage() {
  const [selectedTask, setSelectedTask] = useState("");
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
  const selectedTaskInfo = completedTasks.find((task) => task.id === selectedTask);
  const { progress, fetchProgress } = useVectorProgress(selectedTask || null);
  const { logs, loading: logsLoading } = useTaskLogs({
    taskId: selectedTask,
    logType,
    pageSize: 100,
    autoRefresh: true,
    enabled: Boolean(selectedTask),
  });

  useEffect(() => {
    if (completedTasks.length === 0) {
      setSelectedTask("");
      return;
    }
    setSelectedTask((current) =>
      completedTasks.some((task) => task.id === current) ? current : completedTasks[0].id
    );
  }, [completedTasks]);

  const fetchVectorData = async (page = vectorPage) => {
    if (!selectedTask) {
      setVectorStats(null);
      setVectors([]);
      setVectorTotal(0);
      return;
    }
    setVectorLoading(true);
    try {
      const [statsResult, listResult] = await Promise.all([
        invokeCommand<string>("get_vector_stats", { request: { task_id: selectedTask } }),
        invokeCommand<string>("list_vectors", {
          request: { task_id: selectedTask, limit: VECTOR_PAGE_SIZE, offset: page * VECTOR_PAGE_SIZE },
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
  }, [selectedTask]);

  useEffect(() => {
    fetchVectorData();
  }, [selectedTask, vectorPage]);

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

  const handleVectorize = async () => {
    if (!selectedTask) return;

    setActionLoading(true);
    setIsRunning(true);
    showNotice("start", {
      tone: "running",
      title: "正在提交向量化任务",
      message: `${selectedTaskInfo?.name ?? selectedTask} 正在调用 start_vectorize。`,
    });
    try {
      const result = await invokeCommand<string>("start_vectorize", {
        request: {
          task_id: selectedTask,
          backend: "remote",
          batch_size: 500,
        },
      });
      await fetchProgress();
      const response = JSON.parse(result) as {
        skipped_existing: number;
        pending: number;
        total: number;
        embedding_dim: number;
      };
      await fetchVectorData();
      showNotice("start-success", {
        tone: "running",
        title: "向量化已在后台启动",
        message: `总计 ${response.total} 条，待处理 ${response.pending} 条，已存在 ${response.skipped_existing} 条，维度 ${response.embedding_dim}。`,
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
    if (!selectedTask) return;
    setStopLoading(true);
    try {
      const result = await invokeCommand<string>("stop_vectorize", {
        request: { task_id: selectedTask },
      });
      const response = JSON.parse(result) as { cancelled: boolean };
      showNotice("stop", {
        tone: response.cancelled ? "info" : "error",
        title: response.cancelled ? "已请求取消向量化" : "没有正在运行的向量化任务",
        message: response.cancelled ? "后端会在当前 embedding 请求结束后停止写入后续批次。" : "当前任务没有可取消的向量化进程。",
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
    if (!selectedTask) return;
    const ok = window.confirm("确认清空该任务的全部向量？清空后语义筛选需要重新向量化。");
    if (!ok) return;
    setVectorActionId("all");
    try {
      await invokeCommand("delete_task_vectors", { request: { task_id: selectedTask } });
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
  const total = progress?.total ?? selectedTaskInfo?.available_count ?? 0;
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
        <h2 className="text-sm font-semibold text-cyber-text">选择源任务</h2>
        {completedTasks.length === 0 ? (
          <div className="text-center py-8 text-cyber-muted">
            <Inbox className="w-10 h-10 mx-auto mb-3 opacity-40" />
            <p className="text-sm">暂无已完成且含可用域名的任务</p>
            <p className="text-xs text-cyber-muted-dim mt-1">请先完成扫描任务</p>
          </div>
        ) : (
          <div className="space-y-2">
            {completedTasks.map((task) => (
              <label
                key={task.id}
                className={`grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 overflow-hidden p-3 rounded-lg cursor-pointer transition-all ${
                  selectedTask === task.id
                    ? "bg-white/[0.06] border border-white/20"
                    : "bg-cyber-bg/50 border border-cyber-border hover:border-cyber-border-light"
                }`}
              >
                <input
                  type="radio"
                  name="task"
                  value={task.id}
                  checked={selectedTask === task.id}
                  onChange={(e) => setSelectedTask(e.target.value)}
                  className="accent-cyber-green"
                />
                <span className="min-w-0 truncate text-sm text-cyber-text font-medium" title={task.name}>
                  {task.name}
                </span>
                <span className="shrink-0 text-xs text-cyber-muted">{task.available_count} 个可用域名</span>
              </label>
            ))}
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

        {selectedTask ? (
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
              disabled={!selectedTask || vectorLoading}
            >
              <RefreshCcw className={`w-3.5 h-3.5 ${vectorLoading ? "animate-spin" : ""}`} />
              刷新
            </button>
            <button
              onClick={handleDeleteTaskVectors}
              className="cyber-btn-secondary cyber-btn-sm text-cyber-red"
              disabled={!selectedTask || isVectorizing || vectorStats?.vector_count === 0 || vectorActionId === "all"}
            >
              <Trash2 className="w-3.5 h-3.5" />
              清空
            </button>
          </div>
        </div>

        {!selectedTask ? (
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
          {!selectedTask ? (
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
          disabled={!selectedTask || actionLoading || isVectorizing}
        >
          <Play className="w-4 h-4" />
          {actionLoading || isVectorizing ? "向量化中..." : "开始向量化"}
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
