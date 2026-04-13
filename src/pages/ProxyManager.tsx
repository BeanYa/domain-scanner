import { useState } from "react";
import { Plus, Trash2, TestTube2, Wifi, WifiOff, Upload, Shield } from "lucide-react";

interface Proxy {
  id: number;
  name: string;
  url: string;
  type: "http" | "https" | "socks5";
  active: boolean;
}

const mockProxies: Proxy[] = [
  { id: 1, name: "US Proxy 1", url: "http://us1.proxy.io:8080", type: "http", active: true },
  { id: 2, name: "US Proxy 2", url: "http://us2.proxy.io:8080", type: "http", active: true },
  { id: 3, name: "EU Proxy", url: "socks5://eu.proxy.io:1080", type: "socks5", active: false },
  { id: 4, name: "Asia Proxy", url: "https://asia.proxy.io:443", type: "https", active: true },
  { id: 5, name: "Backup Proxy", url: "http://backup.proxy.io:8080", type: "http", active: true },
];

export default function ProxyManager() {
  const [proxies, setProxies] = useState(mockProxies);
  const [showAdd, setShowAdd] = useState(false);

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
            <p className="text-lg font-bold text-cyber-green">{proxies.filter((p) => p.active).length}</p>
            <p className="text-xs text-cyber-muted">在线代理</p>
          </div>
        </div>
        <div className="glass-panel p-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-cyber-red/10 flex items-center justify-center">
            <WifiOff className="w-5 h-5 text-cyber-red" />
          </div>
          <div>
            <p className="text-lg font-bold text-cyber-red">{proxies.filter((p) => !p.active).length}</p>
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
              <input type="text" placeholder="代理名称" className="cyber-input w-full text-sm" />
            </div>
            <div>
              <label className="block text-xs text-cyber-muted mb-1">类型</label>
              <select className="cyber-input w-full text-sm">
                <option value="http">HTTP</option>
                <option value="https">HTTPS</option>
                <option value="socks5">SOCKS5</option>
              </select>
            </div>
            <div className="col-span-2">
              <label className="block text-xs text-cyber-muted mb-1">URL</label>
              <input type="text" placeholder="http://host:port 或 socks5://host:port" className="cyber-input w-full text-sm font-mono" />
            </div>
          </div>
          <div className="flex gap-2 justify-end">
            <button onClick={() => setShowAdd(false)} className="cyber-btn-secondary text-sm">取消</button>
            <button className="cyber-btn-primary text-sm">添加</button>
          </div>
        </div>
      )}

      {/* Proxy List */}
      <div className="glass-panel overflow-hidden">
        <table className="w-full">
          <thead>
            <tr className="text-xs text-cyber-muted border-b border-cyber-border/20">
              <th className="text-left px-5 py-2 font-medium">名称</th>
              <th className="text-left px-5 py-2 font-medium">URL</th>
              <th className="text-center px-5 py-2 font-medium">类型</th>
              <th className="text-center px-5 py-2 font-medium">状态</th>
              <th className="text-right px-5 py-2 font-medium">操作</th>
            </tr>
          </thead>
          <tbody>
            {proxies.map((proxy) => (
              <tr key={proxy.id} className="border-b border-cyber-border/10 hover:bg-cyber-card/30 transition-colors">
                <td className="px-5 py-3 text-sm text-cyber-text font-medium">{proxy.name}</td>
                <td className="px-5 py-3 text-sm font-mono text-cyber-muted">{proxy.url}</td>
                <td className="px-5 py-3 text-center">
                  <span className={`text-xs px-2 py-0.5 rounded ${typeColors[proxy.type]}`}>
                    {proxy.type.toUpperCase()}
                  </span>
                </td>
                <td className="px-5 py-3 text-center">
                  {proxy.active ? (
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
                    <button className="p-1.5 rounded hover:bg-cyber-green/10 text-cyber-muted hover:text-cyber-green" title="测试">
                      <TestTube2 className="w-3.5 h-3.5" />
                    </button>
                    <button className="p-1.5 rounded hover:bg-cyber-red/10 text-cyber-muted hover:text-cyber-red" title="删除">
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
