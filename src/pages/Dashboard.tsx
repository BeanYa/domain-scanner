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
} from "lucide-react";
import { useNavigate } from "react-router-dom";

const stats = [
  {
    label: "运行中",
    value: "3",
    icon: Activity,
    gradient: "from-cyber-green to-cyber-green-dim",
    textColor: "text-cyber-green",
    shadowClass: "shadow-neon",
    change: "+1 较昨日",
    changeType: "positive" as const,
    miniBars: [60, 80, 45, 90, 70, 85, 75],
  },
  {
    label: "已完成",
    value: "24",
    icon: CheckCircle2,
    gradient: "from-cyber-blue to-cyber-purple",
    textColor: "text-cyber-blue",
    shadowClass: "shadow-neon-blue",
    change: "+5 本周",
    changeType: "positive" as const,
    miniBars: [30, 45, 55, 50, 70, 65, 80],
  },
  {
    label: "可用域名",
    value: "1,847",
    icon: Globe,
    gradient: "from-cyber-orange to-amber-400",
    textColor: "text-cyber-orange",
    shadowClass: "shadow-neon-orange",
    change: "+127 本月",
    changeType: "positive" as const,
    miniBars: [20, 35, 40, 55, 48, 62, 70],
  },
  {
    label: "代理在线",
    value: "8/10",
    icon: Shield,
    gradient: "from-cyber-cyan to-teal-400",
    textColor: "text-cyber-cyan",
    shadowClass: "shadow-neon-cyan",
    change: "-1 离线",
    changeType: "negative" as const,
    miniBars: [90, 90, 88, 92, 85, 80, 80],
  },
];

const recentTasks = [
  { id: "1", name: "4字母扫描", tlds: [".com", ".net"], status: "running" as const, progress: 67, available: 234, total: 913952, speed: "~2.4k/s" },
  { id: "2", name: "AI 相关扫描", tlds: [".io"], status: "running" as const, progress: 42, available: 89, total: 500, speed: "~120/s" },
  { id: "3", name: "3字母扫描", tlds: [".net"], status: "paused" as const, progress: 31, available: 156, total: 17576, speed: "-" },
  { id: "4", name: "品牌词扫描", tlds: [".com"], status: "completed" as const, progress: 100, available: 412, total: 2000, speed: "-" },
  { id: "5", name: "短域名扫描", tlds: [".org", ".io", ".dev"], status: "completed" as const, progress: 100, available: 567, total: 2028, speed: "-" },
];

const statusConfig = {
  running:   { label: "运行中", dotClass: "status-dot-running", btnIcon: Pause, btnLabel: "暂停", badgeClass: "badge-green" },
  paused:    { label: "已暂停", dotClass: "status-dot-paused",  btnIcon: Play, btnLabel: "继续", badgeClass: "badge-orange" },
  completed: { label: "已完成", dotClass: "status-dot-completed", btnIcon: ExternalLink, btnLabel: "查看", badgeClass: "badge-blue" },
  pending:    { label: "等待中", dotClass: "status-dot-idle", btnIcon: Play, btnLabel: "启动", badgeClass: "badge-neutral" },
};

const quickActions = [
  { icon: Zap, label: "新建扫描任务", desc: "正则 / 通配符 / LLM / 手动输入", to: "/tasks/new", color: "group-hover:text-cyber-green hover:border-cyber-green/25" },
  { icon: Shield, label: "管理代理池", desc: "配置代理轮转策略与健康检查", to: "/proxies", color: "group-hover:text-cyber-cyan hover:border-cyber-cyan/25" },
  { icon: Cpu, label: "向量化处理", desc: "GPU 加速语义搜索与智能过滤", to: "/vectorize", color: "group-hover:text-cyber-blue hover:border-cyber-blue/25" },
];

/* Mini bar chart component for stat cards */
function MiniBarChart({ data }: { data: number[] }) {
  return (
    <div className="flex items-end gap-[3px] h-7">
      {data.map((v, i) => (
        <div
          key={i}
          className="flex-1 rounded-sm bg-current opacity-15 transition-all duration-300"
          style={{ height: `${Math.max(10, v)}%` }}
        />
      ))}
      <div
        className="flex-1 rounded-sm bg-current opacity-50 transition-all duration-500"
        style={{ height: `${Math.max(10, data[data.length - 1])}%` }}
      />
    </div>
  );
}

export default function Dashboard() {
  const navigate = useNavigate();

  return (
    <div className="space-y-6 animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold neon-text tracking-tight">仪表盘</h1>
          <p className="text-sm text-cyber-muted mt-1">域名扫描总览与系统状态</p>
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

      {/* Stats Grid */}
      <div className="grid grid-cols-4 gap-4">
        {stats.map((stat) => (
          <div key={stat.label} className={`stat-card group ${stat.shadowClass}`} style={{
            ['--tw-gradient-from' as string]: undefined,
          }}>
            <div className="flex items-start justify-between mb-3">
              <span className="text-xs font-medium text-cyber-muted uppercase tracking-wider">{stat.label}</span>
              <div className={`w-9 h-9 rounded-xl bg-gradient-to-br ${stat.gradient} flex items-center justify-center`}>
                <stat.icon className="w-4.5 h-4.5 text-white/90" />
              </div>
            </div>
            <p className={`text-3xl font-bold tracking-tight ${stat.textColor}`}>{stat.value}</p>

            {/* Mini bar chart */}
            <div className={`${stat.textColor} mt-3`}>
              <MiniBarChart data={stat.miniBars} />
            </div>

            {/* Change indicator */}
            <div className={`flex items-center gap-1 mt-2 text-xs font-medium ${
              stat.changeType === "positive" ? "text-cyber-green" : "text-cyber-orange"
            }`}>
              <span>{stat.change}</span>
            </div>
          </div>
        ))}
      </div>

      {/* Main Content Area */}
      <div className="grid grid-cols-3 gap-6">
        {/* Recent Tasks Panel (2 cols wide) */}
        <div className="col-span-2 glass-panel p-5 space-y-4">
          <div className="section-header">
            <h2 className="section-title">
              <Clock className="w-4 h-4 text-cyber-green" />
              最近任务
            </h2>
            <button
              onClick={() => navigate("/tasks")}
              className="cyber-btn-ghost cyber-btn-sm text-cyber-muted-dim hover:text-cyber-green group"
            >
              查看全部
              <ArrowRight className="w-3 h-3 transition-transform group-hover:translate-x-0.5" />
            </button>
          </div>

          {/* Task List */}
          <div className="space-y-2.5">
            {recentTasks.map((task) => {
              const cfg = statusConfig[task.status];
              const BtnIcon = cfg.btnIcon;
              return (
                <div
                  key={task.id}
                  className="task-card group"
                  onClick={() => navigate(`/tasks/${task.id}`)}
                >
                  <div className="flex items-start justify-between gap-4">
                    {/* Left: Info */}
                    <div className="flex-1 min-w-0">
                      {/* Name row + TLDs + Status + Action */}
                      <div className="flex items-center gap-2 mb-2">
                        <span className={`dot ${cfg.dotClass} shrink-0`} />
                        <span className="text-sm font-semibold text-cyber-text truncate">{task.name}</span>

                        {/* TLD badges */}
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

                      {/* Progress bar */}
                      <div className="flex items-center gap-3">
                        <div className="flex-1 progress-bar max-w-[240px]">
                          <div
                            className="progress-bar-fill"
                            style={{ width: `${task.progress}%` }}
                          />
                        </div>
                        <span className="text-xs font-mono text-cyber-muted w-9 text-right tabular-nums">{task.progress}%</span>
                        <span className="text-[11px] text-cyber-muted-dim w-14 text-right tabular-nums hidden sm:block">{task.speed}</span>
                      </div>
                    </div>

                    {/* Right: Available count + Quick action */}
                    <div className="text-right shrink-0 pl-2 border-l border-cyber-border/20 min-w-[72px]">
                      <p className="text-lg font-bold text-cyber-green tabular-nums">{task.available.toLocaleString()}</p>
                      <p className="text-[10px] text-cyber-muted-dim">可用域名</p>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Right Sidebar */}
        <div className="space-y-4">
          {/* Quick Actions Card */}
          <div className="glass-panel p-5">
            <h2 className="section-title mb-4">
              <TrendingUp className="w-4 h-4 text-cyber-cyan-bright" />
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

          {/* GPU / System Status Card */}
          <div className="glass-panel p-5 space-y-4">
            <h2 className="section-title">
              <Cpu className="w-4 h-4 text-cyber-green" />
              系统状态
            </h2>

            {/* GPU Info */}
            <div className="rounded-xl bg-cyber-bg-elevated/60 p-3.5 space-y-2.5">
              <div className="flex items-center justify-between">
                <span className="text-xs text-cyber-muted">推理后端</span>
                <span className="text-xs font-semibold text-cyber-orange px-2 py-0.5 rounded-md bg-cyber-orange/8">CPU</span>
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

              <div className="flex items-center gap-2 p-2 rounded-lg bg-cyber-green/4 border border-cyber-green/12">
                <Cpu className="w-3.5 h-3.5 text-cyber-green/70" />
                <span className="text-[11px] text-cyber-green/80">启用 --features gpu-directml 以使用 AMD GPU 加速</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
