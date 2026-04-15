import { create } from "zustand";
import type { Task, TaskStatus, ScanMode, BatchCreateResult } from "../types";
import { invokeCommand } from "../services/tauri";

interface TaskStore {
  tasks: Task[];
  loading: boolean;
  error: string | null;
  selectedBatchId: string | null;

  fetchTasks: (status?: TaskStatus, batchId?: string) => Promise<void>;
  createTasks: (name: string, scanMode: ScanMode, tlds: string[], batchName?: string, concurrency?: number, proxyId?: number) => Promise<BatchCreateResult>;
  startTask: (taskId: string) => Promise<void>;
  pauseTask: (taskId: string) => Promise<void>;
  resumeTask: (taskId: string) => Promise<void>;
  rerunTask: (taskId: string) => Promise<string>;
  deleteTask: (taskId: string) => Promise<void>;
  setSelectedBatchId: (id: string | null) => void;
}

export const useTaskStore = create<TaskStore>((set) => ({
  tasks: [],
  loading: false,
  error: null,
  selectedBatchId: null,

  fetchTasks: async (status?: TaskStatus, batchId?: string) => {
    set({ loading: true, error: null });
    try {
      const result = await invokeCommand<string>("list_tasks", {
        request: {
          status: status ?? null,
          batch_id: batchId || null,
          limit: 1000,
          offset: 0,
        },
      });
      const tasks: Task[] = JSON.parse(result);
      set({ tasks, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  createTasks: async (name: string, scanMode: ScanMode, tlds: string[], batchName?: string, concurrency?: number, proxyId?: number) => {
    set({ loading: true, error: null });
    try {
      const result = await invokeCommand<BatchCreateResult>("create_tasks", {
        request: {
          name,
          scan_mode: scanMode,
          tlds,
          batch_name: batchName,
          concurrency: concurrency || 50,
          proxy_id: proxyId || null,
        },
      });
      set({ loading: false });
      return result;
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  startTask: async (taskId: string) => {
    try {
      await invokeCommand("start_task", { taskId });
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  pauseTask: async (taskId: string) => {
    try {
      await invokeCommand("pause_task", { taskId });
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  resumeTask: async (taskId: string) => {
    try {
      await invokeCommand("resume_task", { taskId });
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  rerunTask: async (taskId: string) => {
    try {
      return await invokeCommand<string>("rerun_task", { taskId });
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  deleteTask: async (taskId: string) => {
    try {
      await invokeCommand("delete_task", { taskId });
      set((state) => ({ tasks: state.tasks.filter((t) => t.id !== taskId) }));
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  setSelectedBatchId: (id: string | null) => set({ selectedBatchId: id }),
}));
