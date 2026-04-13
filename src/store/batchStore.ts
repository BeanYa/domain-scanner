import { create } from "zustand";

interface BatchStore {}

export const useBatchStore = create<BatchStore>(() => ({}));
