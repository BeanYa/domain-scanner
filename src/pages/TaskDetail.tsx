import { useState } from "react";
import {
  ArrowLeft,
  Play,
  Pause,
  Download,
  Cpu,
  Filter,
  ChevronDown,
  AlertCircle,
  CheckCircle,
  XCircle,
  Clock,
  Terminal,
} from "lucide-react";
import { useNavigate, useParams } from "react-router-dom";

const mockLogs = [
  { level: "info", message: "扫描任务已启动", time: "12:00:01" },
  { level: "info", message: "正在查询 aaa.com (RDAP)", time: "12:00:02" },
  { level: "info", message: "aaa.com 可用 (响应 150ms)", time: "12:00:03" },
  { level: "warn", message: "aab.com 查询超时，降级为 DNS 查询", time: "12:00:04" },
  { level: "info", message: "aab.com 已注册 (DNS)", time: "12:00:04" },
  { level: "error", message: "aac.com RDAP 限流，等待重试...", time: "12:00:05" },
  { level: "info", message: "aad.com 可用 (响应 120ms)", time: "12:00:06" },
  { level: "info", message: "批量写入 500 条结果", time: "12:00:10" },
  { level: "info", message: "进度: 1,000/456,976 (0.2%)", time: "12:00:15" },
  { level: "warn", message: "代理 proxy-3 连接超时，切换 proxy-4", time: "12:00:20" },
];

const mockResults = [
  { domain: "aaa.com", available: true, method: "RDAP", time: 150 },
  { domain: "aab.com", available: false, method: "DNS", time: 200 },
  { domain: "aad.com", available: true, method: "RDAP", time: 120 },
  { domain: "aaf.com", available: true, method: "RDAP", time: 180 },
  { domain: "aag.com", available: false, method: "RDAP", time: 90 },
  { domain: "aah.com", available: true, method: "DNS", time: 210 },
];

const logColors = {
  info: "text-cyber-green",
  warn: "text-cyber-orange",
  error: "text-cyber-red",
};

export default function TaskDetail() {
  const { id } = useParams();
  const navigate = useNavigate();
  const [showLogs, setShowLogs] = useState(true);
  const [logFilter, setLogFilter] = useState<string>("all");

  const progress = 67;
  const total = 456976;
  const completed = 306189;
  const available = 1234;
  const errors = 56;

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center gap-4">
        <button
          onClick={() => navigate("/tasks")}
          className="p-2 rounded-lg hover:bg-cyber-surface text-cyber-muted hover:text-cyber-text transition-colors"
        >
          <ArrowLeft className="w-5 h-5" />
        </button>
        <div className="flex-1">
          <h1 className="text-xl font-bold text-cyber-text">4字母 .com 扫描</h1>
          <p className="text-sm text-cyber-muted">任务 ID: {id}</p>
        </div>
        <div className="flex items-center gap-2">
          <button className="cyber-btn-secondary flex items-center gap-1 text-sm">
            <Pause className="w-3.5 h-3.5" /> 暂停
          </button>
          <button className="cyber-btn-secondary flex items-center gap-1 text-sm">
            <Download className="w-3.5 h-3.5" /> 导出
          </button>
          <button className="cyber-btn-secondary flex items-center gap-1 text-sm">
            <Cpu className="w-3.5 h-3.5" /> 向量化
          </button>
          <button className="cyber-btn-secondary flex items-center gap-1 text-sm">
            <Filter className="w-3.5 h-3.5" /> 筛选
          </button>
        </div>
      </div>

      {/* Progress Section */}
      <div className="grid grid-cols-4 gap-4">
        <div className="col-span-1 glass-panel p-5 flex items-center justify-center">
          <div className="relative w-32 h-32">
            <svg className="w-full h-full -rotate-90" viewBox="0 0 128 128">
              <circle cx="64" cy="64" r="56" fill="none" stroke="#30363D" strokeWidth="8" />
              <circle
                cx="64" cy="64" r="56" fill="none"
                stroke="url(#progressGrad)" strokeWidth="8"
                strokeLinecap="round"
                strokeDasharray={`${(progress / 100) * 351.86} 351.86`}
              />
              <defs>
                <linearGradient id="progressGrad" x1="0%" y1="0%" x2="100%" y2="0%">
                  <stop offset="0%" stopColor="#00E5A0" />
                  <stop offset="100%" stopColor="#00C9DB" />
                </linearGradient>
              </defs>
            </svg>
            <div className="absolute inset-0 flex flex-col items-center justify-center">
              <span className="text-2xl font-bold neon-text">{progress}%</span>
              <span className="text-[10px] text-cyber-muted">进度</span>
            </div>
          </div>
        </div>
        <div className="col-span-3 grid grid-cols-3 gap-4">
          <div className="glass-panel p-5">
            <p className="text-xs text-cyber-muted mb-1">已完成</p>
            <p className="text-xl font-bold text-cyber-text">{completed.toLocaleString()}</p>
            <p className="text-[10px] text-cyber-muted">/ {total.toLocaleString()}</p>
          </div>
          <div className="glass-panel p-5">
            <p className="text-xs text-cyber-muted mb-1">可用域名</p>
            <p className="text-xl font-bold text-cyber-green">{available.toLocaleString()}</p>
            <p className="text-[10px] text-cyber-muted">
              {((available / completed) * 100).toFixed(1)}% 可用率
            </p>
          </div>
          <div className="glass-panel p-5">
            <p className="text-xs text-cyber-muted mb-1">错误数</p>
            <p className="text-xl font-bold text-cyber-red">{errors}</p>
            <p className="text-[10px] text-cyber-muted">
              {((errors / completed) * 100).toFixed(2)}% 错误率
            </p>
          </div>
        </div>
      </div>

      {/* Results Table */}
      <div className="glass-panel overflow-hidden">
        <div className="px-5 py-3 border-b border-cyber-border/30 flex items-center justify-between">
          <h2 className="text-sm font-semibold text-cyber-text">扫描结果</h2>
          <span className="text-xs text-cyber-muted">显示可用域名</span>
        </div>
        <table className="w-full">
          <thead>
            <tr className="text-xs text-cyber-muted border-b border-cyber-border/20">
              <th className="text-left px-5 py-2 font-medium">域名</th>
              <th className="text-left px-5 py-2 font-medium">状态</th>
              <th className="text-left px-5 py-2 font-medium">查询方式</th>
              <th className="text-right px-5 py-2 font-medium">响应时间</th>
            </tr>
          </thead>
          <tbody>
            {mockResults.map((r, i) => (
              <tr key={i} className="border-b border-cyber-border/10 hover:bg-cyber-card/30 transition-colors">
                <td className="px-5 py-2.5 text-sm font-mono text-cyber-text">{r.domain}</td>
                <td className="px-5 py-2.5">
                  {r.available ? (
                    <span className="flex items-center gap-1 text-xs text-cyber-green">
                      <CheckCircle className="w-3.5 h-3.5" /> 可用
                    </span>
                  ) : (
                    <span className="flex items-center gap-1 text-xs text-cyber-red">
                      <XCircle className="w-3.5 h-3.5" /> 已注册
                    </span>
                  )}
                </td>
                <td className="px-5 py-2.5 text-xs text-cyber-muted">{r.method}</td>
                <td className="px-5 py-2.5 text-xs text-cyber-muted text-right">{r.time}ms</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Log Panel */}
      <div className="glass-panel overflow-hidden">
        <div
          className="px-5 py-3 border-b border-cyber-border/30 flex items-center justify-between cursor-pointer"
          onClick={() => setShowLogs(!showLogs)}
        >
          <h2 className="text-sm font-semibold text-cyber-text flex items-center gap-2">
            <Terminal className="w-4 h-4 text-cyber-green" />
            实时日志
          </h2>
          <div className="flex items-center gap-3">
            <div className="flex bg-cyber-surface rounded p-0.5 text-xs">
              {["all", "info", "warn", "error"].map((lvl) => (
                <button
                  key={lvl}
                  onClick={(e) => { e.stopPropagation(); setLogFilter(lvl); }}
                  className={`px-2 py-0.5 rounded ${
                    logFilter === lvl ? "bg-cyber-green/20 text-cyber-green" : "text-cyber-muted"
                  }`}
                >
                  {lvl === "all" ? "全部" : lvl}
                </button>
              ))}
            </div>
            <ChevronDown className={`w-4 h-4 text-cyber-muted transition-transform ${showLogs ? "" : "-rotate-90"}`} />
          </div>
        </div>
        {showLogs && (
          <div className="p-4 bg-cyber-bg/80 max-h-48 overflow-y-auto font-mono text-xs space-y-1">
            {mockLogs
              .filter((l) => logFilter === "all" || l.level === logFilter)
              .map((log, i) => (
                <div key={i} className="flex gap-3">
                  <span className="text-cyber-muted">{log.time}</span>
                  <span className={`w-12 ${logColors[log.level as keyof typeof logColors]}`}>
                    [{log.level.toUpperCase()}]
                  </span>
                  <span className="text-cyber-text">{log.message}</span>
                </div>
              ))}
          </div>
        )}
      </div>
    </div>
  );
}
