// Tauri invoke/listen service - placeholder, will be implemented by parallel agent
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args);
}

export async function listenEvent<T>(event: string, handler: (payload: T) => void) {
  return listen<T>(event, (e) => handler(e.payload));
}
