import { useState } from "react";
import { Outlet, useLocation } from "react-router-dom";
import Sidebar from "./Sidebar";

export default function AppLayout() {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  return (
    <div className="min-h-screen overflow-x-hidden bg-cyber-bg">
      <Sidebar collapsed={sidebarCollapsed} onToggle={() => setSidebarCollapsed(!sidebarCollapsed)} />

      <main
        className={`min-h-screen min-w-0 overflow-x-hidden transition-[margin,width] duration-300 ease-out ${
          sidebarCollapsed
            ? "ml-[72px] w-[calc(100vw-72px)]"
            : "ml-[240px] w-[calc(100vw-240px)]"
        }`}
      >
        <header className="sticky top-0 z-30 h-14 border-b border-cyber-border bg-black/95">
          <div className="h-full min-w-0 px-6 flex items-center justify-between">
            <Breadcrumb />
            <TopBarActions />
          </div>
        </header>

        <div className="h-[calc(100vh-3.5rem)] overflow-y-auto overflow-x-hidden">
          <div className="w-full min-w-0 p-6 lg:p-8 max-w-[1600px] mx-auto">
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
      <span className="eyebrow text-cyber-text">{title}</span>
      {location.pathname !== "/dashboard" && (
        <>
          <span className="text-cyber-border-light text-xs">/</span>
          <span className="eyebrow">Domain Scanner</span>
        </>
      )}
    </div>
  );
}

/* ---- Top Bar Right Actions ---- */
function TopBarActions() {
  return (
    <div className="flex items-center gap-1">
      <div className="flex items-center gap-1.5 px-2.5 py-1 rounded border border-cyber-border bg-cyber-bg-elevated">
        <span className="status-dot-running w-1.5 h-1.5" />
        <span className="eyebrow text-cyber-text-secondary">在线</span>
      </div>
    </div>
  );
}
