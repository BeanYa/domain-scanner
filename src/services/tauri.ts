import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const TAURI_RUNTIME_UNAVAILABLE =
  "当前页面没有连接到 Tauri 后端。请在 Domain Scanner 桌面应用中打开此页面，或使用 npm run tauri dev 启动完整应用。";

type TauriInternals = {
  invoke?: unknown;
};

export async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTestRuntime() && !hasTauriInvoke()) {
    throw new Error(TAURI_RUNTIME_UNAVAILABLE);
  }

  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeTauriError(error);
  }
}

export async function listenEvent<T>(event: string, handler: (payload: T) => void) {
  if (!isTestRuntime() && !hasTauriInvoke()) {
    console.warn(`[tauri] Skip listening to "${event}": ${TAURI_RUNTIME_UNAVAILABLE}`);
    return async () => {};
  }

  try {
    return await listen<T>(event, (e) => handler(e.payload));
  } catch (error) {
    throw normalizeTauriError(error);
  }
}

function hasTauriInvoke() {
  const internals = (globalThis as typeof globalThis & { __TAURI_INTERNALS__?: TauriInternals })
    .__TAURI_INTERNALS__;
  return typeof internals?.invoke === "function";
}

function isTestRuntime() {
  return import.meta.env.MODE === "test";
}

function normalizeTauriError(error: unknown) {
  if (isMissingTauriRuntimeError(error)) {
    return new Error(TAURI_RUNTIME_UNAVAILABLE);
  }
  return error;
}

function isMissingTauriRuntimeError(error: unknown) {
  if (!(error instanceof TypeError)) return false;
  return error.message.includes("__TAURI_INTERNALS__") || error.message.includes("reading 'invoke'");
}
