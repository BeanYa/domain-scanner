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
  PanelLeftClose,
  PanelLeftOpen,
} from "lucide-react";

interface SidebarProps {
  collapsed: boolean;
  onToggle: () => void;
}

const navItems = [
  { to: "/dashboard", icon: LayoutDashboard, label: "仪表盘", shortcut: "⌘1" },
  { to: "/tasks", icon: ListTodo, label: "任务列表", shortcut: "⌘2" },
  { to: "/tasks/new", icon: PlusCircle, label: "新建任务", shortcut: "⌘N" },
  { to: "/filter", icon: Filter, label: "二次筛选", shortcut: null },
  { to: "/vectorize", icon: Cpu, label: "向量化", shortcut: null },
  { to: "/proxies", icon: Shield, label: "代理管理", shortcut: null },
  { to: "/settings", icon: Settings, label: "设置", shortcut: "," },
];

export default function Sidebar({ collapsed, onToggle }: SidebarProps) {
  const location = useLocation();

  return (
    <aside
      className={`fixed left-0 top-0 h-screen z-50 flex flex-col transition-all duration-300 ease-out ${
        collapsed ? "w-[72px]" : "w-[240px]"
      } bg-cyber-surface/90 backdrop-blur-2xl border-r border-cyber-border/30`}
    >
      {/* Logo Area */}
      <div className="h-16 px-4 flex items-center gap-3 border-b border-cyber-border/20">
        <div className="w-9 h-9 shrink-0 rounded-xl bg-gradient-to-br from-cyber-green to-cyber-cyan flex items-center justify-center shadow-neon">
          <Radar className="w-[18px] h-[18px] text-cyber-bg" />
        </div>
        {!collapsed && (
          <div className="overflow-hidden animate-fade-in">
            <h1 className="text-sm font-bold text-cyber-text tracking-wide leading-none">DomainScan</h1>
            <p className="text-[10px] text-cyber-muted tracking-widest uppercase mt-0.5">Scanner Pro</p>
          </div>
        )}
      </div>

      {/* Collapse Toggle */}
      <button
        onClick={onToggle}
        className="absolute -right-3 top-20 w-6 h-6 rounded-full bg-cyber-card border border-cyber-border/50 
                 flex items-center justify-center shadow-glass-sm hover:bg-cyber-card-hover hover:border-cyber-green/30 
                 transition-all duration-200 cursor-pointer group"
      >
        {collapsed ? (
          <PanelLeftOpen className="w-3.5 h-3.5 text-cyber-muted group-hover:text-cyber-green transition-colors" />
        ) : (
          <PanelLeftClose className="w-3.5 h-3.5 text-cyber-muted group-hover:text-cyber-green transition-colors" />
        )}
      </button>

      {/* Navigation Links */}
      <nav className="flex-1 px-3 py-4 space-y-1 overflow-y-auto no-scrollbar">
        {navItems.map(({ to, icon: Icon, label, shortcut }) => {
          const isActive =
            location.pathname === to ||
            (to === "/tasks" && location.pathname.startsWith("/tasks/") && location.pathname !== "/tasks/new");

          return (
            <NavLink
              key={to}
              to={to}
              title={collapsed ? label : undefined}
              className={`
                group flex items-center ${collapsed ? "justify-center px-2" : "gap-3 px-3"}
                py-2.5 rounded-xl text-sm font-medium transition-all duration-200 relative
                ${
                  isActive
                    ? "bg-gradient-to-r from-cyber-green/12 to-transparent text-cyber-green"
                    : "text-cyber-muted hover:text-cyber-text-secondary hover:bg-cyber-card/50"
                }
              `}
            >
              {/* Active background glow */}
              {isActive && !collapsed && (
                <span className="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-5 rounded-full bg-cyber-green shadow-neon" />
              )}

              <Icon
                className={`shrink-0 transition-all duration-200 ${
                  collapsed ? "w-5 h-5" : "w-[18px] h-[18px]"
                } ${
                  isActive ? "text-cyber-green" : "text-cyber-muted-dim group-hover:text-cyber-text-secondary"
                }`}
              />

              {!collapsed && (
                <>
                  <span className="flex-1 truncate">{label}</span>
                  {shortcut && (
                    <span className={`text-[10px] px-1.5 py-0.5 rounded-md font-mono ${
                      isActive ? "bg-cyber-green/15 text-cyber-green-dim" : "bg-cyber-surface text-cyber-muted-dim opacity-0 group-hover:opacity-100 transition-opacity"
                    }`}>
                      {shortcut}
                    </span>
                  )}
                </>
              )}

              {/* Collapsed active indicator (dot) */}
              {collapsed && isActive && (
                <span className="absolute top-1 right-1.5 w-1.5 h-1.5 rounded-full bg-cyber-green animate-pulse" />
              )}
            </NavLink>
          );
        })}
      </nav>

      {/* Bottom Section */}
      <div className="px-4 py-4 border-t border-cyber-border/20 space-y-3">
        {/* System Status */}
        <div className={`flex items-center gap-2 ${collapsed ? "justify-center" : ""}`}>
          <span className="status-dot-running w-2 h-2" />
          {!collapsed && (
            <>
              <span className="text-xs text-cyber-muted">系统就绪</span>
              <span className="ml-auto text-[10px] text-cyber-muted-dim font-mono">v0.1</span>
            </>
          )}
        </div>

        {/* Version / Build info - only when expanded */}
        {!collapsed && (
          <div className="text-[10px] text-cyber-muted-dim/60 leading-relaxed">
            Tauri 2.0 &middot; Rust + React
          </div>
        )}
      </div>
    </aside>
  );
}
