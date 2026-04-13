import { create } from "zustand";
import type { GpuConfig, GpuStatus, GpuBackend } from "../types";
import { invokeCommand } from "../services/tauri";

interface GpuStore {
  status: GpuStatus | null;
  config: GpuConfig | null;
  loading: boolean;
  error: string | null;

  fetchStatus: () => Promise<void>;
  updateConfig: (updates: { backend?: GpuBackend; device_id?: number; batch_size?: number; model_path?: string }) => Promise<void>;
}

export const useGpuStore = create<GpuStore>((set) => ({
  status: null,
  config: null,
  loading: false,
  error: null,

  fetchStatus: async () => {
    set({ loading: true, error: null });
    try {
      const result = await invokeCommand<string>("get_gpu_status");
      const parsed = JSON.parse(result);
      set({ status: parsed.status, config: parsed.config, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  updateConfig: async (updates) => {
    try {
      await invokeCommand("update_gpu_config", { request: updates });
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));
