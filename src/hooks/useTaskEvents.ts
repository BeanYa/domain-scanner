import { useEffect, useRef } from "react";
import { listenEvent } from "../services/tauri";

interface TaskProgressEvent {
  task_id: string;
  run_id: string;
  completed_count: number;
  total_count: number;
  available_count: number;
  error_count: number;
}

interface TaskStatusEvent {
  task_id: string;
  status: string;
}

/**
 * Hook to listen for Tauri task events (progress updates, status changes)
 */
export function useTaskEvents(
  onProgress?: (event: TaskProgressEvent) => void,
  onStatusChange?: (event: TaskStatusEvent) => void
) {
  const unlistenProgressRef = useRef<(() => void) | null>(null);
  const unlistenStatusRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    const setup = async () => {
      if (onProgress) {
        unlistenProgressRef.current = await listenEvent<TaskProgressEvent>(
          "scan-progress",
          onProgress
        );
      }
      if (onStatusChange) {
        unlistenStatusRef.current = await listenEvent<TaskStatusEvent>(
          "task-status-change",
          onStatusChange
        );
      }
    };

    setup();

    return () => {
      unlistenProgressRef.current?.();
      unlistenStatusRef.current?.();
    };
  }, [onProgress, onStatusChange]);
}
