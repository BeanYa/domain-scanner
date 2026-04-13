import { useState } from "react";
import { Cpu, Play, HardDrive, Gauge, Clock, CheckCircle, AlertTriangle } from "lucide-react";

export default function VectorizePage() {
  const [selectedTask, setSelectedTask] = useState("");
  const [isRunning, setIsRunning] = useState(false);

  const tasks = [
    { id: "t1", name: "4字母 .com 扫描", available: 1234 },
    { id: "t2", name: "AI 相关 .io 扫描", available: 89 },
    { id: "t3", name: "品牌词 .com 扫描", available: 412 },
  ];

  return (
    <div className="p-6 space-y-6 max-w-4xl">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold neon-text">向量化处理</h1>
        <p className="text-sm text-cyber-muted mt-1">将域名文本转化为向量，支持 GPU 加速语义搜索</p>
      </div>

      {/* GPU Status */}
      <div className="glass-panel p-5">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-sm font-semibold text-cyber-text flex items-center gap-2">
            <Cpu className="w-4 h-4 text-cyber-green" />
            GPU 状态
          </h2>
          <span className="text-xs px-2 py-0.5 rounded-full bg-cyber-green/10 text-cyber-green">
            就绪
          </span>
        </div>
        <div className="grid grid-cols-4 gap-4">
          <div className="p-3 rounded-lg bg-cyber-bg/50">
            <p className="text-[10px] text-cyber-muted mb-1">当前后端</p>
            <p className="text-sm font-semibold text-cyber-text">CPU</p>
          </div>
          <div className="p-3 rounded-lg bg-cyber-bg/50">
            <p className="text-[10px] text-cyber-muted mb-1">设备名称</p>
            <p className="text-sm font-semibold text-cyber-text">CPU Only</p>
          </div>
          <div className="p-3 rounded-lg bg-cyber-bg/50">
            <p className="text-[10px] text-cyber-muted mb-1">模型</p>
            <p className="text-sm font-semibold text-cyber-text">MiniLM-L6-v2</p>
          </div>
          <div className="p-3 rounded-lg bg-cyber-bg/50">
            <p className="text-[10px] text-cyber-muted mb-1">维度</p>
            <p className="text-sm font-semibold text-cyber-text">384</p>
          </div>
        </div>
        <div className="mt-3 flex items-center gap-2 text-xs text-cyber-muted">
          <AlertTriangle className="w-3.5 h-3.5 text-cyber-orange" />
          <span>未检测到 GPU，将使用 CPU 推理。可在设置中配置 GPU 后端加速。</span>
        </div>
      </div>

      {/* Task Selection */}
      <div className="glass-panel p-5 space-y-4">
        <h2 className="text-sm font-semibold text-cyber-text">选择源任务</h2>
        <div className="space-y-2">
          {tasks.map((task) => (
            <label
              key={task.id}
              className={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-all ${
                selectedTask === task.id
                  ? "bg-cyber-green/10 border border-cyber-green/30"
                  : "bg-cyber-bg/50 border border-cyber-border/20 hover:border-cyber-border/40"
              }`}
            >
              <input
                type="radio"
                name="task"
                value={task.id}
                checked={selectedTask === task.id}
                onChange={(e) => setSelectedTask(e.target.value)}
                className="accent-cyber-green"
              />
              <span className="text-sm text-cyber-text font-medium">{task.name}</span>
              <span className="text-xs text-cyber-muted ml-auto">{task.available} 个可用域名</span>
            </label>
          ))}
        </div>
      </div>

      {/* Progress */}
      <div className="glass-panel p-5 space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-semibold text-cyber-text flex items-center gap-2">
            <Gauge className="w-4 h-4 text-cyber-cyan" />
            向量化进度
          </h2>
          {isRunning && (
            <span className="text-xs text-cyber-green animate-pulse">处理中...</span>
          )}
        </div>

        {isRunning ? (
          <div className="space-y-3">
            <div className="h-2 rounded-full bg-cyber-surface overflow-hidden">
              <div className="h-full rounded-full bg-gradient-to-r from-cyber-green to-cyber-cyan animate-pulse" style={{ width: "35%" }} />
            </div>
            <div className="grid grid-cols-3 gap-4 text-xs">
              <div className="flex items-center gap-2">
                <HardDrive className="w-3.5 h-3.5 text-cyber-muted" />
                <span className="text-cyber-muted">已处理:</span>
                <span className="text-cyber-text font-medium">432 / 1,234</span>
              </div>
              <div className="flex items-center gap-2">
                <Gauge className="w-3.5 h-3.5 text-cyber-muted" />
                <span className="text-cyber-muted">速度:</span>
                <span className="text-cyber-text font-medium">150/秒</span>
              </div>
              <div className="flex items-center gap-2">
                <Clock className="w-3.5 h-3.5 text-cyber-muted" />
                <span className="text-cyber-muted">预计剩余:</span>
                <span className="text-cyber-text font-medium">5.4 分钟</span>
              </div>
            </div>
          </div>
        ) : (
          <div className="text-center py-8 text-cyber-muted">
            <Cpu className="w-10 h-10 mx-auto mb-2 opacity-40" />
            <p className="text-sm">选择任务后点击开始</p>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex justify-end gap-3">
        <button
          onClick={() => setIsRunning(!isRunning)}
          className="cyber-btn-primary flex items-center gap-2"
          disabled={!selectedTask}
        >
          <Play className="w-4 h-4" />
          {isRunning ? "停止" : "开始向量化"}
        </button>
      </div>
    </div>
  );
}
