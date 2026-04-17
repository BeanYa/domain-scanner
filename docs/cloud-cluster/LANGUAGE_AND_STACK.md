# 使用语言和技术栈

## 本项目桌面端

当前项目使用：

```text
Frontend: React 18 + TypeScript + Vite + Zustand
Desktop shell: Tauri 2
Backend: Rust 2021
Async runtime: Tokio
HTTP client: reqwest
Persistence: rusqlite + SQLite WAL
Logging: local rolling log files
```

集群客户端改造必须沿用当前技术栈。

## 新增依赖原则

优先使用已有依赖：

- `tokio`
- `reqwest`
- `serde`
- `serde_json`
- `uuid`
- `chrono`
- `futures`
- `tokio-util`

只有在必要时新增依赖。新增依赖必须满足：

- Rust stable 支持。
- Windows 桌面端兼容。
- 不引入大型运行时或平台服务依赖。
- 不破坏 Tauri 构建和发布流程。

## 前端约束

- 使用现有设计系统和 `DESIGN.md`。
- 不引入新的 UI 框架。
- 集群节点管理应复用现有页面风格、按钮和表格样式。
- 任务详情页主体验保持单任务视图。

## 独立 worker 项目

独立 worker 服务推荐使用：

```text
Rust + Tokio + Axum + reqwest + tracing + Docker
```

但该实现属于 `domain-scanner-cloudserver` 项目，不放入当前 Tauri 应用中。
