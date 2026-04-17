# 任务规划

## Phase 1: 桌面端文档冻结

- [x] 确认本项目 batch 概念和状态机。
- [x] 确认新增数据库表和迁移策略。
- [x] 确认新增 Tauri 命令。
- [x] 确认与 `domain-scanner-cloudserver` 的 API 边界。
- [x] 确认 UI 入口和信息架构。

验收：

- 本目录文档和 `domain-scanner-cloudserver` 文档术语一致。
- 两边可以独立开始实现。

冻结依据：

- `REQUIREMENTS.md` 固定 batch 计算、调度、结果合并、暂停/停止/恢复语义。
- `DATA_MODEL.md` 固定 `scan_batches`、`cluster_workers`、`scan_items` 扩展和 cursor 推进语义。
- `API_AND_COMMANDS.md` 与 `domain-scanner-cloudserver/docs/API_SPEC.md` 对齐远端 worker API、认证头、状态、结果和日志 cursor。
- `ARCHITECTURE.md` 固定 `TaskRunner -> BatchCoordinator -> Worker -> Merger` 的桌面端链路。
- Settings 中新增“集群节点”作为 worker 注册和管理入口；任务详情页主视图保持单任务体验。

## Phase 2: 本地任务 batch 化

- [ ] `ListGenerator` 支持 `[start_index, end_index)` 范围生成。
- [ ] 新增 `BatchPlan`、`BatchStatus`、`BatchResult`、`BatchLog` 类型。
- [ ] 新增 `scan_batches` repo。
- [ ] 抽出 `BatchExecutor`。
- [ ] 实现 `LocalEmbeddedWorker`。
- [ ] 实现 `BatchCoordinator`。
- [ ] `TaskRunner.start` 改为启动 coordinator。

验收：

- 本地扫描结果与现有实现一致。
- 暂停、恢复、停止行为兼容。
- 前端任务详情页无需理解 batch 也能正常显示。

## Phase 3: Worker 注册客户端

- [ ] 新增 `cluster_workers` 表和 repo。
- [ ] 实现 token 生成和 hash 存储。
- [ ] 实现安装命令生成。
- [ ] 实现 `create_worker_registration`。
- [ ] 实现 `poll_worker_registration`。
- [ ] 实现 `test_worker/list_workers/delete_worker`。
- [ ] 在 Settings 中新增集群节点管理 UI。

验收：

- 客户端能生成安装命令。
- Docker worker 上线后，客户端能探测成功并标记 available。
- worker 离线后能标记 unavailable。

## Phase 4: RemoteHttpWorker

- [ ] 实现远端 worker HTTP client。
- [ ] 支持 `/health`、`/capabilities`。
- [ ] 支持 batch submit/status/results/logs。
- [ ] 支持 pause/cancel。
- [ ] 远端错误分类和重试策略。

验收：

- mock worker 可被客户端注册和调度。
- 客户端能从远端 worker 拉取结果和日志。

## Phase 5: 多 worker 调度

- [ ] coordinator 支持多个 worker。
- [ ] 按 worker capabilities 分配 batch。
- [ ] 支持单 worker 多 batch。
- [ ] 支持 worker 掉线后 batch expired/retry。
- [ ] 支持本地 worker fallback 策略。

验收：

- 单任务可以分配到多个 worker。
- 任一 worker 离线后任务可继续或暂停并给出明确原因。
- 所有结果合并到同一 task/run。

## Phase 6: UI 和稳定性

- [ ] 任务详情页增加折叠 batch/worker 摘要。
- [ ] 增加批量调度日志。
- [ ] 增加状态通知。
- [ ] 增加压测工具或 mock worker。
- [ ] 增加文档化的故障排查。

验收：

- 大任务运行时 UI 稳定。
- 日志和结果分页性能可接受。
- 用户能判断 worker 注册、离线、失败和重试状态。
