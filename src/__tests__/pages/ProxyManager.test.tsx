import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import ProxyManager from "../../pages/ProxyManager";
import { useProxyStore } from "../../store/proxyStore";
import type { ProxyConfig, ProxyTestResult } from "../../types";

vi.mock("../../services/tauri", () => ({
  invokeCommand: vi.fn(),
}));

function makeProxy(): ProxyConfig {
  return {
    id: 1,
    name: "Test Proxy",
    url: "http://127.0.0.1:8080",
    proxy_type: "http",
    username: null,
    password: null,
    is_active: false,
    status: "pending",
    last_checked_at: null,
    last_error: null,
  };
}

function makeTestResult(): ProxyTestResult {
  return {
    proxy_id: 1,
    success: true,
    status: "available",
    message: "所有扫描 RDAP 端点均可通过该代理访问",
    checked_at: "2026-01-01T00:00:00Z",
    reachable_count: 1,
    total_count: 1,
    endpoints: [
      {
        key: "arin",
        label: "ARIN",
        url: "https://rdap.arin.net/registry/domain/example.com",
        reachable: true,
        http_status: 200,
        response_time_ms: 123,
        error_message: null,
      },
    ],
    notes: ["RDAP 端点检测完成。"],
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

describe("ProxyManager proxy testing feedback", () => {
  beforeEach(async () => {
    useProxyStore.setState({
      proxies: [],
      loading: false,
      error: null,
      lastTestResult: null,
    });

    const { invokeCommand } = await import("../../services/tauri");
    (invokeCommand as ReturnType<typeof vi.fn>).mockReset();
  });

  it("shows an in-progress notice and then renders the proxy test result", async () => {
    const pendingTest = deferred<string>();
    const { invokeCommand } = await import("../../services/tauri");
    (invokeCommand as ReturnType<typeof vi.fn>).mockImplementation(
      async (command: string) => {
        if (command === "list_proxies") {
          return JSON.stringify([makeProxy()]);
        }
        if (command === "test_proxy") {
          return pendingTest.promise;
        }
        return undefined;
      }
    );

    const user = userEvent.setup();
    render(<ProxyManager />);

    expect(await screen.findByText("Test Proxy")).toBeInTheDocument();
    await user.click(screen.getByLabelText("测试代理"));

    expect(screen.getByText("正在检测代理")).toBeInTheDocument();
    expect(screen.getByText(/正在连接扫描 RDAP 端点/)).toBeInTheDocument();

    await act(async () => {
      pendingTest.resolve(JSON.stringify(makeTestResult()));
      await Promise.resolve();
    });

    expect(await screen.findByText("代理检测通过")).toBeInTheDocument();
    expect(screen.getByText(/端点通过 1\/1/)).toBeInTheDocument();
    expect(screen.getByText("最近一次代理端点检测")).toBeInTheDocument();
    expect(screen.getByText("ARIN")).toBeInTheDocument();
  });
});
