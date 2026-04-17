import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import NewTask from "../../pages/NewTask";
import { useProxyStore } from "../../store/proxyStore";
import { useTaskStore } from "../../store/taskStore";
import type { ProxyConfig } from "../../types";

vi.mock("../../services/tauri", () => ({
  invokeCommand: vi.fn(),
}));

function makeOfflineProxy(): ProxyConfig {
  return {
    id: 7,
    name: "Broken Proxy",
    url: "http://127.0.0.1:8080",
    proxy_type: "http",
    username: null,
    password: null,
    is_active: false,
    status: "error",
    last_checked_at: "2026-01-01T00:00:00Z",
    last_error: "所有扫描 RDAP 端点均不可达",
  };
}

function makeUncheckedProxy(): ProxyConfig {
  return {
    id: 8,
    name: "Unchecked Proxy",
    url: "socks5://127.0.0.1:1080",
    proxy_type: "socks5",
    username: null,
    password: null,
    is_active: false,
    status: "pending",
    last_checked_at: null,
    last_error: null,
  };
}

describe("NewTask proxy selection", () => {
  beforeEach(async () => {
    useProxyStore.setState({
      proxies: [],
      loading: false,
      error: null,
      lastTestResult: null,
    });
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
        if (command === "list_proxies") {
          return JSON.stringify([makeOfflineProxy(), makeUncheckedProxy()]);
        }
        return undefined;
      }
    );
  });

  it("allows selecting an offline proxy and shows a warning", async () => {
    const user = userEvent.setup();
    render(
      <MemoryRouter>
        <NewTask />
      </MemoryRouter>
    );

    await user.click(screen.getByText("高级设置"));
    const proxySelect = screen.getByRole("combobox", { name: /代理选择/ });
    expect(proxySelect).toHaveAttribute("aria-expanded", "false");

    await user.click(proxySelect);
    expect(proxySelect).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByRole("listbox", { name: "代理选项" })).toBeInTheDocument();

    const offlineOption = await screen.findByRole("option", { name: /Broken Proxy/i });
    expect(offlineOption).toHaveTextContent("离线");

    await user.click(offlineOption);

    expect(screen.getByRole("combobox", { name: /Broken Proxy/i })).toBeInTheDocument();
    expect(screen.getByText(/已选择 离线 代理/)).toBeInTheDocument();
    expect(screen.getByText(/任务仍会使用它连接/)).toBeInTheDocument();
  });

  it("supports selecting an offline proxy with the keyboard", async () => {
    const user = userEvent.setup();
    render(
      <MemoryRouter>
        <NewTask />
      </MemoryRouter>
    );

    await user.click(screen.getByText("高级设置"));
    const proxySelect = screen.getByRole("combobox", { name: /代理选择/ });
    proxySelect.focus();

    await user.keyboard("{ArrowDown}{Enter}");

    expect(screen.getByRole("combobox", { name: /Broken Proxy/i })).toBeInTheDocument();
    expect(screen.getByText(/已选择 离线 代理/)).toBeInTheDocument();
  });

  it("allows selecting an unchecked proxy", async () => {
    const user = userEvent.setup();
    render(
      <MemoryRouter>
        <NewTask />
      </MemoryRouter>
    );

    await user.click(screen.getByText("高级设置"));
    const proxySelect = screen.getByRole("combobox", { name: /代理选择/ });

    await user.click(proxySelect);
    const uncheckedOption = await screen.findByRole("option", { name: /Unchecked Proxy/i });
    expect(uncheckedOption).toHaveTextContent("未检测");

    await user.click(uncheckedOption);

    expect(screen.getByRole("combobox", { name: /Unchecked Proxy/i })).toBeInTheDocument();
    expect(screen.getByText(/已选择 未检测 代理/)).toBeInTheDocument();
  });
});
