import { renderHook, waitFor } from "@testing-library/react";
import { describe, it, expect, beforeEach, vi } from "vitest";
import { useTaskEvents } from "../../hooks/useTaskEvents";

vi.mock("../../services/tauri", () => ({
  listenEvent: vi.fn(),
}));

describe("useTaskEvents", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("subscribes to scan-progress and task-status-change events", async () => {
    const progressHandler = vi.fn();
    const statusHandler = vi.fn();
    const unlistenProgress = vi.fn();
    const unlistenStatus = vi.fn();
    const { listenEvent } = await import("../../services/tauri");

    (listenEvent as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce(unlistenProgress)
      .mockResolvedValueOnce(unlistenStatus);

    const { unmount } = renderHook(() =>
      useTaskEvents(progressHandler, statusHandler)
    );

    await waitFor(() => {
      expect(listenEvent).toHaveBeenCalledWith("scan-progress", progressHandler);
      expect(listenEvent).toHaveBeenCalledWith(
        "task-status-change",
        statusHandler
      );
    });

    unmount();

    expect(unlistenProgress).toHaveBeenCalledTimes(1);
    expect(unlistenStatus).toHaveBeenCalledTimes(1);
  });
});
