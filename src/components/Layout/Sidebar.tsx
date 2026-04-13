import { NavLink, useLocation } from "react-router-dom";
import {
  LayoutDashboard,
  ListTodo,
  PlusCircle,
  Filter,
  Cpu,
  Shield,
  Settings,
  Radar,
} from "lucide-react";

const navItems = [
  { to: "/dashboard", icon: LayoutDashboard, label: "仪表盘" },
  { to: "/tasks", icon: ListTodo, label: "任务列表" },
  { to: "/tasks/new", icon: PlusCircle, label: "新建任务" },
  { to: "/filter", icon: Filter, label: "二次筛选" },
  { to: "/vectorize", icon: Cpu, label: "向量化" },
  { to: "/proxies", icon: Shield, label: "代理管理" },
  { to: "/settings", icon: Settings, label: "设置" },
];

export default function Sidebar() {
  const location = useLocation();

  return (
    <aside className="fixed left-0 top-0 h-screen w-[220px] bg-cyber-surface/80 backdrop-blur-xl border-r border-cyber-border/50 flex flex-col z-50">
      {/* Logo */}
      <div className="px-5 py-6 flex items-center gap-3">
        <div className="w-9 h-9 rounded-lg bg-gradient-to-br from-cyber-green to-cyber-cyan flex items-center justify-center shadow-neon">
          <Radar className="w-5 h-5 text-cyber-bg" />
        </div>
        <div>
          <h1 className="text-sm font-bold text-cyber-text tracking-wide">
            DomainScan
          </h1>
          <p className="text-[10px] text-cyber-muted tracking-widest uppercase">
            Scanner Pro
          </p>
        </div>
      </div>

      {/* Nav Links */}
      <nav className="flex-1 px-3 py-2 space-y-1">
        {navItems.map(({ to, icon: Icon, label }) => {
          const isActive =
            location.pathname === to ||
            (to === "/tasks" && location.pathname.startsWith("/tasks/") && location.pathname !== "/tasks/new");

          return (
            <NavLink
              key={to}
              to={to}
              className={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200 group
                ${
                  isActive
                    ? "bg-cyber-green/10 text-cyber-green shadow-neon"
                    : "text-cyber-muted hover:text-cyber-text hover:bg-cyber-card/60"
                }
              `}
            >
              <Icon
                className={`w-[18px] h-[18px] transition-colors ${
                  isActive ? "text-cyber-green" : "text-cyber-muted group-hover:text-cyber-text"
                }`}
              />
              <span>{label}</span>
              {isActive && (
                <div className="ml-auto w-1.5 h-1.5 rounded-full bg-cyber-green animate-pulse" />
              )}
            </NavLink>
          );
        })}
      </nav>

      {/* Bottom Status */}
      <div className="px-4 py-4 border-t border-cyber-border/40">
        <div className="flex items-center gap-2 text-xs text-cyber-muted">
          <div className="w-2 h-2 rounded-full bg-cyber-green animate-pulse-slow" />
          <span>系统就绪</span>
        </div>
      </div>
    </aside>
  );
}
