# API 和命令设计

## 新增 Tauri 命令

### create_worker_registration

创建 pending worker 注册记录，生成 token 和安装命令。

输入：

```json
{
  "base_url": "http://1.2.3.4:8731",
  "name": "worker-01",
  "script_url": "https://example.com/domain-scanner/worker_install.sh",
  "port": 8731,
  "timeout_seconds": 600
}
```

输出：

```json
{
  "worker_id": "uuid",
  "status": "pending",
  "install_command": "bash <(curl -fsSL https://example.com/domain-scanner/worker_install.sh) -t token-xxx -p 8731",
  "expires_at": "2026-04-17T00:10:00Z"
}
```

### poll_worker_registration

轮询 pending worker 的 `/health`。

成功条件：

- HTTP 200。
- token 鉴权通过。
- worker_id 存在。
- capabilities 有效。
- version 满足最低要求。

### list_workers

列出 worker，包括 local 和 remote。

### test_worker

手动探测 worker `/health`。

### enable_worker / disable_worker

启用或禁用 worker。

### delete_worker

删除 worker。若 worker 正在运行 batch，必须拒绝或要求用户先停止任务。

### list_scan_batches

查询某个 task/run 下的 batch 摘要。

## 远端 Worker API

远端 API 由 `domain-scanner-cloudserver` 实现。桌面端只作为客户端调用：

```text
GET  /health
GET  /capabilities
POST /batches
GET  /batches/{batch_id}/status
GET  /batches/{batch_id}/results?after_seq=0&limit=500
GET  /batches/{batch_id}/logs?after_seq=0&limit=500
POST /batches/{batch_id}/pause
POST /batches/{batch_id}/cancel
```

所有远端请求必须带：

```http
Authorization: Bearer <token>
```

## 前端页面

建议在 Settings 中新增“集群节点”区域：

- 添加 worker。
- 输入 worker 地址、脚本 URL、端口。
- 显示安装命令。
- 显示注册倒计时。
- 显示 worker 状态和能力。
- 支持启用、禁用、删除、重新探测。
