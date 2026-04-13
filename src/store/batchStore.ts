import { create } from "zustand";
import type { TaskBatch } from "../types";
import { invokeCommand } from "../services/tauri";

interface BatchStore {
  batches: TaskBatch[];
  loading: boolean;
  error: string | null;

  fetchBatches: () => Promise<void>;
  batchPause: (batchId: string) => Promise<void>;
  batchResume: (batchId: string) => Promise<void>;
}

export const useBatchStore = create<BatchStore>((set) => ({
  batches: [],
  loading: false,
  error: null,

  fetchBatches: async () => {
    set({ loading: true, error: null });
    try {
      const result = await invokeCommand<string>("list_batches", {
        request: { limit: 100, offset: 0 },
      });
      const batches: TaskBatch[] = JSON.parse(result);
      set({ batches, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  batchPause: async (batchId: string) => {
    try {
      await invokeCommand("batch_pause", { batch_id: batchId });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  batchResume: async (batchId: string) => {
    try {
      await invokeCommand("batch_resume", { batch_id: batchId });
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));
