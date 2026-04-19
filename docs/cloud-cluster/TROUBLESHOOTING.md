# 故障排查

## 本地 mock worker

可用内置无依赖 mock worker 验证远端调度链路：

```powershell
$env:WORKER_TOKEN="token-dev"
$env:PORT="8731"
node tools/mock-worker.mjs
```

然后在 Settings -> 集群节点中添加：

```text
Worker URL: http://127.0.0.1:8731
```

mock worker 默认要求：

```http
Authorization: Bearer token-dev
```

## 常见问题

### 注册一直 pending

- 确认 worker URL 带 `http://` 或 `https://`。
- 确认 `/health` 能返回 `worker_id`、`status` 和 `capabilities`。
- 确认客户端生成的 token 与 worker 运行时 token 一致。

### worker 显示 unavailable

- 手动点击“探测”查看错误消息。
- 检查防火墙和端口映射。
- 检查 HTTPS 证书或代理配置。

### batch 远端失败后回退本地

客户端会记录调度日志，并将失败 batch 回退到本地内置 worker。若希望强制只用本地执行，可在 Settings 中禁用远端 worker。

### 日志或结果重复

远端 worker 必须保持 `batch_id + attempt` 幂等。客户端以 `item_index` 作为结果合并的稳定位置，远端重复返回相同结果会被数据库唯一索引拦截。
