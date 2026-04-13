import { create } from "zustand";

interface ProxyStore {}

export const useProxyStore = create<ProxyStore>(() => ({}));
