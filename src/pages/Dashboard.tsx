import {
  Activity,
  CheckCircle2,
  Globe,
  Shield,
  Zap,
  ArrowRight,
  TrendingUp,
  Clock,
} from "lucide-react";
import { useNavigate } from "react-router-dom";

const stats = [
  {
    label: "运行中",
    value: "3",
    icon: Activity,
    color: "from-cyber-green to-cyber-cyan",
    textColor: "text-cyber-green",
    shadow: "shadow-neon",
  },
  {
    label: "已完成",
    value: "24",
    icon: CheckCircle2,
    color: "from-cyber-blue to-purple-500",
    textColor: "text-cyber-blue",
    shadow: "shadow-neon-blue",
  },
  {
    label: "可用域名",
    value: "1,847",
    icon: Globe,
    color: "from-cyber-orange to-amber-400",
    textColor: "text-cyber-orange",
    shadow: "shadow-neon-cyan",
  },
  {
    label: "代理在线",
    value: "8/10",
    icon: Shield,
    color: "from-cyber-cyan to-teal-400",
    textColor: "text-cyber-cyan",
    shadow: "shadow-neon-cyan",
  },
];

const recentTasks = [
  {
    id: "1",
    name: "4字母 .com 扫描",
    tld: ".com",
    status: "running" as const,
    progress: 67,
    available: 234,
    total: 456976,
  },
  {
    id: "2",
    name: "AI 相关 .io 扫描",
    tld: ".io",
    status: "running" as const,
    progress: 42,
    available: 89,
    total: 500,
  },
  {
    id: "3",
    name: "3字母 .net 扫描",
    tld: ".net",
    status: "paused" as const,
    progress: 31,
    available: 156,
    total: 17576,
  },
  {
    id: "4",
    name: "品牌词 .com 扫描",
    tld: ".com",
    status: "completed" as const,
    progress: 100,
    available: 412,
    total: 2000,
  },
  {
    id: "5",
    name: "短域名 .org 扫描",
    tld: ".org",
    status: "completed" as const,
    progress: 100,
    available: 567,
    total: 676,
  },
];

const statusConfig = {
  running: { label: "运行中", color: "text-cyber-green", bg: "bg-cyber-green/10", dot: "bg-cyber-green animate-pulse" },
  paused: { label: "已暂停", color: "text-cyber-orange", bg: "bg-cyber-orange/10", dot: "bg-cyber-orange" },
  completed: { label: "已完成", color: "text-cyber-blue", bg: "bg-cyber-blue/10", dot: "bg-cyber-blue" },
  pending: { label: "等待中", color: "text-cyber-muted", bg: "bg-cyber-muted/10", dot: "bg-cyber-muted" },
};

export default function Dashboard() {
  const navigate = useNavigate();

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold neon-text">仪表盘</h1>
          <p className="text-sm text-cyber-muted mt-1">域名扫描任务总览</p>
        </div>
        <button
          onClick={() => navigate("/tasks/new")}
          className="cyber-btn-primary flex items-center gap-2"
        >
          <Zap className="w-4 h-4" />
          新建扫描
        </button>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-4 gap-4">
        {stats.map((stat) => (
          <div
            key={stat.label}
            className={`glass-panel p-5 ${stat.shadow} hover:scale-[1.02] transition-transform duration-200`}
          >
            <div className="flex items-center justify-between mb-3">
              <span className="text-xs font-medium text-cyber-muted uppercase tracking-wider">
                {stat.label}
              </span>
              <div className={`w-8 h-8 rounded-lg bg-gradient-to-br ${stat.color} flex items-center justify-center`}>
                <stat.icon className="w-4 h-4 text-cyber-bg" />
              </div>
            </div>
            <p className={`text-3xl font-bold ${stat.textColor}`}>{stat.value}</p>
          </div>
        ))}
      </div>

      {/* Main Content */}
      <div className="grid grid-cols-3 gap-6">
        {/* Recent Tasks */}
        <div className="col-span-2 glass-panel p-5">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-base font-semibold text-cyber-text flex items-center gap-2">
              <Clock className="w-4 h-4 text-cyber-green" />
              最近任务
            </h2>
            <button
              onClick={() => navigate("/tasks")}
              className="text-xs text-cyber-muted hover:text-cyber-green transition-colors flex items-center gap-1"
            >
              查看全部 <ArrowRight className="w-3 h-3" />
            </button>
          </div>
          <div className="space-y-3">
            {recentTasks.map((task) => {
              const cfg = statusConfig[task.status];
              return (
                <div
                  key={task.id}
                  className="flex items-center gap-4 p-3 rounded-lg bg-cyber-bg/50 hover:bg-cyber-card/40 transition-colors cursor-pointer"
                  onClick={() => navigate(`/tasks/${task.id}`)}
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className={`w-1.5 h-1.5 rounded-full ${cfg.dot}`} />
                      <span className="text-sm font-medium text-cyber-text truncate">
                        {task.name}
                      </span>
                      <span className="text-xs text-cyber-muted px-1.5 py-0.5 rounded bg-cyber-surface">
                        {task.tld}
                      </span>
                    </div>
                    <div className="flex items-center gap-3">
                      <div className="flex-1 h-1.5 rounded-full bg-cyber-surface overflow-hidden">
                        <div
                          className="h-full rounded-full bg-gradient-to-r from-cyber-green to-cyber-cyan transition-all duration-500"
                          style={{ width: `${task.progress}%` }}
                        />
                      </div>
                      <span className="text-xs text-cyber-muted w-10 text-right">
                        {task.progress}%
                      </span>
                    </div>
                  </div>
                  <div className="text-right">
                    <p className="text-sm font-semibold text-cyber-green">{task.available}</p>
                    <p className="text-[10px] text-cyber-muted">可用</p>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Quick Actions */}
        <div className="space-y-4">
          <div className="glass-panel p-5">
            <h2 className="text-base font-semibold text-cyber-text mb-4 flex items-center gap-2">
              <TrendingUp className="w-4 h-4 text-cyber-cyan" />
              快捷操作
            </h2>
            <div className="space-y-2">
              <button
                onClick={() => navigate("/tasks/new")}
                className="w-full p-3 rounded-lg bg-cyber-bg/50 hover:bg-cyber-green/10 border border-cyber-border/30 hover:border-cyber-green/30 transition-all text-left group"
              >
                <p className="text-sm font-medium text-cyber-text group-hover:text-cyber-green transition-colors">
                  新建扫描任务
                </p>
                <p className="text-xs text-cyber-muted mt-0.5">正则/通配符/LLM/手动</p>
              </button>
              <button
                onClick={() => navigate("/proxies")}
                className="w-full p-3 rounded-lg bg-cyber-bg/50 hover:bg-cyber-cyan/10 border border-cyber-border/30 hover:border-cyber-cyan/30 transition-all text-left group"
              >
                <p className="text-sm font-medium text-cyber-text group-hover:text-cyber-cyan transition-colors">
                  管理代理
                </p>
                <p className="text-xs text-cyber-muted mt-0.5">配置代理轮转</p>
              </button>
              <button
                onClick={() => navigate("/vectorize")}
                className="w-full p-3 rounded-lg bg-cyber-bg/50 hover:bg-cyber-blue/10 border border-cyber-border/30 hover:border-cyber-blue/30 transition-all text-left group"
              >
                <p className="text-sm font-medium text-cyber-text group-hover:text-cyber-blue transition-colors">
                  向量化处理
                </p>
                <p className="text-xs text-cyber-muted mt-0.5">GPU 加速语义搜索</p>
              </button>
            </div>
          </div>

          {/* GPU Status Card */}
          <div className="glass-panel p-5">
            <h2 className="text-base font-semibold text-cyber-text mb-3 flex items-center gap-2">
              <Cpu className="w-4 h-4 text-cyber-green" />
              GPU 状态
            </h2>
            <div className="space-y-2">
              <div className="flex justify-between text-xs">
                <span className="text-cyber-muted">后端</span>
                <span className="text-cyber-green font-medium">CPU (无 GPU)</span>
              </div>
              <div className="flex justify-between text-xs">
                <span className="text-cyber-muted">模型</span>
                <span className="text-cyber-text">MiniLM-L6-v2</span>
              </div>
              <div className="flex justify-between text-xs">
                <span className="text-cyber-muted">维度</span>
                <span className="text-cyber-text">384</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
