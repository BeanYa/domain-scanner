import { createElement } from "react";
import { act, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";

vi.mock("../services/tauri", () => ({
  invokeCommand: vi.fn(),
  listenEvent: vi.fn(),
}));

describe("App routing", () => {
  beforeEach(async () => {
    window.history.pushState({}, "", "/tasks");
    const { invokeCommand } = await import("../services/tauri");
    (invokeCommand as ReturnType<typeof vi.fn>).mockImplementation(
      async (command: string) => {
        if (command === "list_batches") {
          return JSON.stringify([]);
        }
        if (command === "list_tasks") {
          return JSON.stringify([]);
        }
        if (command === "get_gpu_status") {
          return JSON.stringify({
            backend: "cpu",
            available: true,
            device_name: "CPU",
            vram_total_mb: null,
            vram_used_mb: null,
          });
        }
        if (command === "list_proxies") {
          return JSON.stringify([]);
        }
        return undefined;
      }
    );
  });

  it("renders the task list route inside the app layout", async () => {
    render(createElement(App));
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(screen.getAllByText("任务列表")[0]).toBeInTheDocument();
  });
});
