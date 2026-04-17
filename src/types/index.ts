// Type definitions - placeholder, will be expanded by parallel agent

export type TaskStatus = "pending" | "running" | "paused" | "stopped" | "completed";

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
  status: "pending" | "checking" | "available" | "unavailable" | "error";
  last_checked_at: string | null;
  last_error: string | null;
}

export interface ProxyEndpointCheck {
  key: string;
  label: string;
  url: string;
  reachable: boolean;
  http_status: number | null;
  response_time_ms: number | null;
  error_message: string | null;
}

export interface ProxyTestResult {
  proxy_id: number;
  success: boolean;
  status: ProxyConfig["status"];
  message: string;
  checked_at: string;
  reachable_count: number;
  total_count: number;
  endpoints: ProxyEndpointCheck[];
  notes: string[];
}

export interface LlmConfig {
  id: string;
  name: string;
  base_url: string;
  api_key?: string;
  model?: string;
  embedding_model: string | null;
  embedding_dim: number;
  is_default: boolean;
  is_template?: boolean;
  region?: string;
  category?: string;
  vector_ready?: boolean;
  notes?: string;
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
  log_type: "task" | "request";
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
  run_id: string | null;
  task_id: string;
  total: number;
  processed: number;
  percentage: number;
  backend: GpuBackend;
  speed_per_sec: number | null;
  estimated_remaining_secs: number | null;
  status: "idle" | "running" | "completed" | "failed" | "cancelled" | "interrupted";
  message: string | null;
  updated_at: string;
  started_at: string | null;
  finished_at: string | null;
}

export interface VectorizeRun {
  id: string;
  task_id: string;
  status: "running" | "completed" | "failed" | "cancelled" | "interrupted";
  backend: string;
  total_count: number;
  processed_count: number;
  skipped_existing: number;
  batch_size: number;
  embedding_dim: number;
  error_message: string | null;
  started_at: string;
  updated_at: string;
  finished_at: string | null;
}

export interface VectorStats {
  task_id: string;
  table_name: string;
  embedding_dim: number;
  total_available: number;
  vector_count: number;
  missing_count: number;
  coverage: number;
  running: boolean;
  last_run: VectorizeRun | null;
}

export interface VectorRecord {
  domain_id: number;
  task_id: string;
  domain: string;
  tld: string;
  vector_dim: number;
}
