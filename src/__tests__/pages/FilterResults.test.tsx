import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import FilterResults from "../../pages/FilterResults";
import { useTaskStore } from "../../store/taskStore";
import type { Task } from "../../types";

vi.mock("../../services/tauri", () => ({
  invokeCommand: vi.fn(),
}));

function makeCompletedTask(): Task {
  return {
    id: "task-1",
    batch_id: null,
    name: "Brand Scan",
    signature: "sig-1",
    status: "completed",
    scan_mode: { type: "regex", pattern: "^[a-z]{4}$" },
    config_json: "{}",
    tlds: [".com", ".net"],
    prefix_pattern: "^[a-z]{4}$",
    concurrency: 50,
    proxy_id: null,
    total_count: 100,
    completed_count: 100,
    completed_index: 100,
    available_count: 12,
    error_count: 1,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T01:00:00Z",
    primaryTld() {
      return this.tlds[0];
    },
  };
}

describe("FilterResults", () => {
  beforeEach(async () => {
    useTaskStore.setState({
      tasks: [],
      loading: false,
      error: null,
      selectedBatchId: null,
    });

    const { invokeCommand } = await import("../../services/tauri");
    (invokeCommand as ReturnType<typeof vi.fn>).mockReset();
    (invokeCommand as ReturnType<typeof vi.fn>).mockImplementation(
      async (command: string) => {
        if (command === "list_tasks") {
          return JSON.stringify([makeCompletedTask()]);
        }
        if (command === "filter_exact") {
          return JSON.stringify({
            total: 1,
            items: [
              {
                id: 10,
                task_id: "task-1",
                domain: "acme.com",
                filter_type: "exact",
                filter_pattern: "acme.com",
                is_matched: true,
                score: null,
                embedding_id: null,
              },
            ],
          });
        }
        return undefined;
      }
    );
  });

  it("loads and lists completed tasks as selectable filter sources", async () => {
    const { invokeCommand } = await import("../../services/tauri");

    render(<FilterResults />);

    expect(await screen.findByText("Brand Scan")).toBeInTheDocument();
    expect(screen.getByText(".com")).toBeInTheDocument();
    expect(screen.getByText(".net")).toBeInTheDocument();
    expect(invokeCommand).toHaveBeenCalledWith("list_tasks", {
      request: {
        status: "completed",
        batch_id: null,
        limit: 1000,
        offset: 0,
      },
    });
  });

  it("runs the selected task through the active filter and renders matches", async () => {
    const { invokeCommand } = await import("../../services/tauri");
    const user = userEvent.setup();

    render(<FilterResults />);

    await screen.findByText("Brand Scan");
    await user.type(screen.getByPlaceholderText("输入精确域名，如 techworld"), "acme.com");
    await user.click(screen.getByRole("button", { name: /执行筛选/ }));

    expect(invokeCommand).toHaveBeenCalledWith("filter_exact", {
      request: {
        task_id: "task-1",
        query: "acme.com",
      },
    });
    expect(await screen.findAllByText("acme.com")).toHaveLength(2);
    expect(screen.getByText("精确")).toBeInTheDocument();
  });
});
