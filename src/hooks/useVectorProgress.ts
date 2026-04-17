import { useState, useEffect, useCallback, useRef } from "react";
import { invokeCommand } from "../services/tauri";
import { listenEvent } from "../services/tauri";
import type { GpuBackend } from "../types";

interface VectorProgress {
  run_id: string | null;
  task_id: string;
  total: number;
  processed: number;
  percentage: number;
  backend: GpuBackend;
  speed_per_sec: number | null;
  estimated_remaining_secs: number | null;
  status: "idle" | "running" | "completed" | "failed" | "cancelled" | "interrupted";
  message: string | null;
  updated_at: string;
  started_at: string | null;
  finished_at: string | null;
}

/**
 * Hook for tracking vectorization progress
 */
export function useVectorProgress(taskId: string | null) {
  const [progress, setProgress] = useState<VectorProgress | null>(null);
  const [loading, setLoading] = useState(false);
  const unlistenRef = useRef<(() => void) | null>(null);

  const fetchProgress = useCallback(async () => {
    if (!taskId) return;
    setLoading(true);
    try {
      const result = await invokeCommand<string>("get_vectorize_progress", {
        task_id: taskId,
      });
      const parsed: VectorProgress = JSON.parse(result);
      setProgress(parsed);
    } catch (e) {
      console.error("Failed to fetch vector progress:", e);
    } finally {
      setLoading(false);
    }
  }, [taskId]);

  useEffect(() => {
    if (!taskId) return;

    // Initial fetch
    fetchProgress();

    // Listen for progress events
    const setup = async () => {
      unlistenRef.current = await listenEvent<VectorProgress>(
        `vector-progress-${taskId}`,
        (event) => {
          setProgress(event);
        }
      );
    };

    setup();

    return () => {
      unlistenRef.current?.();
    };
  }, [taskId, fetchProgress]);

  return { progress, loading, fetchProgress };
}
