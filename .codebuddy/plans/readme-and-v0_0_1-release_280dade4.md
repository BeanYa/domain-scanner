---
name: readme-and-v0.0.1-release
overview: 编写完善的 README.md（项目概述、功能、架构、技术栈、CI/CD、开发指南），将版本号改为 0.0.1，打 tag v0.0.1 并推送远程。
todos:
  - id: write-readme
    content: 编写 README.md（概述、功能、架构、技术、CI/CD、开发指南、Contributing）
    status: completed
  - id: bump-version
    content: 更新版本号 0.1.0 → 0.0.1（package.json / Cargo.toml / tauri.conf.json）
    status: completed
  - id: commit-tag-push
    content: 提交、打 tag v0.0.1、推送到远程
    status: completed
    dependencies:
      - write-readme
      - bump-version
---

## 用户需求

1. 编写完善的 README.md，包含：项目概述、项目功能、架构说明、采用技术、CI/CD、开发/Contributing 指南
2. 将项目版本号从 0.1.0 更新为 0.0.1（package.json / Cargo.toml / tauri.conf.json）
3. 打上 v0.0.1 的 tag 并推送到远程仓库

## 产品概述

Domain Scanner 是一款基于 Tauri 2.0 的域名扫描桌面应用，支持多 TLD 并行扫描、DNS 检测、AI 向量化筛选、LLM 智能分析、代理管理等功能。提供 NSIS 安装程序（中英双语），支持 DirectML/CUDA/CPU 三种 GPU 变体。

## 核心功能

- README.md 文档编写（项目概述、功能列表、架构图、技术栈、CI/CD 说明、开发指南、Contributing）
- 版本号同步更新（3 个文件）
- Git tag + 推送远程

## Tech Stack

- 文档格式：Markdown（GitHub Flavored）
- 架构图：Mermaid 语法
- 版本管理：Git + GitHub

## Implementation Approach

直接编写 README.md 并修改版本号，然后执行 git 操作。README 内容基于项目实际代码结构和功能，确保准确性。

## Directory Structure

```
d:\workspace\repo\domain-scanner-app\
├── README.md                     # [NEW] 项目文档
├── package.json                  # [MODIFY] version: "0.1.0" → "0.0.1"
├── src-tauri/
│   ├── Cargo.toml                # [MODIFY] version = "0.1.0" → "0.0.1"
│   └── tauri.conf.json           # [MODIFY] version: "0.1.0" → "0.0.1"
```