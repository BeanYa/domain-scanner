import { act, render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import TaskDetail from "../../pages/TaskDetail";
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

function makeTask(status: Task["status"] = "running"): Task {
  return {
    id: "task-1",
    batch_id: null,
    name: "Massive Scan",
    signature: "sig-1",
    status,
    scan_mode: { type: "regex", pattern: "^[a-z]{5}$" },
    config_json: "{}",
    tlds: [".com", ".net"],
    prefix_pattern: "^[a-z]{5}$",
    concurrency: 200,
    proxy_id: null,
    total_count: 3000000,
    completed_count: 0,
    completed_index: 0,
    available_count: 0,
    error_count: 0,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    primaryTld() {
      return this.tlds[0];
    },
  };
}

describe("TaskDetail", () => {
  beforeEach(async () => {
    vi.useFakeTimers();
    eventHandlers.clear();
    useTaskStore.setState({
      tasks: [makeTask()],
      loading: false,
      error: null,
      selectedBatchId: null,
    });

    const { invokeCommand, listenEvent } = await import("../../services/tauri");

    (listenEvent as ReturnType<typeof vi.fn>).mockImplementation(
      async (event: string, handler: EventHandler) =>
        registerEventHandler(event, handler)
    );

    (invokeCommand as ReturnType<typeof vi.fn>).mockImplementation(
      async (command: string) => {
        if (command === "list_task_runs") {
          return JSON.stringify([
            {
              id: "run-1",
              task_id: "task-1",
              run_number: 1,
              status: "running",
              total_count: 3000000,
              completed_count: 0,
              available_count: 0,
              error_count: 0,
              started_at: "2026-01-01T00:00:00Z",
              finished_at: null,
            },
          ]);
        }
        if (command === "list_scan_items") {
          return JSON.stringify({
            items: [],
            total: 0,
            page: 1,
            per_page: 10,
          });
        }
        if (command === "get_logs") {
          return JSON.stringify([]);
        }
        return undefined;
      }
    );
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  it("does not refetch paged results on scan-progress bursts and coalesces scan-results-updated", async () => {
    const { invokeCommand } = await import("../../services/tauri");

    render(
      <MemoryRouter initialEntries={["/tasks/task-1"]}>
        <Routes>
          <Route path="/tasks/:id" element={<TaskDetail />} />
        </Routes>
      </MemoryRouter>
    );

    expect(screen.getByText("Massive Scan")).toBeInTheDocument();

    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(screen.getByText("Run #1")).toBeInTheDocument();
    expect(invokeCommand).toHaveBeenCalledWith("list_scan_items", expect.anything());

    (invokeCommand as ReturnType<typeof vi.fn>).mockClear();

    await act(async () => {
      for (let index = 1; index <= 50; index += 1) {
        emitEvent("scan-progress", {
          task_id: "task-1",
          run_id: "run-1",
          completed_count: index,
          total_count: 3000000,
          available_count: 0,
          error_count: 0,
          percent: 0,
        });
      }
      vi.advanceTimersByTime(1000);
    });

    expect(invokeCommand).not.toHaveBeenCalledWith("list_scan_items", expect.anything());

    await act(async () => {
      for (let index = 1; index <= 10; index += 1) {
        emitEvent("scan-results-updated", {
          task_id: "task-1",
          run_id: "run-1",
          flushed_count: 500,
          completed_count: index * 500,
        });
      }
      vi.advanceTimersByTime(99);
    });

    expect(invokeCommand).not.toHaveBeenCalledWith("list_scan_items", expect.anything());

    await act(async () => {
      vi.advanceTimersByTime(1);
    });

    const listCalls = (invokeCommand as ReturnType<typeof vi.fn>).mock.calls.filter(
      ([command]) => command === "list_scan_items"
    );
    expect(listCalls).toHaveLength(1);
  });
});
