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

type EventHandler = (payload: unknown) => void;

const eventHandlers = new Map<string, EventHandler[]>();

function registerEventHandler(event: string, handler: EventHandler) {
  const current = eventHandlers.get(event) ?? [];
  current.push(handler);
  eventHandlers.set(event, current);
  return () => {
    const next = (eventHandlers.get(event) ?? []).filter((entry) => entry !== handler);
    eventHandlers.set(event, next);
  };
}

function emitEvent(event: string, payload: unknown) {
  for (const handler of eventHandlers.get(event) ?? []) {
    handler(payload);
  }
}

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
    eventHandlers.clear();
    useBatchStore.setState({ batches: [], loading: false, error: null });

    const { invokeCommand, listenEvent } = await import("../../services/tauri");
    (listenEvent as ReturnType<typeof vi.fn>).mockImplementation(
      async (event: string, handler: EventHandler) =>
        registerEventHandler(event, handler)
    );
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
      <MemoryRouter future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
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
      vi.advanceTimersByTime(15000);
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
      <MemoryRouter future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
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
      vi.advanceTimersByTime(15000);
      await Promise.resolve();
    });

    expect(invokeCommand).not.toHaveBeenCalledWith("list_tasks", expect.anything());
  });

  it("updates task status and progress from events without forcing a full refetch", async () => {
    useTaskStore.setState({
      tasks: [makeTask("running")],
      loading: false,
      error: null,
      selectedBatchId: null,
    });

    const { invokeCommand } = await import("../../services/tauri");

    render(
      <MemoryRouter future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
        <TaskList />
      </MemoryRouter>
    );

    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });

    (invokeCommand as ReturnType<typeof vi.fn>).mockClear();

    await act(async () => {
      emitEvent("scan-progress", {
        task_id: "task-running",
        run_id: "run-1",
        completed_count: 500,
        total_count: 1000,
        available_count: 11,
        error_count: 2,
      });
      emitEvent("task-status-change", {
        task_id: "task-running",
        status: "completed",
      });
      await Promise.resolve();
    });

    expect(screen.getByText("50%")).toBeInTheDocument();
    expect(screen.getByText("已完成")).toBeInTheDocument();
    expect(invokeCommand).not.toHaveBeenCalledWith("list_tasks", expect.anything());
  });
});
