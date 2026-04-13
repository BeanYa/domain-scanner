import { create } from "zustand";

interface LlmStore {}

export const useLlmStore = create<LlmStore>(() => ({}));
