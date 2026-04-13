import { create } from "zustand";

interface GpuStore {}

export const useGpuStore = create<GpuStore>(() => ({}));
