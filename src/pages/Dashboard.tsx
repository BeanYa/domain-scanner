import { useEffect } from "react";
import {
  Activity,
  CheckCircle2,
  Globe,
  Shield,
  Zap,
  ArrowRight,
  TrendingUp,
  Clock,
  Cpu,
  Play,
  Pause,
  ExternalLink,
  ChevronRight,
  Inbox,
} from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useGpuStore } from "../store/gpuStore";
import { useTaskStore } from "../store/taskStore";
import { useProxyStore } from "../store/proxyStore";

const statusConfig = {
  running:   { label: "运行中", dotClass: "status-dot-running", btnIcon: Pause, btnLabel: "暂停", badgeClass: "badge-green" },
  paused:    { label: "已暂停", dotClass: "status-dot-paused",  btnIcon: Play, btnLabel: "继续", badgeClass: "badge-orange" },
  stopped:   { label: "已停止", dotClass: "status-dot-idle", btnIcon: Play, btnLabel: "重新开始", badgeClass: "badge-red" },
  completed: { label: "已完成", dotClass: "status-dot-completed", btnIcon: ExternalLink, btnLabel: "查看", badgeClass: "badge-blue" },
  pending:   { label: "等待中", dotClass: "status-dot-idle", btnIcon: Play, btnLabel: "启动", badgeClass: "badge-neutral" },
};

const quickActions = [
  { icon: Zap, label: "新建扫描任务", desc: "正则 / 通配符 / LLM / 手动输入", to: "/tasks/new", color: "group-hover:text-cyber-green hover:border-cyber-green/25" },
  { icon: Shield, label: "管理代理池", desc: "配置代理轮转策略与健康检查", to: "/proxies", color: "group-hover:text-cyber-cyan hover:border-cyber-cyan/25" },
  { icon: Cpu, label: "向量化处理", desc: "GPU 加速语义搜索与智能过滤", to: "/vectorize", color: "group-hover:text-cyber-blue hover:border-cyber-blue/25" },
];

export default function Dashboard() {
  const navigate = useNavigate();
  const { status: gpuStatus, fetchStatus: fetchGpuStatus } = useGpuStore();
  const { tasks, fetchTasks } = useTaskStore();
  const { proxies, fetchProxies } = useProxyStore();

  useEffect(() => {
    fetchGpuStatus();
    fetchTasks();
    fetchProxies();
  }, []);

  const runningCount = tasks.filter(t => t.status === "running").length;
  const completedCount = tasks.filter(t => t.status === "completed").length;
  const availableCount = tasks.reduce((sum, t) => sum + t.available_count, 0);
  const activeProxies = proxies.filter(p => p.is_active).length;
  const totalProxies = proxies.length;

  const stats = [
    {
      label: "运行中",
      value: String(runningCount),
      icon: Activity,
      textColor: "text-cyber-green",
    },
    {
      label: "已完成",
      value: String(completedCount),
      icon: CheckCircle2,
      textColor: "text-cyber-blue",
    },
    {
      label: "可用域名",
      value: availableCount.toLocaleString(),
      icon: Globe,
      textColor: "text-cyber-orange",
    },
    {
      label: "代理在线",
      value: totalProxies > 0 ? `${activeProxies}/${totalProxies}` : "0",
      icon: Shield,
      textColor: "text-cyber-cyan",
    },
  ];

  const recentTasks = tasks.slice(0, 5).map(t => ({
    id: t.id,
    name: t.name,
    tlds: t.tlds,
    status: t.status as keyof typeof statusConfig,
    progress: t.total_count > 0 ? Math.round((t.completed_count / t.total_count) * 100) : 0,
    available: t.available_count,
    total: t.total_count,
  }));

  const gpuBackendLabel = gpuStatus?.available
    ? gpuStatus.backend === "cuda" ? "CUDA"
    : gpuStatus.backend === "directml" ? "DirectML"
    : gpuStatus.backend === "cpu" ? "CPU"
    : gpuStatus.backend
    : "CPU";
  const gpuDeviceName = gpuStatus?.device_name || "CPU Only";

  return (
    <div className="page-shell">
      <div className="editorial-panel p-6 lg:p-8 min-h-[220px] flex items-end justify-between gap-6">
        <div>
          <div className="eyebrow mb-4">CONTROL ROOM</div>
          <h1 className="page-heading">Domain Scanner</h1>
          <p className="page-subtitle">域名扫描总览、代理池状态与向量化能力集中监控。界面保持低干扰，让任务数据成为主视觉。</p>
        </div>
        <button
          onClick={() => navigate("/tasks/new")}
          className="cyber-btn-primary"
        >
          <Zap className="w-4 h-4" />
          新建扫描
          <ChevronRight className="w-3.5 h-3.5" />
        </button>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
        {stats.map((stat) => (
          <div key={stat.label} className="stat-card group">
            <div className="flex items-start justify-between mb-3">
              <span className="eyebrow">{stat.label}</span>
              <div className="w-9 h-9 rounded-md border border-cyber-border bg-cyber-surface flex items-center justify-center">
                <stat.icon className="w-4 h-4 text-cyber-text-secondary" />
              </div>
            </div>
            <p className={`text-3xl font-normal leading-none tabular-nums ${stat.textColor}`}>{stat.value}</p>
          </div>
        ))}
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        <div className="xl:col-span-2 glass-panel p-5 space-y-4">
          <div className="section-header">
            <h2 className="section-title">
              <Clock className="w-4 h-4 text-cyber-text-secondary" />
              最近任务
            </h2>
            <button
              onClick={() => navigate("/tasks")}
              className="cyber-btn-ghost cyber-btn-sm text-cyber-muted-dim hover:text-white group"
            >
              查看全部
              <ArrowRight className="w-3 h-3 transition-transform group-hover:translate-x-0.5" />
            </button>
          </div>

          <div className="space-y-2.5">
            {recentTasks.length === 0 ? (
              <div className="text-center py-12 text-cyber-muted">
                <Inbox className="w-10 h-10 mx-auto mb-3 opacity-40" />
                <p className="text-sm">暂无任务</p>
                <p className="text-xs text-cyber-muted-dim mt-1">点击「新建扫描」创建第一个扫描任务</p>
              </div>
            ) : (
              recentTasks.map((task) => {
                const cfg = statusConfig[task.status] || statusConfig.pending;
                return (
                  <div
                    key={task.id}
                    className="task-card group"
                    onClick={() => navigate(`/tasks/${task.id}`)}
                  >
                    <div className="flex items-start justify-between gap-4">
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-2">
                          <span className={`dot ${cfg.dotClass} shrink-0`} />
                          <span className="text-sm font-semibold text-cyber-text truncate">{task.name}</span>
                          <div className="flex items-center gap-1 shrink-0">
                            {task.tlds.slice(0, 3).map((tld) => (
                              <span key={tld} className="badge-neutral text-[11px]">{tld}</span>
                            ))}
                            {task.tlds.length > 3 && (
                              <span className="badge-blue text-[11px]">+{task.tlds.length - 3}</span>
                            )}
                          </div>
                          <span className={`${cfg.badgeClass} text-[11px] ml-auto shrink-0`}>{cfg.label}</span>
                        </div>
                        <div className="flex items-center gap-3">
                          <div className="flex-1 progress-bar max-w-[240px]">
                            <div className="progress-bar-fill" style={{ width: `${task.progress}%` }} />
                          </div>
                          <span className="text-xs font-mono text-cyber-muted w-9 text-right tabular-nums">{task.progress}%</span>
                        </div>
                      </div>
                      <div className="text-right shrink-0 pl-2 border-l border-cyber-border min-w-[72px]">
                        <p className="text-lg font-normal text-cyber-green tabular-nums">{task.available.toLocaleString()}</p>
                        <p className="text-[10px] text-cyber-muted-dim">可用域名</p>
                      </div>
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>

        <div className="space-y-4">
          <div className="glass-panel p-5">
            <h2 className="section-title mb-4">
              <TrendingUp className="w-4 h-4 text-cyber-text-secondary" />
              快捷操作
            </h2>
            <div className="space-y-2">
              {quickActions.map(({ icon: Icon, label, desc, to, color }) => (
                <button
                  key={to}
                  onClick={() => navigate(to)}
                  className={`w-full p-3.5 rounded-xl bg-cyber-bg-elevated/60 border border-cyber-border/25
                           text-left group transition-all duration-200 hover:bg-cyber-surface/60 ${color}`}
                >
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-lg bg-cyber-card border border-cyber-border/20
                                flex items-center justify-center shrink-0
                                group-hover:border-current/20 transition-colors">
                      <Icon className="w-4 h-4 text-cyber-muted group-hover:text-inherit transition-colors" />
                    </div>
                    <div>
                      <p className="text-sm font-medium text-cyber-text group-hover:text-inherit transition-colors leading-snug">{label}</p>
                      <p className="text-[11px] text-cyber-muted-dim mt-0.5">{desc}</p>
                    </div>
                    <ChevronRight className="w-3.5 h-3.5 ml-auto text-cyber-border-light opacity-0 group-hover:opacity-100 transition-opacity" />
                  </div>
                </button>
              ))}
            </div>
          </div>

          <div className="glass-panel p-5 space-y-4">
            <h2 className="section-title">
              <Cpu className="w-4 h-4 text-cyber-text-secondary" />
              系统状态
            </h2>

            <div className="rounded-md bg-cyber-surface p-3.5 space-y-2.5 border border-cyber-border">
              <div className="flex items-center justify-between">
                <span className="text-xs text-cyber-muted">推理后端</span>
                <span className={`text-xs font-semibold px-2 py-0.5 rounded-md ${
                  gpuStatus?.available && gpuStatus.backend !== "cpu"
                    ? "text-cyber-green bg-cyber-green/8"
                    : "text-cyber-orange bg-cyber-orange/8"
                }`}>
                  {gpuBackendLabel}
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-xs text-cyber-muted">设备</span>
                <span className="text-xs font-mono text-cyber-text-secondary">{gpuDeviceName}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-xs text-cyber-muted">Embedding 模型</span>
                <span className="text-xs font-mono text-cyber-text-secondary">MiniLM-L6-v2</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-xs text-cyber-muted">向量维度</span>
                <span className="text-xs font-mono text-cyber-text-secondary">384-d</span>
              </div>

              <div className="divider my-1" />

              {gpuStatus?.available && gpuStatus.backend !== "cpu" ? (
                <div className="flex items-center gap-2 p-2 rounded-md bg-white/[0.04] border border-white/10">
                  <Cpu className="w-3.5 h-3.5 text-cyber-green/70" />
                  <span className="text-[11px] text-cyber-green/80">GPU 加速已启用</span>
                </div>
              ) : (
                <div className="flex items-center gap-2 p-2 rounded-md bg-cyber-orange/10 border border-cyber-orange/20">
                  <Cpu className="w-3.5 h-3.5 text-cyber-orange/70" />
                  <span className="text-[11px] text-cyber-orange/80">CPU 模式运行中，可在设置中配置 GPU</span>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
