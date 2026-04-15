// Type definitions - placeholder, will be expanded by parallel agent

export type TaskStatus = "pending" | "running" | "paused" | "completed";

export type ScanMode =
  | { type: "regex"; pattern: string }
  | { type: "wildcard"; pattern: string }
  | { type: "llm"; config_id: string; prompt: string }
  | { type: "manual"; domains: string[] };

export type GpuBackend = "auto" | "cuda" | "directml" | "rocm" | "coreml" | "cpu" | "remote";

export interface Task {
  id: string;
  batch_id: string | null;
  name: string;
  signature: string;
  status: TaskStatus;
  scan_mode: ScanMode;
  config_json: string;
  /** Multiple TLDs this task scans (e.g. [".com", ".net", ".org"]) */
  tlds: string[];
  prefix_pattern: string | null;
  concurrency: number;
  proxy_id: number | null;
  total_count: number;
  completed_count: number;
  completed_index: number;
  available_count: number;
  error_count: number;
  created_at: string;
  updated_at: string;

  /** Get the primary TLD for backward compat display */
  primaryTld(): string;
}

export interface TaskBatch {
  id: string;
  name: string;
  task_count: number;
  created_at: string;
}

export interface GpuConfig {
  id: number;
  backend: GpuBackend;
  device_id: number;
  batch_size: number;
  model_path: string | null;
}

export interface GpuStatus {
  backend: GpuBackend;
  available: boolean;
  device_name: string | null;
  vram_total_mb: number | null;
  vram_used_mb: number | null;
}

export interface ScanItem {
  id: number;
  task_id: string;
  run_id: string;
  domain: string;
  tld: string;
  item_index: number;
  status: "pending" | "checking" | "available" | "unavailable" | "error";
  is_available: boolean | null;
  query_method: string | null;
  response_time_ms: number | null;
  error_message: string | null;
  checked_at: string | null;
}

export interface ProxyConfig {
  id: number;
  name: string | null;
  url: string;
  proxy_type: "http" | "https" | "socks5";
  username: string | null;
  password: string | null;
  is_active: boolean;
}

export interface LlmConfig {
  id: string;
  name: string;
  base_url: string;
  api_key: string;
  model: string;
  embedding_model: string | null;
  embedding_dim: number;
  is_default: boolean;
}

export interface BatchCreateResult {
  created: number;
  skipped: number;
  task_ids: string[];
  skipped_tlds: string[];
}

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
}

export interface LogEntry {
  id: number;
  task_id: string;
  run_id: string | null;
  level: "debug" | "info" | "warn" | "error";
  message: string;
  created_at: string;
}

export interface TaskRun {
  id: string;
  task_id: string;
  run_number: number;
  status: TaskStatus;
  total_count: number;
  completed_count: number;
  available_count: number;
  error_count: number;
  started_at: string;
  finished_at: string | null;
}

export interface VectorProgress {
  task_id: string;
  completed: number;
  total: number;
  eta_seconds: number | null;
}
