import { useState, useEffect } from "react";
import {
  ArrowLeft,
  Play,
  Pause,
  Download,
  Cpu,
  Filter,
  CheckCircle,
  XCircle,
  Clock,
  Terminal,
  ChevronDown,
  Table,
  Inbox,
} from "lucide-react";
import { useNavigate, useParams } from "react-router-dom";
import { useTaskStore } from "../store/taskStore";

const logColors: Record<string, string> = {
  info: "text-cyber-green",
  warn: "text-cyber-orange",
  error: "text-cyber-red",
};

export default function TaskDetail() {
  const { id } = useParams();
  const navigate = useNavigate();
  const [showLogs, setShowLogs] = useState(true);
  const [logFilter, setLogFilter] = useState<string>("all");
  const { tasks, fetchTasks } = useTaskStore();

  useEffect(() => {
    if (tasks.length === 0) fetchTasks();
  }, []);

  const task = tasks.find((t) => t.id === id);

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

  const progress = task.total_count > 0 ? Math.round((task.completed_count / task.total_count) * 100) : 0;

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
          <button className="cyber-btn-secondary cyber-btn-sm"><Pause className="w-3.5 h-3.5" /> 暂停</button>
          <button className="cyber-btn-secondary cyber-btn-sm"><Download className="w-3.5 h-3.5" /> 导出</button>
          <button className="cyber-btn-secondary cyber-btn-sm"><Cpu className="w-3.5 h-3.5" /> 向量化</button>
          <button className="cyber-btn-secondary cyber-btn-sm"><Filter className="w-3.5 h-3.5" /> 筛选</button>
        </div>
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
            <p className="text-xl font-bold text-cyber-text tabular-nums">{task.completed_count.toLocaleString()}</p>
            <p className="text-[10px] text-cyber-muted-dim font-mono tabular-nums">/ {task.total_count.toLocaleString()}</p>
          </div>
          <div className="rounded-xl bg-cyber-bg-elevated/60 border border-cyber-border/20 p-4 space-y-1">
            <p className="text-xs text-cyber-muted">可用域名</p>
            <p className="text-xl font-bold text-cyber-green tabular-nums">{task.available_count.toLocaleString()}</p>
            <p className="text-[10px] text-cyber-muted-dim">
              {task.completed_count > 0 ? ((task.available_count / task.completed_count) * 100).toFixed(1) : "0.0"}% 可用率
            </p>
          </div>
          <div className="rounded-xl bg-cyber-bg-elevated/60 border border-cyber-border/20 p-4 space-y-1">
            <p className="text-xs text-cyber-muted">错误数</p>
            <p className="text-xl font-bold text-cyber-red tabular-nums">{task.error_count}</p>
            <p className="text-[10px] text-cyber-muted-dim">
              {task.completed_count > 0 ? ((task.error_count / task.completed_count) * 100).toFixed(2) : "0.00"}% 错误率
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
        </div>
        <div className="text-center py-12 text-cyber-muted">
          <Table className="w-10 h-10 mx-auto mb-3 opacity-40" />
          <p className="text-sm">暂无扫描结果</p>
          <p className="text-xs text-cyber-muted-dim mt-1">任务运行后结果将在此显示</p>
        </div>
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
            <div className="text-center py-8 text-cyber-muted">
              <Terminal className="w-8 h-8 mx-auto mb-2 opacity-30" />
              <p className="text-xs">任务运行后日志将在此显示</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
