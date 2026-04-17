import { useState, useCallback, useRef, useEffect } from "react";
import { invokeCommand } from "../services/tauri";
import { listenEvent } from "../services/tauri";
import type { LogEntry } from "../types";

interface UseTaskLogsOptions {
  taskId: string;
  logType?: "task" | "request";
  level?: string;
  pageSize?: number;
  autoRefresh?: boolean;
  enabled?: boolean;
}

/**
 * Hook for fetching and streaming task logs
 */
export function useTaskLogs(options: UseTaskLogsOptions) {
  const { taskId, logType = "task", level, pageSize = 100, autoRefresh = true, enabled = true } = options;
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [offset, setOffset] = useState(0);
  const unlistenRef = useRef<(() => void) | null>(null);

  const fetchLogs = useCallback(
    async (newOffset?: number) => {
      if (!enabled || !taskId) return [];
      setLoading(true);
      try {
        const result = await invokeCommand<string>("get_logs", {
          request: {
            task_id: taskId,
            log_type: logType,
            level: level || null,
            limit: pageSize,
            offset: newOffset ?? offset,
          },
        });
        const fetched: LogEntry[] = JSON.parse(result);
        if (newOffset !== undefined) {
          setOffset(newOffset);
        }
        return fetched;
      } catch (e) {
        console.error("Failed to fetch logs:", e);
        return [];
      } finally {
        setLoading(false);
      }
    },
    [enabled, taskId, logType, level, pageSize, offset]
  );

  const loadMore = useCallback(async () => {
    const newOffset = offset + pageSize;
    const fetched = await fetchLogs(newOffset);
    setLogs((prev) => [...prev, ...fetched]);
  }, [offset, pageSize, fetchLogs]);

  // Initial fetch
  useEffect(() => {
    if (!enabled || !taskId) {
      setLogs([]);
      return;
    }
    const load = async () => {
      const fetched = await fetchLogs(0);
      setLogs(fetched);
    };
    load();
  }, [enabled, taskId, logType, level]);

  // Listen for real-time log events
  useEffect(() => {
    if (!enabled || !taskId || !autoRefresh) return;

    const setup = async () => {
      unlistenRef.current = await listenEvent<LogEntry>(
        `task-log-${taskId}`,
        (log) => {
          if (log.log_type !== logType) return;
          setLogs((prev) => [log, ...prev]);
        }
      );
    };

    setup();

    return () => {
      unlistenRef.current?.();
    };
  }, [enabled, taskId, logType, autoRefresh]);

  return { logs, loading, fetchLogs, loadMore };
}
