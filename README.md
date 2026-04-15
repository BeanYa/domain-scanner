# Domain Scanner

<div align="center">

**高性能域名扫描桌面应用**

基于 Tauri 2.0 + React + Rust 构建的域名可用性扫描工具，支持多 TLD 并行扫描、GPU 加速向量嵌入、LLM 智能分析。

[![CI/CD Release](https://github.com/BeanYa/domain-scanner/actions/workflows/release.yml/badge.svg)](https://github.com/BeanYa/domain-scanner/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

[English](#english) · [中文](#中文)

</div>

---

## 中文

### 项目概述

Domain Scanner 是一款专业的域名扫描桌面应用，帮助用户快速发现可注册的域名。通过 DNS 查询批量检测域名可用性，结合 AI 向量化筛选和 LLM 智能分析，高效挖掘有价值的域名资源。

### 核心功能

| 功能 | 说明 |
|------|------|
| **多 TLD 扫描** | 单任务支持多顶级域名（.com / .net / .org 等），笛卡尔积生成域名列表 |
| **批量 DNS 检测** | 异步并发 DNS 查询，高速检测域名注册状态 |
| **GPU 加速嵌入** | 本地 ONNX Runtime 推理，支持 DirectML (AMD) / CUDA (NVIDIA) / CPU 三种模式 |
| **远程 API 嵌入** | 支持 OpenAI 等远程 Embedding API，无需本地 GPU |
| **向量相似度筛选** | 基于 sqlite-vec 向量搜索，按语义相似度过滤域名 |
| **LLM 智能分析** | 集成大语言模型，自动评估域名商业价值和潜在用途 |
| **代理管理** | SOCKS5/HTTP 代理池，支持轮换和负载均衡 |
| **数据导出** | CSV / JSON / TXT 多格式导出，含注册商、NS 等详细信息 |
| **实时监控** | 任务进度、日志流、扫描速度等实时仪表盘 |

### 大规模单任务设计

本项目当前优化重点不是“同时存在多少个任务”，而是“单个任务可能需要扫描上百万到上千万个候选地址”。围绕这个场景，运行链路做了以下约束：

- **有界并发**：扫描引擎使用固定上限并发，避免因为候选空间指数级膨胀导致内存和 socket 失控。
- **结果分批持久化**：请求结果先在内存中做小批量缓冲，再批量写入 SQLite；这样既保留高吞吐，也避免每个请求都独占数据库锁。
- **进度节流推送**：后端按时间窗口推送 `scan-progress`，而不是为每条结果都触发一次前端刷新；详情页会在短周期内只刷新当前页数据。
- **结果分页**：任务详情页结果固定分页，每页 10 条，默认展示最新完成的数据，避免百万级结果直接灌进浏览器。
- **日志文件化**：运行日志不再依赖数据库表做高频写入，而是按 `task_id/run_id` 落盘；单文件 10MB 滚动，前端只读取尾部窗口。
- **非权威 fallback 语义**：RDAP 成功才会归类为“可用/已注册”；RDAP 失败后即使 DNS fallback 给出“可能可用/可能已注册”，结果也会归类为 `error`，只作为提示，不作为最终判定。

### 架构

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (React 18)                   │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐  │
│  │Dashboard │ │TaskList  │ │NewTask   │ │Settings   │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └─────┬─────┘  │
│       │            │            │              │         │
│  ┌────┴────────────┴────────────┴──────────────┴─────┐  │
│  │           Zustand Stores + React Router            │  │
│  └─────────────────────┬────────────────────────────┘  │
│                        │ Tauri IPC                      │
├────────────────────────┼────────────────────────────────┤
│                   Backend (Rust)                        │
│  ┌─────────────────────┴────────────────────────────┐  │
│  │              Commands (IPC Layer)                 │  │
│  │  task · scan · filter · export · vector · llm ·  │  │
│  │  gpu · proxy · batch · log                        │  │
│  └─────────────────────┬────────────────────────────┘  │
│                        │                                │
│  ┌─────────┐ ┌────────┴───────┐ ┌──────────────────┐  │
│  │  Scanner │ │   Embedding    │ │      LLM         │  │
│  │ Engine   │ │ Local/GPU/API  │ │   Client         │  │
│  └────┬────┘ └───────┬───────┘ └────────┬─────────┘  │
│       │              │                   │              │
│  ┌────┴──────────────┴───────────────────┴──────────┐  │
│  │     Persistence Layer (SQLite + Rolling Files)    │  │
│  │   task · task_run · scan_item · vector · filter   │  │
│  │   proxy · gpu · llm · batch · task logs (*.log)   │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### 扫描链路

```text
ListGenerator
    -> bounded FuturesUnordered workers
    -> DomainChecker (RDAP primary / DNS fallback hint)
    -> in-memory result buffer
    -> SQLite batch flush (scan_items + progress checkpoint)
    -> throttled progress event
    -> TaskDetail page-1 refresh
```

说明：

- `scan_items` 负责长期结果存储与分页查询。
- `task_runs` 负责每次 rerun 的独立统计和历史隔离。
- `task-logs-*/<task_id>/run-<run_id>.log` 负责高频日志持久化。
- 前端详情页只追踪当前选中 run，不会尝试把整次扫描结果加载进内存。

### 技术栈

#### 前端

| 技术 | 用途 |
|------|------|
| [React 18](https://react.dev/) | UI 框架 |
| [TypeScript](https://www.typescriptlang.org/) | 类型安全 |
| [Vite](https://vitejs.dev/) | 构建工具 |
| [Tailwind CSS](https://tailwindcss.com/) | 样式系统 |
| [Zustand](https://zustand-demo.pmnd.rs/) | 状态管理 |
| [React Router](https://reactrouter.com/) | 路由管理 |
| [Recharts](https://recharts.org/) | 数据可视化 |
| [Lucide Icons](https://lucide.dev/) | 图标库 |
| [react-window](https://github.com/bvaughn/react-window) | 虚拟列表 |

#### 后端

| 技术 | 用途 |
|------|------|
| [Rust](https://www.rust-lang.org/) | 核心语言 |
| [Tauri 2.0](https://v2.tauri.app/) | 桌面框架 |
| [Tokio](https://tokio.rs/) | 异步运行时 |
| [rusqlite](https://github.com/rusqlite/rusqlite) | SQLite 数据库 |
| [sqlite-vec](https://github.com/asg017/sqlite-vec) | 向量搜索扩展 |
| [ONNX Runtime](https://onnxruntime.ai/) | GPU 推理 (DirectML/CUDA) |
| [hickory-resolver](https://github.com/hickory-dns/hickory-dns) | DNS 解析 |
| [reqwest](https://github.com/seanmonstar/reqwest) | HTTP 客户端 |

### CI/CD

项目使用 GitHub Actions 自动构建和发布。推送 `v*` 标签时自动触发：

```
git tag v0.0.1 && git push origin v0.0.1
        │
        ▼
┌─────────────────────────────────────────┐
│         GitHub Actions Workflow          │
│                                         │
│  ┌──────────────┐  ┌──────────────┐    │
│  │  DirectML    │  │    CUDA      │    │
│  │  (AMD/Intel) │  │  (NVIDIA)    │    │
│  └──────┬───────┘  └──────┬───────┘    │
│         │                  │            │
│  ┌──────┴───────┐          │            │
│  │    CPU       │          │            │
│  │  (No GPU)    │          │            │
│  └──────┬───────┘          │            │
│         │    │             │            │
│         ▼    ▼             ▼            │
│  ┌──────────────────────────────┐      │
│  │   GitHub Release (Draft)     │      │
│  │   + SHA256 Checksums         │      │
│  └──────────────────────────────┘      │
└─────────────────────────────────────────┘
```

**下载指南**：

| 文件 | GPU 变体 | 适用用户 |
|------|----------|---------|
| `*_DirectML-setup.exe` | AMD / Intel GPU | AMD Radeon, Intel Arc |
| `*_CUDA-setup.exe` | NVIDIA GPU | NVIDIA GeForce / RTX |
| `*_CPU-setup.exe` | 纯 CPU | 无本地 GPU，使用远程 API |

### 开发

#### 环境要求

- **Node.js** >= 18
- **Rust** (via [rustup](https://rustup.rs/))
- **Visual Studio Build Tools** 2022 (C++ 桌面开发工作负载)

#### 快速开始

```bash
# 克隆仓库
git clone https://github.com/BeanYa/domain-scanner.git
cd domain-scanner

# 安装前端依赖
npm install

# 开发模式运行（默认 CPU 模式）
npm run tauri dev

# 开发模式运行（DirectML GPU 模式）
npm run tauri dev -- --features gpu-directml
```

#### 构建

```bash
# 构建 CPU 版本
npm run tauri build

# 构建 DirectML (AMD GPU) 版本
npm run tauri build -- --features gpu-directml

# 构建 CUDA (NVIDIA GPU) 版本
npm run tauri build -- --features gpu-cuda

# 或使用一键部署脚本
.\deploy.ps1 -GpuMode directml   # AMD GPU
.\deploy.ps1 -GpuMode cuda       # NVIDIA GPU
.\deploy.ps1 -GpuMode cpu        # CPU Only
```

构建产物位于 `src-tauri/target/release/bundle/`，使用 `deploy.ps1` 会自动收集到 `releases/` 目录。

#### 测试

```bash
# 前端测试
npm run test

# Rust 测试
cd src-tauri && cargo test

# 前端测试覆盖率
npm run test:coverage
```

#### 项目结构

```
domain-scanner-app/
├── src/                          # 前端源码 (React + TypeScript)
│   ├── pages/                    # 页面组件
│   │   ├── Dashboard.tsx         # 仪表盘
│   │   ├── TaskList.tsx          # 任务列表
│   │   ├── TaskDetail.tsx        # 任务详情
│   │   ├── NewTask.tsx           # 新建任务
│   │   ├── FilterResults.tsx     # 筛选结果
│   │   ├── VectorizePage.tsx     # 向量化
│   │   ├── ProxyManager.tsx      # 代理管理
│   │   └── Settings.tsx          # 设置
│   ├── components/Layout/        # 布局组件
│   ├── hooks/                    # 自定义 Hooks
│   ├── store/                    # Zustand 状态管理
│   ├── services/                 # Tauri IPC 封装
│   └── types/                    # TypeScript 类型
│
├── src-tauri/                    # 后端源码 (Rust + Tauri)
│   └── src/
│       ├── commands/             # Tauri IPC 命令层
│       ├── db/                   # SQLite 数据访问层
│       ├── models/               # 数据模型
│       ├── scanner/              # 核心扫描引擎
│       │   ├── engine.rs         # 扫描调度
│       │   ├── domain_checker.rs # RDAP / DNS 检测
│       │   ├── list_generator.rs # 域名列表生成
│       │   ├── task_runner.rs    # 任务生命周期与后台运行
│       │   └── signature.rs      # 任务签名去重
│       ├── embedding/            # 向量嵌入模块
│       │   ├── gpu_detector.rs   # GPU 检测
│       │   ├── local_model.rs    # 本地 ONNX 推理
│       │   └── remote_api.rs     # 远程 API
│       ├── llm/                  # LLM 集成
│       ├── proxy/                # 代理管理
│       └── export/               # 数据导出
│
├── .github/workflows/            # GitHub Actions
│   └── release.yml               # CI/CD 发布工作流
└── deploy.ps1                    # Windows 一键部署脚本
```

### Contributing

欢迎贡献！请遵循以下流程：

1. **Fork** 本仓库
2. 创建功能分支：`git checkout -b feature/your-feature`
3. 提交更改：`git commit -m 'feat: add your feature'`
4. 推送分支：`git push origin feature/your-feature`
5. 提交 **Pull Request**

#### 代码规范

- **Rust**：遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)，使用 `cargo fmt` 和 `cargo clippy`
- **TypeScript**：遵循项目 ESLint 配置，使用 Prettier 格式化
- **提交信息**：遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范
  - `feat:` 新功能
  - `fix:` 修复 Bug
  - `docs:` 文档更新
  - `refactor:` 代码重构
  - `test:` 测试相关
  - `chore:` 构建/工具变更

#### 开发建议

- 新增功能请同时添加单元测试
- 修改 Rust 后端需确保 `cargo test` 通过
- 修改前端需确保 `npm run build` 无错误
- 保持前后端类型定义一致（`src/types/index.ts` 与 `src-tauri/src/models/`）

---

<a id="english"></a>

## English

### Overview

Domain Scanner is a professional domain scanning desktop application that helps users quickly discover available domain names. It combines batch DNS queries with AI-powered vector filtering and LLM analysis to efficiently identify valuable domain resources.

### Key Features

| Feature | Description |
|---------|-------------|
| **Multi-TLD Scanning** | Scan multiple top-level domains (.com / .net / .org, etc.) in a single task with Cartesian product generation |
| **Batch DNS Detection** | Asynchronous concurrent DNS queries for high-speed availability checking |
| **GPU-Accelerated Embedding** | Local ONNX Runtime inference with DirectML (AMD) / CUDA (NVIDIA) / CPU modes |
| **Remote API Embedding** | Support for OpenAI and other remote Embedding APIs without local GPU |
| **Vector Similarity Filtering** | Semantic similarity filtering via sqlite-vec vector search |
| **LLM Analysis** | Integrated LLM for automated domain valuation and use-case analysis |
| **Proxy Management** | SOCKS5/HTTP proxy pool with rotation and load balancing |
| **Data Export** | CSV / JSON / TXT export with registrar, NS, and other details |
| **Real-time Monitoring** | Live dashboard with task progress, log streaming, and scan speed |

### Large Single-Task Execution Model

The primary scaling concern is not the number of tasks in the UI. It is a single task scanning millions or tens of millions of generated candidates through a bounded proxy pool.

- **Bounded concurrency** keeps request fan-out under control.
- **Buffered persistence** batches scan results before writing to SQLite, reducing lock contention.
- **Throttled UI updates** send progress snapshots on a short interval instead of repainting per result.
- **Paginated result views** keep the browser on 10 rows per page, focused on the latest completed entries.
- **Rolling file logs** store per-run request diagnostics outside SQLite with a 10MB rolling file strategy.
- **Authoritative status policy** treats only successful RDAP responses as final `available/unavailable`; DNS fallback after RDAP failure is retained as a hint but classified as `error`.

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (React 18)                   │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐  │
│  │Dashboard │ │TaskList  │ │NewTask   │ │Settings   │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └─────┬─────┘  │
│       │            │            │              │         │
│  ┌────┴────────────┴────────────┴──────────────┴─────┐  │
│  │           Zustand Stores + React Router            │  │
│  └─────────────────────┬────────────────────────────┘  │
│                        │ Tauri IPC                      │
├────────────────────────┼────────────────────────────────┤
│                   Backend (Rust)                        │
│  ┌─────────────────────┴────────────────────────────┐  │
│  │              Commands (IPC Layer)                 │  │
│  │  task · scan · filter · export · vector · llm ·  │  │
│  │  gpu · proxy · batch · log                        │  │
│  └─────────────────────┬────────────────────────────┘  │
│                        │                                │
│  ┌─────────┐ ┌────────┴───────┐ ┌──────────────────┐  │
│  │  Scanner │ │   Embedding    │ │      LLM         │  │
│  │ Engine   │ │ Local/GPU/API  │ │   Client         │  │
│  └────┬────┘ └───────┬───────┘ └────────┬─────────┘  │
│       │              │                   │              │
│  ┌────┴──────────────┴───────────────────┴──────────┐  │
│  │     Persistence Layer (SQLite + Rolling Files)    │  │
│  │   task · task_run · scan_item · vector · filter   │  │
│  │   proxy · gpu · llm · batch · task logs (*.log)   │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Tech Stack

#### Frontend

| Technology | Purpose |
|-----------|---------|
| [React 18](https://react.dev/) | UI Framework |
| [TypeScript](https://www.typescriptlang.org/) | Type Safety |
| [Vite](https://vitejs.dev/) | Build Tool |
| [Tailwind CSS](https://tailwindcss.com/) | Styling |
| [Zustand](https://zustand-demo.pmnd.rs/) | State Management |
| [React Router](https://reactrouter.com/) | Routing |
| [Recharts](https://recharts.org/) | Data Visualization |
| [Lucide Icons](https://lucide.dev/) | Icon Library |

#### Backend

| Technology | Purpose |
|-----------|---------|
| [Rust](https://www.rust-lang.org/) | Core Language |
| [Tauri 2.0](https://v2.tauri.app/) | Desktop Framework |
| [Tokio](https://tokio.rs/) | Async Runtime |
| [rusqlite](https://github.com/rusqlite/rusqlite) | SQLite Database |
| [sqlite-vec](https://github.com/asg017/sqlite-vec) | Vector Search Extension |
| [ONNX Runtime](https://onnxruntime.ai/) | GPU Inference (DirectML/CUDA) |
| [hickory-resolver](https://github.com/hickory-dns/hickory-dns) | DNS Resolution |
| [reqwest](https://github.com/seanmonstar/reqwest) | HTTP Client |

### CI/CD

The project uses GitHub Actions for automated builds and releases. Pushing a `v*` tag triggers the workflow:

```bash
git tag v0.0.1 && git push origin v0.0.1
```

Three GPU variants are built in parallel and uploaded to a single GitHub Release (draft):

| File | GPU Variant | Target Users |
|------|-------------|-------------|
| `*_DirectML-setup.exe` | AMD / Intel GPU | AMD Radeon, Intel Arc |
| `*_CUDA-setup.exe` | NVIDIA GPU | NVIDIA GeForce / RTX |
| `*_CPU-setup.exe` | CPU Only | No local GPU, uses remote API |

SHA256 checksums are automatically generated and attached.

### Development

#### Prerequisites

- **Node.js** >= 18
- **Rust** (via [rustup](https://rustup.rs/))
- **Visual Studio Build Tools** 2022 (C++ Desktop workload)

#### Quick Start

```bash
# Clone the repository
git clone https://github.com/BeanYa/domain-scanner.git
cd domain-scanner

# Install frontend dependencies
npm install

# Run in development mode (CPU mode)
npm run tauri dev

# Run with DirectML GPU mode
npm run tauri dev -- --features gpu-directml
```

#### Build

```bash
# Build CPU version
npm run tauri build

# Build DirectML (AMD GPU) version
npm run tauri build -- --features gpu-directml

# Build CUDA (NVIDIA GPU) version
npm run tauri build -- --features gpu-cuda

# Or use the one-click deploy script
.\deploy.ps1 -GpuMode directml   # AMD GPU
.\deploy.ps1 -GpuMode cuda       # NVIDIA GPU
.\deploy.ps1 -GpuMode cpu        # CPU Only
```

#### Testing

```bash
# Frontend tests
npm run test

# Rust tests
cd src-tauri && cargo test

# Frontend test coverage
npm run test:coverage
```

### Contributing

Contributions are welcome! Please follow this process:

1. **Fork** this repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Commit your changes: `git commit -m 'feat: add your feature'`
4. Push the branch: `git push origin feature/your-feature`
5. Submit a **Pull Request**

#### Code Guidelines

- **Rust**: Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/), run `cargo fmt` and `cargo clippy`
- **TypeScript**: Follow project ESLint config, format with Prettier
- **Commit Messages**: Follow [Conventional Commits](https://www.conventionalcommits.org/)
  - `feat:` New feature
  - `fix:` Bug fix
  - `docs:` Documentation
  - `refactor:` Code refactoring
  - `test:` Testing
  - `chore:` Build/tooling changes

### License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
