import { create } from "zustand";
import type { LlmConfig } from "../types";
import { invokeCommand } from "../services/tauri";

interface LlmStore {
  configs: LlmConfig[];
  loading: boolean;
  error: string | null;
  testing: boolean;

  fetchConfigs: () => Promise<void>;
  saveConfig: (config: Partial<LlmConfig> & { name: string; base_url: string; api_key: string }) => Promise<void>;
  testConfig: (configId: string) => Promise<boolean>;
}

export const useLlmStore = create<LlmStore>((set) => ({
  configs: [],
  loading: false,
  error: null,
  testing: false,

  fetchConfigs: async () => {
    set({ loading: true, error: null });
    try {
      const result = await invokeCommand<string>("list_llm_configs");
      const configs: LlmConfig[] = JSON.parse(result);
      set({ configs, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  saveConfig: async (config) => {
    set({ loading: true, error: null });
    try {
      await invokeCommand("save_llm_config", { request: config });
      set({ loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  testConfig: async (configId: string) => {
    set({ testing: true, error: null });
    try {
      const result = await invokeCommand<string>("test_llm_config", { config_id: configId });
      const parsed = JSON.parse(result);
      set({ testing: false });
      return parsed.success === true;
    } catch (e) {
      set({ error: String(e), testing: false });
      return false;
    }
  },
}));
