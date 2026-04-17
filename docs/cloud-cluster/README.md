# Domain Scanner Cloud Cluster Integration

本文档集描述 Domain Scanner 桌面端为支持云端 worker 集群需要做的改造。它和独立项目 `domain-scanner-cloudserver` 配套，但两边可以同步实现，也可以独立推进。

桌面端负责：

- 生成 worker 注册 token 和安装命令。
- 探测 Docker worker 是否上线。
- 维护 worker 列表、状态、能力和鉴权信息。
- 把任务拆成多个 batch。
- 分配 batch 到本地 worker 或远端 worker。
- 轮询 batch 状态、日志和结果。
- 合并结果到现有 SQLite 和任务结果视图。
- 保持现有任务详情页“像单个任务运行”的体验。

云端 worker 负责：

- 在 Docker 中运行。
- 接收 batch。
- 执行 RDAP/DNS 请求。
- 暴露状态、日志和结果增量接口。
- 不和其他 worker 通信。
- 不主动回调客户端。

## 文档索引

- [需求文档](REQUIREMENTS.md)
- [架构设计](ARCHITECTURE.md)
- [任务规划](TASK_PLAN.md)
- [数据模型](DATA_MODEL.md)
- [API 和命令设计](API_AND_COMMANDS.md)
- [语言和技术栈](LANGUAGE_AND_STACK.md)
- [约束规范](CONSTRAINTS.md)
- [测试计划](TEST_PLAN.md)

## 关联项目

独立 worker 服务文档项目：

```text
C:\universe\workspace\repo\domain-scanner-cloudserver
```

本项目文档以桌面端实现为主；`domain-scanner-cloudserver` 以 Docker worker 服务实现为主。
