# 需求文档

## 背景

当前 Domain Scanner 主要在本地进程中执行扫描任务。为了应对网络质量、RDAP 限流、代理能力和大规模任务耗时问题，桌面端需要支持把一个扫描任务拆成多个 batch，分配给本地或远端 worker 执行。

本文档仅描述桌面端需要承担的功能。远端 Docker worker 的服务端实现由 `domain-scanner-cloudserver` 独立承担。

## 核心概念

设任务总请求量为 `T`，每个 batch 最多容纳 `m` 个请求：

```text
n = ceil(T / m)
batch_index = 0..n-1
start_index = batch_index * m
end_index = min(start_index + m, T)
```

约束：

```text
n * m >= T
n * m < T + m
start_index < T
```

本项目内“分片”等同于 `batch`。

## 功能需求

### 本地任务 batch 化

- 所有扫描任务先统一拆分为 batch。
- 本地扫描也通过 `LocalEmbeddedWorker` 执行 batch。
- 当前本地执行路径不再直接运行完整任务。
- 现有前端命令保持兼容。
- 现有任务详情页保持单任务视图。

### Worker 注册

- 用户可以在客户端添加远端 worker 地址。
- 客户端生成一次性注册 token。
- 客户端生成一条安装命令，例如：

```bash
bash <(curl -fsSL https://example.com/domain-scanner/worker_install.sh) -t token-xxx -p 8731
```

- 用户在服务器执行命令后，客户端轮询 worker `/health`。
- 超时时间内探测成功则注册为可用 worker。
- 超时则注册记录进入 `expired` 或 `error` 状态。

### Batch 调度

- 客户端是唯一调度者。
- worker 之间互不通信。
- 单 worker 可运行多个 batch。
- 调度时必须考虑 worker capabilities：
  - `max_running_batches`
  - `max_total_concurrency`
  - `max_batch_concurrency`
- 本地 worker 可作为 fallback，也可由用户禁用。

### 结果和日志合并

- 客户端轮询 worker 的结果和日志增量。
- 结果写入现有 `scan_items`。
- 日志写入现有任务日志文件。
- 本地持久化成功后才推进 cursor。
- 前端仍通过现有 `list_scan_items` 和 `get_logs` 获取数据。

### 暂停、停止、恢复

- 暂停任务时：
  - 停止提交新的 batch。
  - 对 running batch 发送 pause。
  - 已拉取结果继续合并。
- 停止任务时：
  - 对 active batch 发送 cancel。
  - run 标记为 stopped。
- 恢复任务时：
  - 跳过已成功完成的 batch。
  - 重试 failed/expired/retryable batch。

## 非功能需求

- 任务详情页主视图必须稳定，不因 batch 数量产生 UI 抖动。
- 大量 batch 不能导致前端加载所有 batch 明细。
- batch 轮询必须有节流和退避。
- 远端 worker token 不得明文出现在日志中。
- 本地扫描结果必须与旧扫描引擎保持行为兼容。

## 非目标

- 桌面端不实现远端 Docker worker 服务。
- 桌面端不作为公网可回调服务。
- 桌面端不让 worker 直接写本地数据库。
- 桌面端不展示每个请求的实时逐条流式 UI。
