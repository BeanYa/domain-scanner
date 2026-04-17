# 测试计划

## Rust 单元测试

### Batch 规划

- `T = 1000, m = 100` 创建 10 个 batch。
- `T = 1001, m = 100` 创建 11 个 batch。
- `T = 1, m = 100` 创建 1 个 batch。
- `m = 1` 时创建 `T` 个 batch。
- 不创建空 batch。
- 最后一个 batch 的 `end_index = T`。

### ListGenerator 范围

- 从指定 `start_index` 开始。
- 不越过 `end_index`。
- 多 TLD 下 item_index 与 domain 对应关系稳定。
- 手动模式和正则模式都可范围生成。

### ResultMerger

- 重复结果不重复插入。
- 写入失败不推进 cursor。
- 写入成功推进 cursor。
- available/error 统计正确。

### 状态聚合

- 全部 succeeded -> completed。
- 部分 running -> running。
- failed 重试耗尽 -> paused。
- stopped 优先级高于 batch 状态。

## 前端测试

- Settings 能生成安装命令。
- pending worker 显示倒计时。
- worker available/unavailable 状态展示正确。
- 任务详情页无 batch 数据时保持旧体验。
- batch 摘要折叠区不会影响结果表和日志面板。

## 集成测试

- mock worker 注册成功。
- mock worker token 错误。
- mock worker 超时。
- mock worker 返回 capabilities。
- coordinator 提交 batch。
- coordinator 拉取结果和日志。
- pause/cancel 请求发出。

## 回归测试

- 现有任务创建。
- 现有本地扫描。
- 暂停恢复。
- 停止后 rerun。
- 结果分页。
- 日志分页。
- 代理设置。
