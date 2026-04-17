import { create } from "zustand";
import type { ProxyConfig, ProxyTestResult } from "../types";
import { invokeCommand } from "../services/tauri";

interface ProxyStore {
  proxies: ProxyConfig[];
  loading: boolean;
  error: string | null;
  lastTestResult: ProxyTestResult | null;

  fetchProxies: (activeOnly?: boolean) => Promise<void>;
  createProxy: (proxy: { name?: string; url: string; proxy_type: string; username?: string; password?: string }) => Promise<void>;
  testProxy: (proxyId: number) => Promise<ProxyTestResult | null>;
  deleteProxy: (proxyId: number) => Promise<void>;
}

export const useProxyStore = create<ProxyStore>((set, get) => ({
  proxies: [],
  loading: false,
  error: null,
  lastTestResult: null,

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
      const parsed: ProxyTestResult = JSON.parse(result);
      set((state) => ({
        lastTestResult: parsed,
        proxies: state.proxies.map((proxy) =>
          proxy.id === proxyId
            ? {
                ...proxy,
                is_active: parsed.success,
                status: parsed.status,
                last_checked_at: parsed.checked_at,
                last_error: parsed.success ? null : parsed.message,
              }
            : proxy
        ),
      }));
      return parsed;
    } catch (e) {
      set({ error: String(e) });
      return null;
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
