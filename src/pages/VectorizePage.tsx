import { useState, useEffect } from "react";
import { Cpu, Play, HardDrive, Gauge, Clock, Inbox } from "lucide-react";
import { useGpuStore } from "../store/gpuStore";
import { useTaskStore } from "../store/taskStore";

export default function VectorizePage() {
  const [selectedTask, setSelectedTask] = useState("");
  const [isRunning, setIsRunning] = useState(false);
  const { status: gpuStatus, fetchStatus: fetchGpuStatus } = useGpuStore();
  const { tasks, fetchTasks } = useTaskStore();

  useEffect(() => {
    fetchGpuStatus();
    fetchTasks();
  }, []);

  const completedTasks = tasks.filter(t => t.status === "completed" && t.available_count > 0);

  const gpuBackendLabel = gpuStatus?.available
    ? gpuStatus.backend === "cuda" ? "CUDA"
    : gpuStatus.backend === "directml" ? "DirectML"
    : gpuStatus.backend === "cpu" ? "CPU"
    : gpuStatus.backend
    : "CPU";
  const gpuDeviceName = gpuStatus?.device_name || "CPU Only";

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
          <span className={`text-xs px-2 py-0.5 rounded-full ${
            gpuStatus?.available && gpuStatus.backend !== "cpu"
              ? "bg-cyber-green/10 text-cyber-green"
              : "bg-cyber-orange/10 text-cyber-orange"
          }`}>
            {gpuStatus?.available && gpuStatus.backend !== "cpu" ? "GPU 已启用" : "CPU 模式"}
          </span>
        </div>
        <div className="grid grid-cols-4 gap-4">
          <div className="p-3 rounded-lg bg-cyber-bg/50">
            <p className="text-[10px] text-cyber-muted mb-1">当前后端</p>
            <p className="text-sm font-semibold text-cyber-text">{gpuBackendLabel}</p>
          </div>
          <div className="p-3 rounded-lg bg-cyber-bg/50">
            <p className="text-[10px] text-cyber-muted mb-1">设备名称</p>
            <p className="text-sm font-semibold text-cyber-text">{gpuDeviceName}</p>
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
        {(!gpuStatus?.available || gpuStatus.backend === "cpu") && (
          <div className="mt-3 flex items-center gap-2 text-xs text-cyber-muted">
            <Cpu className="w-3.5 h-3.5 text-cyber-orange" />
            <span>未检测到 GPU，将使用 CPU 推理。可在设置中配置 GPU 后端加速。</span>
          </div>
        )}
      </div>

      {/* Task Selection */}
      <div className="glass-panel p-5 space-y-4">
        <h2 className="text-sm font-semibold text-cyber-text">选择源任务</h2>
        {completedTasks.length === 0 ? (
          <div className="text-center py-8 text-cyber-muted">
            <Inbox className="w-10 h-10 mx-auto mb-3 opacity-40" />
            <p className="text-sm">暂无已完成且含可用域名的任务</p>
            <p className="text-xs text-cyber-muted-dim mt-1">请先完成扫描任务</p>
          </div>
        ) : (
          <div className="space-y-2">
            {completedTasks.map((task) => (
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
                <span className="text-xs text-cyber-muted ml-auto">{task.available_count} 个可用域名</span>
              </label>
            ))}
          </div>
        )}
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
                <span className="text-cyber-text font-medium">0 / 0</span>
              </div>
              <div className="flex items-center gap-2">
                <Gauge className="w-3.5 h-3.5 text-cyber-muted" />
                <span className="text-cyber-muted">速度:</span>
                <span className="text-cyber-text font-medium">-</span>
              </div>
              <div className="flex items-center gap-2">
                <Clock className="w-3.5 h-3.5 text-cyber-muted" />
                <span className="text-cyber-muted">预计剩余:</span>
                <span className="text-cyber-text font-medium">-</span>
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
