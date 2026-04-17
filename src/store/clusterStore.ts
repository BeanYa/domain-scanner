import { create } from "zustand";
import type {
  ClusterWorker,
  CreateWorkerRegistrationResponse,
  WorkerHealthCheckResult,
} from "../types";
import { invokeCommand } from "../services/tauri";

interface ClusterStore {
  workers: ClusterWorker[];
  loading: boolean;
  registering: boolean;
  testingWorkerId: string | null;
  error: string | null;
  lastRegistration: CreateWorkerRegistrationResponse | null;
  lastHealthResult: WorkerHealthCheckResult | null;

  fetchWorkers: () => Promise<void>;
  createWorkerRegistration: (request: {
    base_url: string;
    name?: string;
    script_url?: string;
    port?: number;
    timeout_seconds?: number;
  }) => Promise<CreateWorkerRegistrationResponse | null>;
  pollWorkerRegistration: (workerId: string) => Promise<WorkerHealthCheckResult | null>;
  testWorker: (workerId: string) => Promise<WorkerHealthCheckResult | null>;
  enableWorker: (workerId: string) => Promise<void>;
  disableWorker: (workerId: string) => Promise<void>;
  deleteWorker: (workerId: string) => Promise<void>;
}

export const useClusterStore = create<ClusterStore>((set, get) => ({
  workers: [],
  loading: false,
  registering: false,
  testingWorkerId: null,
  error: null,
  lastRegistration: null,
  lastHealthResult: null,

  fetchWorkers: async () => {
    set({ loading: true, error: null });
    try {
      const result = await invokeCommand<string>("list_workers");
      set({ workers: JSON.parse(result) as ClusterWorker[], loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  createWorkerRegistration: async (request) => {
    set({ registering: true, error: null });
    try {
      const result = await invokeCommand<string>("create_worker_registration", { request });
      const registration = JSON.parse(result) as CreateWorkerRegistrationResponse;
      set({ registering: false, lastRegistration: registration });
      await get().fetchWorkers();
      return registration;
    } catch (e) {
      set({ error: String(e), registering: false });
      return null;
    }
  },

  pollWorkerRegistration: async (workerId) => {
    set({ testingWorkerId: workerId, error: null });
    try {
      const result = await invokeCommand<string>("poll_worker_registration", {
        request: { worker_id: workerId },
      });
      const health = JSON.parse(result) as WorkerHealthCheckResult;
      set({ testingWorkerId: null, lastHealthResult: health });
      await get().fetchWorkers();
      return health;
    } catch (e) {
      set({ error: String(e), testingWorkerId: null });
      return null;
    }
  },

  testWorker: async (workerId) => {
    set({ testingWorkerId: workerId, error: null });
    try {
      const result = await invokeCommand<string>("test_worker", {
        request: { worker_id: workerId },
      });
      const health = JSON.parse(result) as WorkerHealthCheckResult;
      set({ testingWorkerId: null, lastHealthResult: health });
      await get().fetchWorkers();
      return health;
    } catch (e) {
      set({ error: String(e), testingWorkerId: null });
      return null;
    }
  },

  enableWorker: async (workerId) => {
    try {
      await invokeCommand("enable_worker", { request: { worker_id: workerId } });
      await get().fetchWorkers();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  disableWorker: async (workerId) => {
    try {
      await invokeCommand("disable_worker", { request: { worker_id: workerId } });
      await get().fetchWorkers();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  deleteWorker: async (workerId) => {
    try {
      await invokeCommand("delete_worker", { request: { worker_id: workerId } });
      await get().fetchWorkers();
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));

