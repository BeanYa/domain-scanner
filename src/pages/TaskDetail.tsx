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
  Copy,
  ExternalLink,
  Table,
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
];

const mockResults = [
  { domain: "aaa.com", available: true, method: "RDAP", time: 150, registrar: "-" },
  { domain: "aab.com", available: false, method: "DNS", time: 200, registrar: "GoDaddy" },
  { domain: "aad.com", available: true, method: "RDAP", time: 120, registrar: "-" },
  { domain: "aaf.com", available: true, method: "RDAP", time: 180, registrar: "-" },
  { domain: "aag.com", available: false, method: "RDAP", time: 90, registrar: "Namecheap" },
  { domain: "aah.com", available: true, method: "DNS", time: 210, registrar: "-" },
];

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

  const tlds = [".com", ".net", ".org"];
  const progress = 67;
  const total = 913952;
  const completed = 612378;
  const available = 2345;
  const errors = 89;

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
          <h1 className="text-xl font-bold text-cyber-text">4字母多TLD扫描</h1>
          <div className="flex items-center gap-2 mt-1 flex-wrap">
            {tlds.map((tld) => (
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
        {/* Circular Progress */}
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

        {/* Stats Grid */}
        <div className="grid grid-cols-3 gap-4">
          <div className="rounded-xl bg-cyber-bg-elevated/60 border border-cyber-border/20 p-4 space-y-1">
            <p className="text-xs text-cyber-muted">已完成</p>
            <p className="text-xl font-bold text-cyber-text tabular-nums">{completed.toLocaleString()}</p>
            <p className="text-[10px] text-cyber-muted-dim font-mono tabular-nums">/ {total.toLocaleString()}</p>
          </div>
          <div className="rounded-xl bg-cyber-bg-elevated/60 border border-cyber-border/20 p-4 space-y-1">
            <p className="text-xs text-cyber-muted">可用域名</p>
            <p className="text-xl font-bold text-cyber-green tabular-nums">{available.toLocaleString()}</p>
            <p className="text-[10px] text-cyber-muted-dim">{((available / completed) * 100).toFixed(1)}% 可用率</p>
          </div>
          <div className="rounded-xl bg-cyber-bg-elevated/60 border border-cyber-border/20 p-4 space-y-1">
            <p className="text-xs text-cyber-muted">错误数</p>
            <p className="text-xl font-bold text-cyber-red tabular-nums">{errors}</p>
            <p className="text-[10px] text-cyber-muted-dim">{((errors / completed) * 100).toFixed(2)}% 错误率</p>
          </div>
        </div>
      </div>

      {/* Results Table */}
      <div className="glass-panel overflow-hidden">
        <div className="px-5 py-3.5 border-b border-cyber-border/30 flex items-center justify-between">
          <h2 className="section-title m-0">
            <Table className="w-4 h-4 text-cyber-green" /> 扫描结果（可用域名）
          </h2>
          <div className="flex items-center gap-2">
            <span className="badge-green">{mockResults.filter(r => r.available).length} 可用</span>
            <span className="badge-neutral">{mockResults.length - mockResults.filter(r => r.available).length} 已注册</span>
          </div>
        </div>
        <div className="overflow-x-auto">
          <table className="data-table">
            <thead>
              <tr>
                <th>域名</th>
                <th className="w-20">状态</th>
                <th className="w-24">查询方式</th>
                <th className="w-24 text-right">延迟</th>
                <th className="w-32">注册商</th>
              </tr>
            </thead>
            <tbody>
              {mockResults.map((r, i) => (
                <tr key={i}>
                  <td>
                    <span className="font-mono text-sm text-cyber-text group-hover/link:text-cyber-green transition-colors cursor-pointer"
                      onClick={() => navigator.clipboard.writeText(r.domain)}>
                      {r.domain}
                    </span>
                  </td>
                  <td>
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
                  <td><span className={`badge ${r.method === 'RDAP' ? 'badge-blue' : 'badge-neutral'}`}>{r.method}</span></td>
                  <td className="font-mono text-right tabular-nums text-cyber-muted-dim">{r.time}ms</td>
                  <td className="text-cyber-muted-dim text-xs">{r.registrar}</td>
                </tr>
              ))}
            </tbody>
          </table>
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
            <div className="divide-y divide-cyber-border/10">
              {mockLogs
                .filter((l) => logFilter === "all" || l.level === logFilter)
                .map((log, i) => (
                  <div key={i} className="flex gap-4 px-5 py-2 hover:bg-cyber-card/30 transition-colors">
                    <span className="text-cyber-muted-dim w-16 shrink-0 tabular-nums">{log.time}</span>
                    <span className={`w-14 shrink-0 uppercase tracking-wider font-semibold ${logColors[log.level]}`}>
                      [{log.level}]
                    </span>
                    <span className="text-cyber-text-secondary">{log.message}</span>
                  </div>
                ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
