import { useState } from "react";
import { Search, Play, Pause, Download, ChevronDown, ChevronRight, Trash2 } from "lucide-react";
import { useNavigate } from "react-router-dom";
import type { TaskStatus } from "../types";

interface MockTask {
  id: string;
  name: string;
  tld: string;
  status: TaskStatus;
  total_count: number;
  completed_count: number;
  available_count: number;
  error_count: number;
  batch_id: string | null;
}

interface MockBatch {
  id: string;
  name: string;
  task_count: number;
  tasks: MockTask[];
}

const mockBatches: MockBatch[] = [
  {
    id: "b1",
    name: "多 TLD 扫描批次 #1",
    task_count: 3,
    tasks: [
      { id: "t1", name: "4字母 .com", tld: ".com", status: "running", total_count: 456976, completed_count: 306189, available_count: 1234, error_count: 56, batch_id: "b1" },
      { id: "t2", name: "4字母 .net", tld: ".net", status: "running", total_count: 456976, completed_count: 198034, available_count: 876, error_count: 23, batch_id: "b1" },
      { id: "t3", name: "4字母 .org", tld: ".org", status: "paused", total_count: 456976, completed_count: 45697, available_count: 321, error_count: 12, batch_id: "b1" },
    ],
  },
  {
    id: "b2",
    name: "AI 域名批次",
    task_count: 2,
    tasks: [
      { id: "t4", name: "AI 相关 .io", tld: ".io", status: "completed", total_count: 500, completed_count: 500, available_count: 89, error_count: 5, batch_id: "b2" },
      { id: "t5", name: "AI 相关 .ai", tld: ".ai", status: "completed", total_count: 300, completed_count: 300, available_count: 45, error_count: 2, batch_id: "b2" },
    ],
  },
];

const statusConfig = {
  pending: { label: "等待中", color: "text-cyber-muted", bg: "bg-cyber-muted/10", dot: "bg-cyber-muted" },
  running: { label: "运行中", color: "text-cyber-green", bg: "bg-cyber-green/10", dot: "bg-cyber-green animate-pulse" },
  paused: { label: "已暂停", color: "text-cyber-orange", bg: "bg-cyber-orange/10", dot: "bg-cyber-orange" },
  completed: { label: "已完成", color: "text-cyber-blue", bg: "bg-cyber-blue/10", dot: "bg-cyber-blue" },
};

type TabFilter = "all" | TaskStatus;

export default function TaskList() {
  const [activeTab, setActiveTab] = useState<TabFilter>("all");
  const [search, setSearch] = useState("");
  const [expandedBatches, setExpandedBatches] = useState<Set<string>>(new Set(mockBatches.map((b) => b.id)));
  const navigate = useNavigate();

  const tabs: { key: TabFilter; label: string }[] = [
    { key: "all", label: "全部" },
    { key: "running", label: "运行中" },
    { key: "paused", label: "已暂停" },
    { key: "completed", label: "已完成" },
  ];

  const toggleBatch = (id: string) => {
    setExpandedBatches((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const filteredBatches = mockBatches
    .map((batch) => ({
      ...batch,
      tasks: batch.tasks.filter(
        (t) =>
          (activeTab === "all" || t.status === activeTab) &&
          (search === "" || t.name.toLowerCase().includes(search.toLowerCase()))
      ),
    }))
    .filter((b) => b.tasks.length > 0);

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold neon-text">任务列表</h1>
          <p className="text-sm text-cyber-muted mt-1">按批次管理扫描任务</p>
        </div>
        <button
          onClick={() => navigate("/tasks/new")}
          className="cyber-btn-primary"
        >
          新建任务
        </button>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="flex bg-cyber-surface rounded-lg p-1 border border-cyber-border/50">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`px-4 py-1.5 rounded-md text-xs font-medium transition-all ${
                activeTab === tab.key
                  ? "bg-cyber-green/20 text-cyber-green"
                  : "text-cyber-muted hover:text-cyber-text"
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
        <div className="relative flex-1 max-w-xs">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-cyber-muted" />
          <input
            type="text"
            placeholder="搜索任务..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="cyber-input w-full pl-9 text-sm"
          />
        </div>
      </div>

      {/* Batch Groups */}
      <div className="space-y-4">
        {filteredBatches.map((batch) => {
          const isExpanded = expandedBatches.has(batch.id);
          return (
            <div key={batch.id} className="glass-panel overflow-hidden">
              {/* Batch Header */}
              <div
                className="flex items-center gap-3 px-5 py-3 cursor-pointer hover:bg-cyber-card/40 transition-colors"
                onClick={() => toggleBatch(batch.id)}
              >
                {isExpanded ? (
                  <ChevronDown className="w-4 h-4 text-cyber-muted" />
                ) : (
                  <ChevronRight className="w-4 h-4 text-cyber-muted" />
                )}
                <span className="text-sm font-semibold text-cyber-text">{batch.name}</span>
                <span className="text-xs text-cyber-muted px-2 py-0.5 rounded-full bg-cyber-surface">
                  {batch.task_count} 个任务
                </span>
                <div className="ml-auto flex items-center gap-2">
                  <button className="cyber-btn-secondary text-xs px-2 py-1" onClick={(e) => { e.stopPropagation(); }}>
                    <Pause className="w-3 h-3 mr-1" /> 全部暂停
                  </button>
                  <button className="cyber-btn-secondary text-xs px-2 py-1" onClick={(e) => { e.stopPropagation(); }}>
                    <Download className="w-3 h-3 mr-1" /> 批量导出
                  </button>
                </div>
              </div>

              {/* Tasks */}
              {isExpanded && (
                <div className="border-t border-cyber-border/30">
                  {batch.tasks.map((task) => {
                    const cfg = statusConfig[task.status];
                    const progress = task.total_count > 0
                      ? Math.round((task.completed_count / task.total_count) * 100)
                      : 0;
                    return (
                      <div
                        key={task.id}
                        className="flex items-center gap-4 px-5 py-3 hover:bg-cyber-bg/30 transition-colors cursor-pointer border-b border-cyber-border/20 last:border-0"
                        onClick={() => navigate(`/tasks/${task.id}`)}
                      >
                        <span className={`w-2 h-2 rounded-full ${cfg.dot}`} />
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <span className="text-sm text-cyber-text font-medium truncate">
                              {task.name}
                            </span>
                            <span className="text-xs text-cyber-muted px-1.5 py-0.5 rounded bg-cyber-surface">
                              {task.tld}
                            </span>
                            <span className={`text-xs px-1.5 py-0.5 rounded ${cfg.bg} ${cfg.color}`}>
                              {cfg.label}
                            </span>
                          </div>
                          <div className="flex items-center gap-3 mt-1.5">
                            <div className="flex-1 h-1 rounded-full bg-cyber-surface overflow-hidden">
                              <div
                                className="h-full rounded-full bg-gradient-to-r from-cyber-green to-cyber-cyan"
                                style={{ width: `${progress}%` }}
                              />
                            </div>
                            <span className="text-[10px] text-cyber-muted">{progress}%</span>
                          </div>
                        </div>
                        <div className="flex items-center gap-6 text-xs">
                          <div className="text-center">
                            <p className="text-cyber-green font-semibold">{task.available_count}</p>
                            <p className="text-cyber-muted">可用</p>
                          </div>
                          <div className="text-center">
                            <p className="text-cyber-orange">{task.error_count}</p>
                            <p className="text-cyber-muted">错误</p>
                          </div>
                          <div className="flex gap-1">
                            {task.status === "running" && (
                              <button className="p-1.5 rounded hover:bg-cyber-orange/10 text-cyber-muted hover:text-cyber-orange" onClick={(e) => { e.stopPropagation(); }}>
                                <Pause className="w-3.5 h-3.5" />
                              </button>
                            )}
                            {task.status === "paused" && (
                              <button className="p-1.5 rounded hover:bg-cyber-green/10 text-cyber-muted hover:text-cyber-green" onClick={(e) => { e.stopPropagation(); }}>
                                <Play className="w-3.5 h-3.5" />
                              </button>
                            )}
                            <button className="p-1.5 rounded hover:bg-cyber-red/10 text-cyber-muted hover:text-cyber-red" onClick={(e) => { e.stopPropagation(); }}>
                              <Trash2 className="w-3.5 h-3.5" />
                            </button>
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          );
        })}

        {filteredBatches.length === 0 && (
          <div className="glass-panel p-12 text-center">
            <p className="text-cyber-muted">没有找到匹配的任务</p>
          </div>
        )}
      </div>
    </div>
  );
}
