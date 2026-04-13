import { describe, it, expect, beforeEach, vi } from "vitest";
import { useGpuStore } from "../../store/gpuStore";
import type { GpuConfig, GpuStatus, GpuBackend } from "../../types";

vi.mock("../../services/tauri", () => ({
  invokeCommand: vi.fn(),
  listenEvent: vi.fn(),
}));

describe("gpuStore", () => {
  beforeEach(() => {
    useGpuStore.setState({
      status: null,
      config: null,
      loading: false,
      error: null,
    });
    vi.clearAllMocks();
  });

  it("should have correct initial state", () => {
    const state = useGpuStore.getState();
    expect(state.status).toBeNull();
    expect(state.config).toBeNull();
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  it("should fetch GPU status", async () => {
    const mockStatus: GpuStatus = {
      backend: "cpu",
      available: true,
      device_name: "CPU",
      vram_total_mb: null,
      vram_used_mb: null,
    };
    const mockConfig: GpuConfig = {
      id: 1,
      backend: "auto",
      device_id: 0,
      batch_size: 500,
      model_path: null,
    };

    const { invokeCommand } = await import("../../services/tauri");
    (invokeCommand as ReturnType<typeof vi.fn>).mockResolvedValue(
      JSON.stringify({ status: mockStatus, config: mockConfig })
    );

    await useGpuStore.getState().fetchStatus();

    const state = useGpuStore.getState();
    expect(state.status).toEqual(mockStatus);
    expect(state.config).toEqual(mockConfig);
    expect(state.loading).toBe(false);
  });
});
