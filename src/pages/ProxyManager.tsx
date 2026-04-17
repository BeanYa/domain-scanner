import { useState, useEffect } from "react";
import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  Clock3,
  Inbox,
  Loader2,
  Plus,
  Shield,
  TestTube2,
  Trash2,
  Upload,
  Wifi,
  WifiOff,
  XCircle,
} from "lucide-react";
import { ProxyStatusBadge } from "../components/ProxyStatusBadge";
import { useProxyStore } from "../store/proxyStore";
import type { ProxyConfig, ProxyTestResult } from "../types";

type TestNotice = {
  state: "running" | "success" | "error";
  title: string;
  message: string;
};

export default function ProxyManager() {
  const { proxies, fetchProxies, createProxy, deleteProxy, testProxy, lastTestResult } = useProxyStore();
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ name: "", url: "", proxy_type: "http", username: "", password: "" });
  const [testing, setTesting] = useState<number | null>(null);
  const [testingAll, setTestingAll] = useState(false);
  const [testNotice, setTestNotice] = useState<TestNotice | null>(null);

  useEffect(() => {
    fetchProxies();
  }, []);

  const typeColors: Record<ProxyConfig["proxy_type"], string> = {
    http: "text-cyber-green bg-cyber-green/10",
    https: "text-cyber-blue bg-cyber-blue/10",
    socks5: "text-cyber-cyan bg-cyber-cyan/10",
  };

  const availableCount = proxies.filter((p) => p.status === "available" && p.is_active).length;
  const offlineCount = proxies.filter((p) => p.status === "unavailable" || p.status === "pending").length;
  const errorCount = proxies.filter((p) => p.status === "error").length;

  const handleTestProxy = async (proxyId: number) => {
    const proxy = proxies.find((item) => item.id === proxyId);
    const proxyLabel = formatProxyLabel(proxy);
    setTesting(proxyId);
    setTestNotice({
      state: "running",
      title: "正在检测代理",
      message: `${proxyLabel} 正在连接扫描 RDAP 端点，请等待响应。`,
    });
    try {
      const result = await testProxy(proxyId);
      await fetchProxies();
      if (result) {
        setTestNotice({
          state: result.success ? "success" : "error",
          title: result.success ? "代理检测通过" : "代理检测未通过",
          message: `${proxyLabel}：${result.message}。端点通过 ${result.reachable_count}/${result.total_count}。`,
        });
      } else {
        const error = useProxyStore.getState().error;
        setTestNotice({
          state: "error",
          title: "代理检测失败",
          message: `${proxyLabel}：${error || "没有收到检测结果，请检查代理配置或稍后重试。"}`,
        });
      }
    } finally {
      setTesting(null);
    }
  };

  const handleTestAll = async () => {
    if (proxies.length === 0) return;
    setTestingAll(true);
    let passed = 0;
    let failed = 0;
    for (const [index, proxy] of proxies.entries()) {
      const proxyLabel = formatProxyLabel(proxy);
      setTesting(proxy.id);
      setTestNotice({
        state: "running",
        title: "正在批量检测代理",
        message: `(${index + 1}/${proxies.length}) ${proxyLabel} 正在连接扫描 RDAP 端点。`,
      });
      const result = await testProxy(proxy.id);
      if (result?.success) {
        passed += 1;
      } else {
        failed += 1;
      }
    }
    await fetchProxies();
    setTesting(null);
    setTestingAll(false);
    setTestNotice({
      state: failed === 0 ? "success" : "error",
      title: "批量检测完成",
      message: `已检测 ${proxies.length} 个代理，通过 ${passed} 个，失败 ${failed} 个。`,
    });
  };

  return (
    <div className="page-shell max-w-6xl">
      <div className="flex items-end justify-between gap-4">
        <div>
          <div className="eyebrow mb-3">PROXY POOL</div>
          <h1 className="page-heading">代理管理</h1>
          <p className="page-subtitle">配置代理轮转、端点检测和健康状态，让高并发扫描保持稳定节奏。</p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleTestAll}
            disabled={testingAll || proxies.length === 0}
            className="cyber-btn-secondary flex items-center gap-2 disabled:opacity-40"
          >
            <Activity className={`w-4 h-4 ${testingAll ? "animate-pulse" : ""}`} /> 检测全部
          </button>
          <button
            onClick={() => setShowAdd(true)}
            className="cyber-btn-primary flex items-center gap-2"
          >
            <Plus className="w-4 h-4" /> 添加代理
          </button>
          <button
            className="cyber-btn-secondary flex items-center gap-2"
            onClick={() => {
              const notice = {
                state: "error" as const,
                title: "批量导入尚未接入",
                message: "当前只支持逐条添加代理；批量导入需要文件选择和解析流程，前端入口已响应但功能尚未实现。",
              };
              console.info("[ui-action] proxy-batch-import", notice);
              setTestNotice(notice);
            }}
          >
            <Upload className="w-4 h-4" /> 批量导入
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
        <div className="glass-panel p-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-md border border-white/10 bg-white/[0.04] flex items-center justify-center">
            <Wifi className="w-5 h-5 text-cyber-green" />
          </div>
          <div>
            <p className="text-lg font-bold text-cyber-green">{availableCount}</p>
            <p className="text-xs text-cyber-muted">可用代理</p>
          </div>
        </div>
        <div className="glass-panel p-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-md border border-cyber-red/20 bg-cyber-red/10 flex items-center justify-center">
            <WifiOff className="w-5 h-5 text-cyber-red" />
          </div>
          <div>
            <p className="text-lg font-bold text-cyber-red">{offlineCount}</p>
            <p className="text-xs text-cyber-muted">离线代理</p>
          </div>
        </div>
        <div className="glass-panel p-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-md border border-cyber-orange/20 bg-cyber-orange/10 flex items-center justify-center">
            <AlertTriangle className="w-5 h-5 text-cyber-orange" />
          </div>
          <div>
            <p className="text-lg font-bold text-cyber-orange">{errorCount}</p>
            <p className="text-xs text-cyber-muted">失败代理</p>
          </div>
        </div>
        <div className="glass-panel p-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-md border border-white/10 bg-white/[0.04] flex items-center justify-center">
            <Shield className="w-5 h-5 text-cyber-cyan" />
          </div>
          <div>
            <p className="text-lg font-bold text-cyber-text">{proxies.length}</p>
            <p className="text-xs text-cyber-muted">总计</p>
          </div>
        </div>
      </div>

      {testNotice && <ProxyTestNotice notice={testNotice} onClose={() => setTestNotice(null)} />}
      {lastTestResult && <ProxyTestSummary result={lastTestResult} />}

      {showAdd && (
        <div className="glass-panel p-5 space-y-4">
          <h2 className="text-sm font-semibold text-cyber-text">添加新代理</h2>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs text-cyber-muted mb-1">名称</label>
              <input type="text" placeholder="代理名称" className="cyber-input w-full text-sm"
                value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
            </div>
            <div>
              <label className="block text-xs text-cyber-muted mb-1">类型</label>
              <select className="cyber-input w-full text-sm"
                value={form.proxy_type} onChange={(e) => setForm({ ...form, proxy_type: e.target.value })}>
                <option value="http">HTTP</option>
                <option value="https">HTTPS</option>
                <option value="socks5">SOCKS5</option>
              </select>
            </div>
            <div className="col-span-2">
              <label className="block text-xs text-cyber-muted mb-1">URL</label>
              <input type="text" placeholder="host:port 或 http://host:port" className="cyber-input w-full text-sm font-mono"
                value={form.url} onChange={(e) => setForm({ ...form, url: e.target.value })} />
            </div>
            <div>
              <label className="block text-xs text-cyber-muted mb-1">用户名 <span className="text-cyber-muted-dim">(可选)</span></label>
              <input type="text" placeholder="代理用户名" className="cyber-input w-full text-sm"
                value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} />
            </div>
            <div>
              <label className="block text-xs text-cyber-muted mb-1">密码 <span className="text-cyber-muted-dim">(可选)</span></label>
              <input type="password" placeholder="代理密码" className="cyber-input w-full text-sm"
                value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} />
            </div>
          </div>
          <div className="flex gap-2 justify-end">
            <button onClick={() => { setShowAdd(false); setForm({ name: "", url: "", proxy_type: "http", username: "", password: "" }); }} className="cyber-btn-secondary text-sm">取消</button>
            <button className="cyber-btn-primary text-sm" onClick={async () => {
              if (!form.url.trim()) return;
              await createProxy({
                name: form.name || undefined,
                url: form.url,
                proxy_type: form.proxy_type,
                username: form.username || undefined,
                password: form.password || undefined,
              });
              setForm({ name: "", url: "", proxy_type: "http", username: "", password: "" });
              setShowAdd(false);
            }}>添加</button>
          </div>
        </div>
      )}

      {proxies.length === 0 ? (
        <div className="glass-panel p-12 text-center text-cyber-muted">
          <Inbox className="w-10 h-10 mx-auto mb-3 opacity-40" />
          <p className="text-sm">暂无代理</p>
          <p className="text-xs text-cyber-muted-dim mt-1">点击「添加代理」配置代理服务器</p>
        </div>
      ) : (
        <div className="glass-panel overflow-hidden">
          <table className="w-full">
            <thead>
              <tr className="text-xs text-cyber-muted border-b border-cyber-border/20">
                <th className="text-left px-5 py-2 font-medium">名称</th>
                <th className="text-left px-5 py-2 font-medium">URL</th>
                <th className="text-center px-5 py-2 font-medium">类型</th>
                <th className="text-center px-5 py-2 font-medium">认证</th>
                <th className="text-center px-5 py-2 font-medium">状态</th>
                <th className="text-left px-5 py-2 font-medium">最近检测</th>
                <th className="text-right px-5 py-2 font-medium">操作</th>
              </tr>
            </thead>
            <tbody>
              {proxies.map((proxy) => {
                return (
                <tr key={proxy.id} className="border-b border-cyber-border hover:bg-cyber-card transition-colors">
                  <td className="px-5 py-3 text-sm text-cyber-text font-medium">{proxy.name || "-"}</td>
                  <td className="px-5 py-3">
                    <div className="text-sm font-mono text-cyber-muted max-w-[360px] truncate" title={proxy.url}>
                      {proxy.url}
                    </div>
                    {proxy.last_error && (
                      <div className="mt-1 text-[11px] text-cyber-orange max-w-[360px] truncate" title={proxy.last_error}>
                        {proxy.last_error}
                      </div>
                    )}
                  </td>
                  <td className="px-5 py-3 text-center">
                    <span className={`text-xs px-2 py-0.5 rounded ${typeColors[proxy.proxy_type]}`}>
                      {proxy.proxy_type.toUpperCase()}
                    </span>
                  </td>
                  <td className="px-5 py-3 text-center">
                    {proxy.username ? (
                      <span className="text-xs text-cyber-cyan">{proxy.username}</span>
                    ) : (
                      <span className="text-xs text-cyber-muted-dim">-</span>
                    )}
                  </td>
                  <td className="px-5 py-3 text-center">
                    <ProxyStatusBadge status={proxy.status} className="py-1 text-xs" />
                  </td>
                  <td className="px-5 py-3 text-left">
                    {proxy.last_checked_at ? (
                      <span className="inline-flex items-center gap-1.5 text-xs text-cyber-muted">
                        <Clock3 className="w-3 h-3" />
                        {new Date(proxy.last_checked_at).toLocaleString()}
                      </span>
                    ) : (
                      <span className="text-xs text-cyber-muted-dim">未检测</span>
                    )}
                  </td>
                  <td className="px-5 py-3 text-right">
                    <div className="flex items-center justify-end gap-1">
                      <button
                        className="p-1.5 rounded hover:bg-cyber-green/10 text-cyber-muted hover:text-cyber-green disabled:opacity-40"
                        title={testing === proxy.id ? "正在测试" : "测试代理"}
                        aria-label={testing === proxy.id ? "正在测试代理" : "测试代理"}
                        disabled={testing === proxy.id}
                        onClick={() => handleTestProxy(proxy.id)}
                      >
                        <TestTube2 className={`w-3.5 h-3.5 ${testing === proxy.id ? "animate-spin" : ""}`} />
                      </button>
                      <button
                        className="p-1.5 rounded hover:bg-cyber-red/10 text-cyber-muted hover:text-cyber-red"
                        title="删除"
                        onClick={() => deleteProxy(proxy.id)}
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </td>
                </tr>
              )})}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function ProxyTestNotice({ notice, onClose }: { notice: TestNotice; onClose: () => void }) {
  const tone = notice.state === "success"
    ? "border-cyber-green/35 bg-cyber-green/[0.06]"
    : notice.state === "running"
    ? "border-cyber-cyan/35 bg-cyber-cyan/[0.06]"
    : "border-cyber-orange/40 bg-cyber-orange/[0.06]";
  const Icon = notice.state === "success"
    ? CheckCircle2
    : notice.state === "running"
    ? Loader2
    : AlertTriangle;
  const iconClass = notice.state === "success"
    ? "text-cyber-green"
    : notice.state === "running"
    ? "text-cyber-cyan animate-spin"
    : "text-cyber-orange";

  return (
    <div className={`glass-panel border ${tone} p-4`}>
      <div className="flex items-start gap-3">
        <Icon className={`w-4 h-4 mt-0.5 shrink-0 ${iconClass}`} />
        <div className="min-w-0 flex-1">
          <p className="text-sm font-semibold text-cyber-text">{notice.title}</p>
          <p className="mt-1 text-xs text-cyber-muted leading-5">{notice.message}</p>
        </div>
        <button
          className="p-1 rounded text-cyber-muted-dim hover:text-cyber-text hover:bg-cyber-card/70"
          onClick={onClose}
          title="关闭"
        >
          <XCircle className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}

function ProxyTestSummary({ result }: { result: ProxyTestResult }) {
  return (
    <div className="glass-panel p-5 space-y-4">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h2 className="text-sm font-semibold text-cyber-text">最近一次代理端点检测</h2>
          <p className="text-xs text-cyber-muted mt-1">
            {result.message}，通过 {result.reachable_count}/{result.total_count} 个扫描端点
          </p>
        </div>
        <span className={result.success ? "badge-green" : "badge-orange"}>
          {result.success ? "检测通过" : "需要处理"}
        </span>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
        {result.endpoints.map((endpoint) => (
          <div
            key={endpoint.key}
            className="border border-cyber-border/20 rounded-md px-3 py-2 bg-cyber-card/30"
          >
            <div className="flex items-center justify-between gap-3">
              <span className="text-xs font-medium text-cyber-text">{endpoint.label}</span>
              <span className={endpoint.reachable ? "text-xs text-cyber-green" : "text-xs text-cyber-red"}>
                {endpoint.reachable ? "可达" : "失败"}
              </span>
            </div>
            <div className="mt-1 flex items-center gap-3 text-[11px] text-cyber-muted">
              <span className="font-mono truncate" title={endpoint.url}>{endpoint.url}</span>
              <span className="font-mono truncate">{endpoint.http_status ?? "NO_STATUS"}</span>
              <span>{endpoint.response_time_ms ?? 0} ms</span>
            </div>
            {endpoint.error_message && (
              <div className="mt-1 text-[11px] text-cyber-orange truncate" title={endpoint.error_message}>
                {endpoint.error_message}
              </div>
            )}
          </div>
        ))}
      </div>
      {result.notes.length > 0 && (
        <div className="space-y-1 border-t border-cyber-border/20 pt-3">
          {result.notes.map((note) => (
            <p key={note} className="text-[11px] text-cyber-muted leading-5">{note}</p>
          ))}
        </div>
      )}
    </div>
  );
}

function formatProxyLabel(proxy: ProxyConfig | undefined) {
  if (!proxy) return "该代理";
  return proxy.name ? `${proxy.name} (${proxy.url})` : proxy.url;
}
