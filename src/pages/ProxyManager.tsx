import { useState, useEffect } from "react";
import { Plus, Trash2, TestTube2, Wifi, WifiOff, Upload, Shield, Inbox } from "lucide-react";
import { useProxyStore } from "../store/proxyStore";

export default function ProxyManager() {
  const { proxies, fetchProxies, createProxy, deleteProxy, testProxy } = useProxyStore();
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ name: "", url: "", proxy_type: "http", username: "", password: "" });
  const [testing, setTesting] = useState<number | null>(null);

  useEffect(() => {
    fetchProxies();
  }, []);

  const typeColors = {
    http: "text-cyber-green bg-cyber-green/10",
    https: "text-cyber-blue bg-cyber-blue/10",
    socks5: "text-cyber-cyan bg-cyber-cyan/10",
  };

  return (
    <div className="p-6 space-y-6 max-w-4xl">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold neon-text">代理管理</h1>
          <p className="text-sm text-cyber-muted mt-1">配置代理轮转，支持 HTTP/HTTPS/SOCKS5</p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setShowAdd(true)}
            className="cyber-btn-primary flex items-center gap-2"
          >
            <Plus className="w-4 h-4" /> 添加代理
          </button>
          <button className="cyber-btn-secondary flex items-center gap-2">
            <Upload className="w-4 h-4" /> 批量导入
          </button>
        </div>
      </div>

      {/* Summary */}
      <div className="grid grid-cols-3 gap-4">
        <div className="glass-panel p-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-cyber-green/10 flex items-center justify-center">
            <Wifi className="w-5 h-5 text-cyber-green" />
          </div>
          <div>
            <p className="text-lg font-bold text-cyber-green">{proxies.filter((p) => p.is_active).length}</p>
            <p className="text-xs text-cyber-muted">在线代理</p>
          </div>
        </div>
        <div className="glass-panel p-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-cyber-red/10 flex items-center justify-center">
            <WifiOff className="w-5 h-5 text-cyber-red" />
          </div>
          <div>
            <p className="text-lg font-bold text-cyber-red">{proxies.filter((p) => !p.is_active).length}</p>
            <p className="text-xs text-cyber-muted">离线代理</p>
          </div>
        </div>
        <div className="glass-panel p-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-cyber-cyan/10 flex items-center justify-center">
            <Shield className="w-5 h-5 text-cyber-cyan" />
          </div>
          <div>
            <p className="text-lg font-bold text-cyber-text">{proxies.length}</p>
            <p className="text-xs text-cyber-muted">总计</p>
          </div>
        </div>
      </div>

      {/* Add Form */}
      {showAdd && (
        <div className="glass-panel p-5 space-y-4 neon-border">
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

      {/* Proxy List */}
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
                <th className="text-right px-5 py-2 font-medium">操作</th>
              </tr>
            </thead>
            <tbody>
              {proxies.map((proxy) => (
                <tr key={proxy.id} className="border-b border-cyber-border/10 hover:bg-cyber-card/30 transition-colors">
                  <td className="px-5 py-3 text-sm text-cyber-text font-medium">{proxy.name || "-"}</td>
                  <td className="px-5 py-3 text-sm font-mono text-cyber-muted">{proxy.url}</td>
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
                    {proxy.is_active ? (
                      <span className="inline-flex items-center gap-1 text-xs text-cyber-green">
                        <div className="w-1.5 h-1.5 rounded-full bg-cyber-green" /> 在线
                      </span>
                    ) : (
                      <span className="inline-flex items-center gap-1 text-xs text-cyber-red">
                        <div className="w-1.5 h-1.5 rounded-full bg-cyber-red" /> 离线
                      </span>
                    )}
                  </td>
                  <td className="px-5 py-3 text-right">
                    <div className="flex items-center justify-end gap-1">
                      <button
                        className="p-1.5 rounded hover:bg-cyber-green/10 text-cyber-muted hover:text-cyber-green disabled:opacity-40"
                        title="测试"
                        disabled={testing === proxy.id}
                        onClick={async () => {
                          setTesting(proxy.id);
                          await testProxy(proxy.id);
                          await fetchProxies();
                          setTesting(null);
                        }}
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
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
