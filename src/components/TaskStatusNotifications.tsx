import { useEffect, useRef, useState } from "react";
import { BellRing, X } from "lucide-react";
import { listenEvent } from "../services/tauri";

interface TaskStatusEvent {
  task_id: string;
  task_name?: string;
  status: "pending" | "running" | "paused" | "stopped" | "completed";
  reason?: string | null;
  message?: string;
}

interface Toast {
  id: string;
  title: string;
  message: string;
  status: TaskStatusEvent["status"];
}

const statusLabel: Record<TaskStatusEvent["status"], string> = {
  pending: "等待中",
  running: "运行中",
  paused: "已暂停",
  stopped: "已停止",
  completed: "已完成",
};

const statusTone: Record<TaskStatusEvent["status"], string> = {
  pending: "border-cyber-muted-dim/40",
  running: "border-cyber-green/40",
  paused: "border-cyber-orange/50",
  stopped: "border-cyber-red/45",
  completed: "border-cyber-blue/40",
};

export default function TaskStatusNotifications() {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const lastEventRef = useRef<string>("");

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    const subscription = listenEvent<TaskStatusEvent>("task-status-change", (event) => {
      const dedupeKey = `${event.task_id}:${event.status}:${event.reason ?? ""}`;
      if (lastEventRef.current === dedupeKey) {
        return;
      }
      lastEventRef.current = dedupeKey;
      window.setTimeout(() => {
        if (lastEventRef.current === dedupeKey) {
          lastEventRef.current = "";
        }
      }, 1500);

      const taskName = event.task_name || event.task_id;
      const title = `任务状态：${statusLabel[event.status]}`;
      const message = event.message
        ? `${taskName}：${event.message}`
        : `${taskName} 已切换为 ${statusLabel[event.status]}`;
      const toast: Toast = {
        id: `${dedupeKey}:${Date.now()}`,
        title,
        message,
        status: event.status,
      };

      setToasts((current) => [...current.slice(-2), toast]);
      window.setTimeout(() => {
        setToasts((current) => current.filter((item) => item.id !== toast.id));
      }, 5200);

      if ("Notification" in window && Notification.permission === "granted") {
        new Notification(title, { body: message });
      }
    });

    Promise.resolve(subscription).then((cleanup) => {
      if (!cleanup) return;
      if (disposed) {
        cleanup();
      } else {
        unlisten = cleanup;
      }
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  if (toasts.length === 0) return null;

  return (
    <div className="fixed right-5 bottom-5 z-50 w-[360px] max-w-[calc(100vw-2rem)] space-y-2 pointer-events-none">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`pointer-events-auto glass-panel border ${statusTone[toast.status]} p-4 animate-fade-in`}
        >
          <div className="flex items-start gap-3">
            <BellRing className="w-4 h-4 mt-0.5 text-cyber-cyan shrink-0" />
            <div className="min-w-0 flex-1">
              <p className="text-sm font-semibold text-cyber-text">{toast.title}</p>
              <p className="mt-1 text-xs text-cyber-muted leading-5">{toast.message}</p>
            </div>
            <button
              className="p-1 rounded text-cyber-muted-dim hover:text-cyber-text hover:bg-cyber-card"
              onClick={() => setToasts((current) => current.filter((item) => item.id !== toast.id))}
              title="关闭"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
