import { create } from "zustand";
import type { ProxyConfig } from "../types";
import { invokeCommand } from "../services/tauri";

interface ProxyStore {
  proxies: ProxyConfig[];
  loading: boolean;
  error: string | null;

  fetchProxies: (activeOnly?: boolean) => Promise<void>;
  createProxy: (proxy: { name?: string; url: string; proxy_type: string; username?: string; password?: string }) => Promise<void>;
  testProxy: (proxyId: number) => Promise<boolean>;
  deleteProxy: (proxyId: number) => Promise<void>;
}

export const useProxyStore = create<ProxyStore>((set, get) => ({
  proxies: [],
  loading: false,
  error: null,

  fetchProxies: async (activeOnly?: boolean) => {
    set({ loading: true, error: null });
    try {
      const result = await invokeCommand<string>("list_proxies", {
        active_only: activeOnly || false,
      });
      const proxies: ProxyConfig[] = JSON.parse(result);
      set({ proxies, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  createProxy: async (proxy) => {
    try {
      const result = await invokeCommand<string>("create_proxy", { request: proxy });
      const created: ProxyConfig = JSON.parse(result);
      set((state) => ({ proxies: [...state.proxies, created] }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  testProxy: async (proxyId: number) => {
    try {
      const result = await invokeCommand<string>("test_proxy", { proxy_id: proxyId });
      const parsed = JSON.parse(result);
      return parsed.success === true;
    } catch (e) {
      set({ error: String(e) });
      return false;
    }
  },

  deleteProxy: async (proxyId: number) => {
    try {
      await invokeCommand("delete_proxy", { proxy_id: proxyId });
      set((state) => ({ proxies: state.proxies.filter((p) => p.id !== proxyId) }));
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));
