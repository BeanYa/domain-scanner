import { describe, it, expect, vi } from "vitest";

// We test the tauri service by mocking the @tauri-apps/api/core module
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

describe("tauri service", () => {
  it("invokeCommand should call invoke with correct args", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { invokeCommand } = await import("../../services/tauri");

    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue({ result: "ok" });

    const result = await invokeCommand("test_command", { arg1: "value1" });

    expect(invoke).toHaveBeenCalledWith("test_command", { arg1: "value1" });
    expect(result).toEqual({ result: "ok" });
  });

  it("invokeCommand should work without args", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { invokeCommand } = await import("../../services/tauri");

    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue("success");

    const result = await invokeCommand("test_command");

    expect(invoke).toHaveBeenCalledWith("test_command", undefined);
    expect(result).toBe("success");
  });

  it("listenEvent should call listen with correct args", async () => {
    const { listen } = await import("@tauri-apps/api/event");
    const { listenEvent } = await import("../../services/tauri");

    const mockUnlisten = vi.fn();
    (listen as ReturnType<typeof vi.fn>).mockResolvedValue(mockUnlisten);

    const handler = vi.fn();
    const unlisten = await listenEvent("test-event", handler);

    expect(listen).toHaveBeenCalledWith("test-event", expect.any(Function));
    expect(unlisten).toBe(mockUnlisten);
  });
});
