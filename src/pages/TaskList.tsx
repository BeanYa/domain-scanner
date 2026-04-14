import { useState } from "react";
import {
  Search,
  ChevronDown,
  ChevronRight,
  Play,
  Pause,
  CheckCircle2,
  XCircle,
  Clock,
  ExternalLink,
  Filter,
  MoreHorizontal,
} from "lucide-react";
import { useNavigate } from "react-router-dom";

interface MockTask {
  id: string;
  name: string;
  tlds: string[];
  status: TaskStatus;
  total_count: number;
  completed_count: number;
  available_count: number;
  error_count: number;
  batch_id: string | null;
}

type TaskStatus = "running" | "paused" | "completed" | "failed" | "pending";

const mockBatches = [
  {
    id: "b1",
    name: "批量扫描 #1",
    tasks: [
      { id: "t1", name: "4字母扫描", tlds: [".com", ".net"], status: "running" as const, total_count: 913952, completed_count: 504223, available_count: 2110, error_count: 79, batch_id: "b1" },
      { id: "t2", name: "品牌词扫描", tlds: [".io", ".ai"], status: "completed" as const, total_count: 800, completed_count: 800, available_count: 134, error_count: 7, batch_id: "b1" },
      { id: "t3", name: "3字母扫描", tlds: [".net", ".org"], status: "paused" as const, total_count: 35152, completed_count: 10890, available_count: 312, error_count: 15, batch_id: "b1" },
    ],
  },
];

const statusConfig: Record<TaskStatus, { label: string; dotClass: string; badgeClass: string; color: string }> = {
  running:   { label: "运行中", dotClass: "status-dot-running", badgeClass: "badge-green",   color: "text-cyber-green" },
  paused:    { label: "已暂停", dotClass: "status-dot-paused",  badgeClass: "badge-orange",  color: "text-cyber-orange" },
  completed: { label: "已完成", dotClass: "status-dot-completed", badgeClass: "badge-blue",    color: "text-cyber-blue" },
  failed:    { label: "失败",   dotClass: "status-dot-error",   badgeClass: "badge-red",     color: "text-cyber-red" },
  pending:   { label: "等待中", dotClass: "status-dot-idle",    badgeClass: "badge-neutral", color: "text-cyber-muted-dim" },
};

export default function TaskList() {
  const navigate = useNavigate();
  const [search, setSearch] = useState("");
  const [expandedBatches] = useState<Set<string>>(new Set(["b1"]));

  return (
    <div className="space-y-6 animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold neon-text tracking-tight">任务列表</h1>
          <p className="text-sm text-cyber-muted mt-1">管理和监控所有域名扫描任务</p>
        </div>
        <button
          onClick={() => navigate("/tasks/new")}
          className="cyber-btn-primary"
        >
          新建任务
        </button>
      </div>

      {/* Search & Filter Bar */}
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
        <button className="cyber-btn-secondary cyber-btn-sm">
          <Filter className="w-3.5 h-3.5" /> 筛选
        </button>
      </div>

      {/* Batch Groups */}
      <div className="space-y-4">
        {mockBatches.map((batch) => {
          const isExpanded = expandedBatches.has(batch.id);
          return (
            <div key={batch.id} className="glass-panel overflow-hidden">
              {/* Batch Header */}
              <div className="px-5 py-4 flex items-center gap-3 cursor-pointer hover:bg-cyber-card/30 transition-colors border-b border-cyber-border/20 group"
                onClick={() => {/* toggle */}}>
                <ChevronDown className={`w-4 h-4 text-cyber-muted transition-transform ${isExpanded ? "" : "-rotate-90"}`} />
                <span className="text-sm font-semibold text-cyber-text">{batch.name}</span>
                <span className="badge-neutral ml-1">{batch.tasks.length} 任务</span>

                {/* Summary stats */}
                <div className="ml-auto flex items-center gap-4 mr-4">
                  <div className="hidden md:flex items-center gap-4 text-xs">
                    <span className="text-cyber-muted">
                      运行: <strong className="text-cyber-green">{batch.tasks.filter(t => t.status === "running").length}</strong>
                    </span>
                    <span className="text-cyber-border-light">|</span>
                    <span className="text-cyber-muted">
                      完成: <strong className="text-cyber-blue">{batch.tasks.filter(t => t.status === "completed").length}</strong>
                    </span>
                    <span className="text-cyber-border-light">|</span>
                    <span className="text-cyber-muted">
                      可用: <strong className="text-cyber-orange tabular-nums">{batch.tasks.reduce((sum, t) => sum + t.available_count, 0).toLocaleString()}</strong>
                    </span>
                  </div>
                </div>
              </div>

              {/* Tasks */}
              {isExpanded && (
                <div className="divide-y divide-cyber-border/15">
                  {batch.tasks.map((task) => {
                    const cfg = statusConfig[task.status];
                    const progress = task.total_count > 0 ? Math.round((task.completed_count / task.total_count) * 100) : 0;

                    return (
                      <div
                        key={task.id}
                        className="flex items-center gap-4 px-5 py-4 hover:bg-cyber-bg-elevated/40 transition-colors cursor-pointer group/task"
                        onClick={() => navigate(`/tasks/${task.id}`)}
                      >
                        {/* Status Dot */}
                        <span className={`dot ${cfg.dotClass} shrink-0`} />

                        {/* Name + TLDs */}
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-1">
                            <span className="text-sm font-semibold text-cyber-text truncate group-hover/task:text-cyber-green transition-colors">
                              {task.name}
                            </span>
                            <div className="flex items-center gap-1 shrink-0">
                              {task.tlds.slice(0, 3).map((tld) => (
                                <span key={tld} className="badge-neutral text-[11px]">{tld}</span>
                              ))}
                              {task.tlds.length > 3 && (
                                <span className="badge-blue text-[11px]">+{task.tlds.length - 3}</span>
                              )}
                            </div>
                            <span className={`${cfg.badgeClass} text-[11px] shrink-0`}>{cfg.label}</span>
                          </div>
                          {/* Progress bar inline */}
                          <div className="flex items-center gap-3">
                            <div className="progress-bar flex-1 max-w-[280px]">
                              <div className="progress-bar-fill" style={{ width: `${progress}%` }} />
                            </div>
                            <span className="text-xs font-mono text-cyber-muted-dim w-8 tabular-nums">{progress}%</span>
                          </div>
                        </div>

                        {/* Stats row */}
                        <div className="hidden lg:flex items-center gap-6 text-right shrink-0 pl-3 border-l border-cyber-border/20">
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

                        {/* Action hint */}
                        <ExternalLink className="w-4 h-4 text-cyber-border-light opacity-0 group-hover/task:opacity-100 group-hover/task:text-cyber-green transition-all shrink-0" />
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
