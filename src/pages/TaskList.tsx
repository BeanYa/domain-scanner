import { startTransition, useCallback, useEffect, useMemo, useState } from "react";
import {
  Search,
  ChevronDown,
  ChevronRight,
  ExternalLink,
  Filter,
  Inbox,
} from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useTaskEvents } from "../hooks/useTaskEvents";
import { useTaskStore } from "../store/taskStore";
import { useBatchStore } from "../store/batchStore";
import type { TaskStatus } from "../types";
import ActionNotice, { type ActionNoticeState } from "../components/ActionNotice";

const TASK_LIST_POLL_INTERVAL_MS = 15000;

const statusConfig: Record<TaskStatus, { label: string; dotClass: string; badgeClass: string; color: string }> = {
  running:   { label: "运行中", dotClass: "status-dot-running", badgeClass: "badge-green",   color: "text-cyber-green" },
  paused:    { label: "已暂停", dotClass: "status-dot-paused",  badgeClass: "badge-orange",  color: "text-cyber-orange" },
  stopped:   { label: "已停止", dotClass: "status-dot-idle",    badgeClass: "badge-red",     color: "text-cyber-red" },
  completed: { label: "已完成", dotClass: "status-dot-completed", badgeClass: "badge-blue",    color: "text-cyber-blue" },
  pending:   { label: "等待中", dotClass: "status-dot-idle",    badgeClass: "badge-neutral", color: "text-cyber-muted-dim" },
};

export default function TaskList() {
  const navigate = useNavigate();
  const [search, setSearch] = useState("");
  const [expandedBatches, setExpandedBatches] = useState<Set<string>>(new Set());
  const [notice, setNotice] = useState<ActionNoticeState | null>(null);
  const { tasks, fetchTasks, applyTaskProgress, applyTaskStatus } = useTaskStore();
  const { batches, fetchBatches } = useBatchStore();

  useEffect(() => {
    fetchBatches();
    fetchTasks();
  }, [fetchBatches, fetchTasks]);

  const handleProgress = useCallback(
    (event: {
      task_id: string;
      completed_count: number;
      total_count: number;
      available_count: number;
      error_count: number;
    }) => {
      startTransition(() => {
        applyTaskProgress(event);
      });
    },
    [applyTaskProgress]
  );

  const handleStatusChange = useCallback(
    (event: { task_id: string; status: string }) => {
      const knownStatuses = new Set<TaskStatus>([
        "pending",
        "running",
        "paused",
        "stopped",
        "completed",
      ]);
      if (!knownStatuses.has(event.status as TaskStatus)) {
        return;
      }

      startTransition(() => {
        applyTaskStatus(event.task_id, event.status as TaskStatus);
      });
    },
    [applyTaskStatus]
  );

  useTaskEvents(handleProgress, handleStatusChange);

  const hasRunning = useMemo(
    () => tasks.some((task) => task.status === "running"),
    [tasks]
  );

  // Auto-refresh while any task is running
  useEffect(() => {
    if (!hasRunning) return;
    const interval = setInterval(() => fetchTasks(), TASK_LIST_POLL_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [hasRunning, fetchTasks]);

  // Group tasks by batch_id
  const batchMap = new Map<string, typeof tasks>();
  const orphanTasks: typeof tasks = [];
  for (const task of tasks) {
    if (task.batch_id) {
      const list = batchMap.get(task.batch_id) || [];
      list.push(task);
      batchMap.set(task.batch_id, list);
    } else {
      orphanTasks.push(task);
    }
  }

  const toggleBatch = (id: string) => {
    setExpandedBatches((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  };

  const filteredTasks = (list: typeof tasks) =>
    search
      ? list.filter((t) =>
          t.name.toLowerCase().includes(search.toLowerCase()) ||
          t.tlds.some((tld) => tld.includes(search)) ||
          t.status.includes(search.toLowerCase())
        )
      : list;

  return (
    <div className="page-shell">
      <div className="flex items-end justify-between gap-4">
        <div>
          <div className="eyebrow mb-3">SCAN ARCHIVE</div>
          <h1 className="page-heading">任务列表</h1>
          <p className="page-subtitle">管理批次、查看进度，并从同一张任务胶片中追踪所有域名扫描结果。</p>
        </div>
        <button
          onClick={() => navigate("/tasks/new")}
          className="cyber-btn-primary"
        >
          新建任务
        </button>
      </div>

      <div className="flex items-center gap-3">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-cyber-muted-dim pointer-events-none" />
          <input
            type="text"
            placeholder="搜索任务名称、TLD 或状态..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="cyber-input pl-10"
          />
        </div>
        <button
          className="cyber-btn-secondary cyber-btn-sm"
          onClick={() => {
            const nextNotice = {
              tone: "info" as const,
              title: "筛选入口已响应",
              message: "任务列表当前支持搜索任务名称、TLD 和状态；独立筛选面板尚未接入，后续会在这里展开更多条件。",
            };
            console.info("[ui-action] task-list-filter", nextNotice);
            setNotice(nextNotice);
          }}
        >
          <Filter className="w-3.5 h-3.5" /> 筛选
        </button>
      </div>

      {notice && <ActionNotice notice={notice} onClose={() => setNotice(null)} />}

      {tasks.length === 0 ? (
        <div className="glass-panel p-12 text-center text-cyber-muted">
          <Inbox className="w-10 h-10 mx-auto mb-3 opacity-40" />
          <p className="text-sm">暂无扫描任务</p>
          <p className="text-xs text-cyber-muted-dim mt-1">点击「新建任务」开始扫描</p>
        </div>
      ) : (
        <div className="space-y-4">
          {batches.map((batch) => {
            const batchTasks = filteredTasks(batchMap.get(batch.id) || []);
            if (batchTasks.length === 0) return null;
            const isExpanded = expandedBatches.has(batch.id);
            return (
              <div key={batch.id} className="glass-panel overflow-hidden">
                <div className="px-5 py-4 flex items-center gap-3 cursor-pointer hover:bg-cyber-card transition-colors border-b border-cyber-border group overflow-hidden"
                  onClick={() => toggleBatch(batch.id)}>
                  <ChevronDown className={`w-4 h-4 text-cyber-muted transition-transform ${isExpanded ? "" : "-rotate-90"}`} />
                  <span className="min-w-0 truncate text-sm font-semibold text-cyber-text">{batch.name}</span>
                  <span className="badge-neutral ml-1">{batchTasks.length} 任务</span>
                  <div className="ml-auto flex items-center gap-4 mr-4">
                    <div className="hidden md:flex items-center gap-4 text-xs">
                      <span className="text-cyber-muted">
                        运行: <strong className="text-cyber-green">{batchTasks.filter(t => t.status === "running").length}</strong>
                      </span>
                      <span className="text-cyber-border-light">/</span>
                      <span className="text-cyber-muted">
                        完成: <strong className="text-cyber-blue">{batchTasks.filter(t => t.status === "completed").length}</strong>
                      </span>
                      <span className="text-cyber-border-light">/</span>
                      <span className="text-cyber-muted">
                        可用: <strong className="text-cyber-orange tabular-nums">{batchTasks.reduce((sum, t) => sum + t.available_count, 0).toLocaleString()}</strong>
                      </span>
                    </div>
                  </div>
                </div>
                {isExpanded && (
                  <div className="divide-y divide-cyber-border/15">
                    {batchTasks.map((task) => {
                      const cfg = statusConfig[task.status] || statusConfig.pending;
                      const progress = task.total_count > 0 ? Math.round((task.completed_count / task.total_count) * 100) : 0;
                      return (
                        <div
                          key={task.id}
                          className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-4 px-5 py-4 hover:bg-cyber-bg-elevated/40 transition-colors cursor-pointer group/task overflow-hidden"
                          onClick={() => navigate(`/tasks/${task.id}`)}
                        >
                          <span className={`dot ${cfg.dotClass} shrink-0`} />
                          <div className="flex-1 min-w-0">
                            <div className="flex min-w-0 items-center gap-2">
                              <span
                                className="block min-w-0 truncate text-sm font-semibold text-cyber-text group-hover/task:text-cyber-green transition-colors"
                                title={task.name}
                              >
                                {task.name}
                              </span>
                              <span className={`${cfg.badgeClass} text-[11px] shrink-0`}>{cfg.label}</span>
                            </div>
                            <div className="mt-2 flex max-h-7 flex-wrap items-center gap-1 overflow-hidden">
                              {task.tlds.slice(0, 5).map((tld) => (
                                <span key={tld} className="badge-neutral text-[11px]">{tld}</span>
                              ))}
                              {task.tlds.length > 5 && (
                                <span className="badge-blue text-[11px]">+{task.tlds.length - 5}</span>
                              )}
                            </div>
                            <div className="mt-2 flex items-center gap-3">
                              <div className="progress-bar flex-1 max-w-[280px]">
                                <div className="progress-bar-fill" style={{ width: `${progress}%` }} />
                              </div>
                              <span className="text-xs font-mono text-cyber-muted-dim w-8 tabular-nums">{progress}%</span>
                            </div>
                          </div>
                          <div className="hidden lg:flex items-center gap-6 text-right shrink-0 pl-3 border-l border-cyber-border">
                            <div>
                              <p className="text-sm font-semibold text-cyber-green tabular-nums">{task.available_count.toLocaleString()}</p>
                              <p className="text-[10px] text-cyber-muted-dim">可用</p>
                            </div>
                            <div>
                              <p className="text-sm font-mono text-cyber-text-secondary tabular-nums">{task.completed_count.toLocaleString()}</p>
                              <p className="text-[10px] text-cyber-muted-dim">完成</p>
                            </div>
                            {task.error_count > 0 && (
                              <div>
                                <p className="text-sm font-mono text-cyber-red tabular-nums">{task.error_count}</p>
                                <p className="text-[10px] text-cyber-muted-dim">错误</p>
                              </div>
                            )}
                          </div>
                          <ExternalLink className="w-4 h-4 text-cyber-border-light opacity-0 group-hover/task:opacity-100 group-hover/task:text-cyber-green transition-all shrink-0" />
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            );
          })}

          {filteredTasks(orphanTasks).map((task) => {
            const cfg = statusConfig[task.status] || statusConfig.pending;
            const progress = task.total_count > 0 ? Math.round((task.completed_count / task.total_count) * 100) : 0;
            return (
              <div
                key={task.id}
                className="glass-panel grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-4 px-5 py-4 hover:bg-cyber-bg-elevated/40 transition-colors cursor-pointer group/task overflow-hidden"
                onClick={() => navigate(`/tasks/${task.id}`)}
              >
                <span className={`dot ${cfg.dotClass} shrink-0`} />
                <div className="flex-1 min-w-0">
                  <div className="flex min-w-0 items-center gap-2">
                    <span
                      className="block min-w-0 truncate text-sm font-semibold text-cyber-text group-hover/task:text-cyber-green transition-colors"
                      title={task.name}
                    >
                      {task.name}
                    </span>
                    <span className={`${cfg.badgeClass} text-[11px] shrink-0`}>{cfg.label}</span>
                  </div>
                  <div className="mt-2 flex max-h-7 flex-wrap items-center gap-1 overflow-hidden">
                    {task.tlds.slice(0, 5).map((tld) => (
                      <span key={tld} className="badge-neutral text-[11px]">{tld}</span>
                    ))}
                    {task.tlds.length > 5 && (
                      <span className="badge-blue text-[11px]">+{task.tlds.length - 5}</span>
                    )}
                  </div>
                  <div className="mt-2 flex items-center gap-3">
                    <div className="progress-bar flex-1 max-w-[280px]">
                      <div className="progress-bar-fill" style={{ width: `${progress}%` }} />
                    </div>
                    <span className="text-xs font-mono text-cyber-muted-dim w-8 tabular-nums">{progress}%</span>
                  </div>
                </div>
                <ExternalLink className="w-4 h-4 text-cyber-border-light opacity-0 group-hover/task:opacity-100 group-hover/task:text-cyber-green transition-all shrink-0" />
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
