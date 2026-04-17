# 约束规范

## 架构约束

- 客户端是唯一协调者。
- worker 之间不通信。
- worker 不主动回调客户端。
- batch 是最小远端调度单位。
- 本地执行也必须走 batch 协议。
- 远端 worker 不直接写客户端 SQLite。
- 远端 worker 不直接写任务结果文件。

## 兼容约束

- 保持现有前端核心命令兼容。
- 保持现有任务详情页主视图兼容。
- 保持现有 scan progress 事件语义兼容。
- 保持现有结果分页查询方式兼容。
- 保持现有日志查询方式兼容。

## 并发约束

- 任务请求并发大于 batch 请求量时，使用 batch 请求量。
- 实际 batch 并发必须小于等于 worker `max_batch_concurrency`。
- 单 worker 总并发必须小于等于 `max_total_concurrency`。
- 单 worker 同时运行 batch 数必须小于等于 `max_running_batches`。

## 安全约束

- token 不得出现在日志中。
- 安装命令在 UI 中展示时应明确这是敏感命令。
- 远端请求必须鉴权。
- 生产建议使用 HTTPS。
- 代理账号密码必须脱敏。

## 数据约束

- 结果合并必须幂等。
- cursor 只能在本地持久化成功后推进。
- 失败重试不得导致重复结果污染统计。
- 删除任务必须清理相关 batch 记录。

## 实现约束

- 不一次性把所有候选域名加载进内存。
- 不为每个请求创建 OS 线程。
- 不在前端加载全部 batch 明细。
- 不让远端 worker 依赖 Tauri。
