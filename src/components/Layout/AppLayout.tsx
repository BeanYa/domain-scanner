import { useState } from "react";
import { Outlet, useLocation } from "react-router-dom";
import Sidebar from "./Sidebar";

export default function AppLayout() {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  return (
    <div className="min-h-screen bg-cyber-bg flex">
      {/* Sidebar */}
      <Sidebar collapsed={sidebarCollapsed} onToggle={() => setSidebarCollapsed(!sidebarCollapsed)} />

      {/* Main Content Area */}
      <main
        className={`flex-1 min-h-screen transition-all duration-300 ease-out ${
          sidebarCollapsed ? "ml-[72px]" : "ml-[240px]"
        }`}
      >
        {/* Top Bar */}
        <header className="sticky top-0 z-30 h-14 border-b border-cyber-border/30 backdrop-blur-xl bg-cyber-bg/80">
          <div className="h-full px-6 flex items-center justify-between">
            <Breadcrumb />
            <div className="flex items-center gap-3">
              <TopBarActions />
            </div>
          </div>
        </header>

        {/* Page Content with scrollable area */}
        <div className="h-[calc(100vh-3.5rem)] overflow-y-auto">
          <div className="p-6 max-w-[1440px] mx-auto">
            <Outlet />
          </div>
        </div>
      </main>
    </div>
  );
}

/* ---- Breadcrumb / Page Title ---- */
function Breadcrumb() {
  const location = useLocation();

  const routeMap: Record<string, string> = {
    "/dashboard": "仪表盘",
    "/tasks": "任务列表",
    "/tasks/new": "新建任务",
    "/filter": "二次筛选",
    "/vectorize": "向量化处理",
    "/proxies": "代理管理",
    "/settings": "设置",
  };

  // Handle dynamic routes
  let currentPath = location.pathname;
  if (currentPath.startsWith("/tasks/") && currentPath !== "/tasks/new") {
    currentPath = "/tasks"; // Task detail page
  }

  const title = routeMap[currentPath] || "页面";

  return (
    <div className="flex items-center gap-2">
      <span className="text-sm font-semibold text-cyber-text">{title}</span>
      {location.pathname !== "/dashboard" && (
        <>
          <span className="text-cyber-border-light text-xs">/</span>
          <span className="text-xs text-cyber-muted">Domain Scanner</span>
        </>
      )}
    </div>
  );
}

/* ---- Top Bar Right Actions ---- */
function TopBarActions() {
  return (
    <div className="flex items-center gap-1">
      {/* Status indicator */}
      <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-cyber-green/8 border border-cyber-green/15">
        <span className="status-dot-running w-1.5 h-1.5" />
        <span className="text-[11px] font-medium text-cyber-green">在线</span>
      </div>
    </div>
  );
}
