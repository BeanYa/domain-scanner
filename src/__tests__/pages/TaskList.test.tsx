import { act, render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import TaskList from "../../pages/TaskList";
import { useBatchStore } from "../../store/batchStore";
import { useTaskStore } from "../../store/taskStore";
import type { Task } from "../../types";

vi.mock("../../services/tauri", () => ({
  invokeCommand: vi.fn(),
  listenEvent: vi.fn(),
}));

function makeTask(status: Task["status"]): Task {
  return {
    id: `task-${status}`,
    batch_id: null,
    name: `${status} task`,
    signature: `sig-${status}`,
    status,
    scan_mode: { type: "regex", pattern: "^[a-z]{4}$" },
    config_json: "{}",
    tlds: [".com"],
    prefix_pattern: "^[a-z]{4}$",
    concurrency: 50,
    proxy_id: null,
    total_count: 1000,
    completed_count: 100,
    completed_index: 100,
    available_count: 3,
    error_count: 0,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    primaryTld() {
      return this.tlds[0];
    },
  };
}

describe("TaskList polling", () => {
  beforeEach(async () => {
    vi.useFakeTimers();
    useBatchStore.setState({ batches: [], loading: false, error: null });

    const { invokeCommand } = await import("../../services/tauri");
    (invokeCommand as ReturnType<typeof vi.fn>).mockImplementation(
      async (command: string) => {
        if (command === "list_batches") {
          return JSON.stringify([]);
        }
        if (command === "list_tasks") {
          return JSON.stringify(useTaskStore.getState().tasks);
        }
        return undefined;
      }
    );
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  it("polls while at least one task is running", async () => {
    useTaskStore.setState({
      tasks: [makeTask("running")],
      loading: false,
      error: null,
      selectedBatchId: null,
    });

    const { invokeCommand } = await import("../../services/tauri");

    render(
      <MemoryRouter>
        <TaskList />
      </MemoryRouter>
    );

    expect(screen.getByText("running task")).toBeInTheDocument();

    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(invokeCommand).toHaveBeenCalledWith("list_tasks", expect.anything());

    (invokeCommand as ReturnType<typeof vi.fn>).mockClear();
    await act(async () => {
      vi.advanceTimersByTime(3000);
      await Promise.resolve();
    });

    expect(invokeCommand).toHaveBeenCalledWith("list_tasks", expect.anything());
  });

  it("does not start polling when no task is running", async () => {
    useTaskStore.setState({
      tasks: [makeTask("completed")],
      loading: false,
      error: null,
      selectedBatchId: null,
    });

    const { invokeCommand } = await import("../../services/tauri");

    render(
      <MemoryRouter>
        <TaskList />
      </MemoryRouter>
    );

    expect(screen.getByText("completed task")).toBeInTheDocument();

    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(invokeCommand).toHaveBeenCalledWith("list_tasks", expect.anything());

    (invokeCommand as ReturnType<typeof vi.fn>).mockClear();
    await act(async () => {
      vi.advanceTimersByTime(3000);
      await Promise.resolve();
    });

    expect(invokeCommand).not.toHaveBeenCalledWith("list_tasks", expect.anything());
  });
});
