import { describe, it, expect, beforeEach, vi } from "vitest";
import { useTaskStore } from "../../store/taskStore";
import type { Task } from "../../types";

// Mock the tauri service
vi.mock("../../services/tauri", () => ({
  invokeCommand: vi.fn(),
  listenEvent: vi.fn(),
}));

describe("taskStore", () => {
  beforeEach(() => {
    useTaskStore.setState({
      tasks: [],
      loading: false,
      error: null,
      selectedBatchId: null,
    });
    vi.clearAllMocks();
  });

  it("should have correct initial state", () => {
    const state = useTaskStore.getState();
    expect(state.tasks).toEqual([]);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
    expect(state.selectedBatchId).toBeNull();
  });

  it("should set selected batch id", () => {
    useTaskStore.getState().setSelectedBatchId("batch-1");
    expect(useTaskStore.getState().selectedBatchId).toBe("batch-1");
  });

  it("should clear selected batch id", () => {
    useTaskStore.getState().setSelectedBatchId("batch-1");
    useTaskStore.getState().setSelectedBatchId(null);
    expect(useTaskStore.getState().selectedBatchId).toBeNull();
  });

  it("should remove task from list on delete", async () => {
    const mockTasks: Task[] = [
      {
        id: "task-1",
        batch_id: null,
        name: "Test",
        signature: "sig1",
        status: "pending",
        scan_mode: { type: "regex", pattern: "^[a-z]{3}$" },
        config_json: "{}",
        tld: ".com",
        prefix_pattern: null,
        total_count: 100,
        completed_count: 0,
        completed_index: 0,
        available_count: 0,
        error_count: 0,
        created_at: "2026-01-01",
        updated_at: "2026-01-01",
      },
      {
        id: "task-2",
        batch_id: null,
        name: "Test 2",
        signature: "sig2",
        status: "running",
        scan_mode: { type: "regex", pattern: "^[a-z]{4}$" },
        config_json: "{}",
        tld: ".net",
        prefix_pattern: null,
        total_count: 200,
        completed_count: 50,
        completed_index: 50,
        available_count: 10,
        error_count: 1,
        created_at: "2026-01-01",
        updated_at: "2026-01-01",
      },
    ];

    useTaskStore.setState({ tasks: mockTasks });

    const { invokeCommand } = await import("../../services/tauri");
    (invokeCommand as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);

    await useTaskStore.getState().deleteTask("task-1");

    expect(useTaskStore.getState().tasks).toHaveLength(1);
    expect(useTaskStore.getState().tasks[0].id).toBe("task-2");
  });
});
