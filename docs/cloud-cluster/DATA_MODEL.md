# 数据模型

## 新增 scan_batches

不要复用现有 `task_batches`。现有 `task_batches` 表用于任务批量创建分组，新的扫描分片表建议命名为 `scan_batches`。

```sql
CREATE TABLE IF NOT EXISTS scan_batches (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(id),
  run_id TEXT NOT NULL REFERENCES task_runs(id),
  batch_index INTEGER NOT NULL,
  start_index INTEGER NOT NULL,
  end_index INTEGER NOT NULL,
  request_count INTEGER NOT NULL,
  status TEXT NOT NULL,
  worker_id TEXT,
  attempt INTEGER DEFAULT 0,
  completed_count INTEGER DEFAULT 0,
  available_count INTEGER DEFAULT 0,
  error_count INTEGER DEFAULT 0,
  result_cursor INTEGER DEFAULT 0,
  log_cursor INTEGER DEFAULT 0,
  lease_expires_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(task_id, run_id, batch_index)
);
```

索引：

```sql
CREATE INDEX IF NOT EXISTS idx_scan_batches_run_status
ON scan_batches(task_id, run_id, status);

CREATE INDEX IF NOT EXISTS idx_scan_batches_worker_status
ON scan_batches(worker_id, status);
```

## 新增 cluster_workers

```sql
CREATE TABLE IF NOT EXISTS cluster_workers (
  id TEXT PRIMARY KEY,
  name TEXT,
  base_url TEXT,
  worker_type TEXT NOT NULL DEFAULT 'remote',
  status TEXT NOT NULL,
  registration_token_hash TEXT,
  auth_token_ref TEXT,
  version TEXT,
  max_running_batches INTEGER,
  max_total_concurrency INTEGER,
  max_batch_concurrency INTEGER,
  current_running_batches INTEGER DEFAULT 0,
  current_concurrency INTEGER DEFAULT 0,
  install_command TEXT,
  expires_at TEXT,
  last_heartbeat_at TEXT,
  last_checked_at TEXT,
  last_error TEXT,
  enabled INTEGER DEFAULT 1,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

状态：

```text
pending / available / unavailable / error / expired / disabled
```

## 扩展 scan_items

建议新增：

```sql
ALTER TABLE scan_items ADD COLUMN batch_id TEXT;
ALTER TABLE scan_items ADD COLUMN worker_id TEXT;
```

推荐新增唯一索引：

```sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_scan_items_task_run_index_unique
ON scan_items(task_id, run_id, item_index);
```

短期兼容策略：

- 保留现有 `(task_id, run_id, domain)` 唯一约束。
- 合并时优先按 `item_index` 判重。
- 后续迁移稳定后，再调整唯一约束策略。

## Cursor 语义

- `result_cursor`: 已成功写入本地 `scan_items` 的最大 result seq。
- `log_cursor`: 已成功写入本地日志文件的最大 log seq。
- 只有本地持久化成功后才能推进 cursor。

## 本地 worker 记录

本地 worker 可作为一条特殊记录存在：

```text
worker_type = local
base_url = null
status = available
id = local
```

本地 worker 不需要 token，也不需要 Docker 安装命令。
