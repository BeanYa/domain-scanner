---
name: github-ci-cd-release
overview: 创建 GitHub Actions workflow，使用 tauri-action 构建 Domain Scanner 安装程序，区分 AMD (DirectML) 和 NVIDIA (CUDA) 分别编译输出到 GitHub Release。
todos:
  - id: config-tauri-bundle
    content: 更新 tauri.conf.json 添加 NSIS + WebView2 + Windows 安装配置
    status: completed
  - id: create-nsis-hooks
    content: 创建 src-tauri/nsis-hooks.nsh 安装钩子脚本
    status: completed
  - id: create-release-workflow
    content: 创建 .github/workflows/release.yml（DirectML/CUDA/CPU 三变体矩阵构建）
    status: completed
    dependencies:
      - config-tauri-bundle
  - id: update-deploy-script
    content: 更新 deploy.ps1：版本号动态读取 + SHA256 哈希校验 + NSIS 产物验证
    status: completed
    dependencies:
      - config-tauri-bundle
  - id: push-and-verify
    content: 推送代码并验证 GitHub Actions 工作流运行
    status: completed
    dependencies:
      - create-release-workflow
      - update-deploy-script
---

## 用户需求

用户要求编写 GitHub Actions CI/CD 工作流，利用 GitHub Actions 自动构建并输出 Release。需要区分 NVIDIA 和 AMD 显卡，分别编译不同 GPU 变体的安装程序。

## 产品概述

Domain Scanner 是一个 Tauri 2.0 + React + Rust 的域名扫描桌面应用。需要建立完整的 CI/CD 流水线：推送 tag 触发自动构建，输出包含 DirectML(AMD)、CUDA(NVIDIA)、CPU 三种 GPU 变体的 Windows 安装程序到 GitHub Release。

## 核心功能

- 配置 Tauri NSIS 安装程序（中英双语、安装模式可选、WebView2 嵌入）
- 创建 GitHub Actions 工作流，按 GPU 变体分矩阵构建
- 构建三种 Windows 安装包变体：DirectML/AMD、CUDA/NVIDIA、CPU-only
- 自动创建 GitHub Release 并上传所有安装包
- 更新本地部署脚本适配 NSIS 配置

## 技术栈

- **CI/CD**: GitHub Actions + `tauri-apps/tauri-action@v0`
- **安装程序**: Tauri 2.0 内置 NSIS（主选）+ WiX v3（MSI 保留）
- **构建矩阵**: windows-latest x 3 GPU变体 (DirectML / CUDA / CPU)
- **触发方式**: 推送 tag `v*` + 手动 workflow_dispatch

## 实现方案

### 构建矩阵设计

使用 GitHub Actions 的 `strategy.matrix` 并行构建三种 GPU 变体，所有变体产物上传到同一个 Release：

| Job ID | Cargo Features | 适用场景 | 产物后缀标识 |
| --- | --- | --- | --- |
| `windows-directml` | `--features gpu-directml` | AMD/Intel GPU on Windows | `_DirectML` |
| `windows-cuda` | `--features gpu-cuda` | NVIDIA GPU | `_CUDA` |
| `windows-cpu` | (default, no features) | 纯CPU，使用远程API | `_CPU` |


### 工作流架构

```
推送 tag v* / 手动触发
  ├─ Job: windows-directml  →  tauri build --features gpu-directml  →  上传到 Release
  ├─ Job: windows-cuda      →  tauri build --features gpu-cuda      →  上传到 Release
  └─ Job: windows-cpu       →  tauri build                          →  上传到 Release
```

- `fail-fast: false`：一个变体失败不影响其他
- `tauri-action@v0`：自动调用 tauri build，生成 NSIS+MSI，创建/追加 Release
- Rust cache (`swatinem/rust-cache@v2`)：加速后续构建
- `tagName: v__VERSION__`：tauri-action 自动替换为应用版本号

### NSIS 安装程序配置

在 `tauri.conf.json` 的 `bundle.windows` 中配置：

- `nsis.languages`: ["SimpChinese", "English"]
- `nsis.displayLanguageSelector`: true
- `nsis.installMode`: "both"（当前用户/所有用户可选）
- `nsis.allowDowngrades`: true
- `webviewInstallMode`: { type: "embedBootstrapper" }（+1.8MB，离线可用）

### 关键技术决策

1. **tauri-action 的 `args` 传递 cargo features**：`args: --features gpu-directml` 直接传递给 `tauri build`
2. **多 Job 共享同一 Release**：tauri-action 自动检测已有 Release 并追加产物（通过相同 tagName）
3. **产物区分**：NSIS 默认产物名包含平台信息，但不含 GPU 变体标识。需要在 Release Body 中明确标注各变体适用场景

## 实现注意事项

- tauri-action 首次运行会自动下载 NSIS/WiX 工具，可能因网络问题失败，需设置重试 (`retryAttempts: 2`)
- GPU 变体的产物文件名默认相同（都会覆盖），需要通过 `releaseAssetNamePattern` 或在构建后重命名来区分
- `ort` crate 的 GPU features 在 CI 环境中可能需要额外的系统库（DirectML 是 Windows 自带的，CUDA 需要运行时但编译时只需头文件）
- Rust cache 应仅缓存 `src-tauri/target` 目录，key 应包含 cargo features 以避免缓存冲突

## 目录结构

```
d:\workspace\repo\domain-scanner-app\
├── .github/
│   └── workflows/
│       └── release.yml            # [NEW] GitHub Actions CI/CD 工作流
├── src-tauri/
│   ├── tauri.conf.json            # [MODIFY] 添加 bundle.windows NSIS + WebView2 配置
│   └── nsis-hooks.nsh             # [NEW] NSIS 安装钩子脚本
└── deploy.ps1                     # [MODIFY] 版本号动态读取 + SHA256 哈希校验
```

### 文件变更详情

**`.github/workflows/release.yml`** [NEW]

GitHub Actions 工作流定义文件，包含：

- 触发条件：推送 tag `v*` + workflow_dispatch
- 3 个并行构建 Job（DirectML / CUDA / CPU）
- 每个 Job：checkout → setup node → setup rust → rust cache → npm install → tauri-action build
- tauri-action 配置：tagName/releaseName/releaseBody/releaseDraft
- 产物重命名步骤：区分 GPU 变体后上传到同一 Release
- Release Body 包含各变体说明

**`src-tauri/tauri.conf.json`** [MODIFY]

- 在 `bundle` 下添加 `windows` 子节点
- NSIS: languages/displayLanguageSelector/installMode/allowDowngrades/installerHooks
- WebView2: webviewInstallMode = embedBootstrapper

**`src-tauri/nsis-hooks.nsh`** [NEW]

- NSIS_HOOK_PREINSTALL：检查旧版本路径
- NSIS_HOOK_POSTINSTALL：安装完成提示
- NSIS_HOOK_PREUNINSTALL：卸载前确认

**`deploy.ps1`** [MODIFY]

- 从 package.json 动态读取版本号（替代硬编码 v0.1.0）
- 构建完成后生成 SHA256 哈希文件
- 产物验证：检查 NSIS 和 MSI 是否生成