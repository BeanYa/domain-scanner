---
name: windows-installer-build
overview: 配置 Tauri 2.0 NSIS 安装程序（中文支持+自定义安装模式+WebView2处理），更新 tauri.conf.json 和 deploy.ps1，然后执行构建输出安装包。
todos:
  - id: config-tauri-bundle
    content: 更新 tauri.conf.json 添加 NSIS + WebView2 + Windows 安装配置
    status: pending
  - id: create-nsis-hooks
    content: 创建 src-tauri/nsis-hooks.nsh 安装钩子脚本
    status: pending
  - id: update-deploy-script
    content: 更新 deploy.ps1：版本号动态读取、NSIS 产物验证、哈希校验
    status: pending
    dependencies:
      - config-tauri-bundle
  - id: build-installer
    content: 执行 tauri build 编译输出安装程序到 releases/
    status: pending
    dependencies:
      - config-tauri-bundle
      - create-nsis-hooks
      - update-deploy-script
---

## 用户需求

用户要求为 Windows 上的 Tauri 2.0 桌面应用寻找可用的安装程序框架，编写安装程序脚本，并编译输出安装程序。

## 产品概述

Domain Scanner 是一个 Tauri 2.0 + React + Rust 的域名扫描桌面应用，需要生成专业的 Windows 安装程序（.exe 安装包和 .msi 包），支持中文/英文双语、自定义安装模式、WebView2 自动安装等功能。

## 核心功能

- 配置 Tauri 2.0 内置的 NSIS 安装程序框架（主选），同时保留 MSI 输出
- NSIS 安装程序支持：中英双语选择、当前用户/所有用户安装模式、允许降级安装、安装后自动运行
- WebView2 运行时嵌入引导程序，确保离线环境可用
- 创建 NSIS 安装钩子脚本（安装前后自定义操作）
- 更新 deploy.ps1 部署脚本，完善构建和产物收集流程
- 执行实际构建，输出安装程序到 releases/ 目录

## 技术栈

- **安装程序框架**: Tauri 2.0 内置 NSIS（主选，生成 `-setup.exe`）+ WiX v3（保留，生成 `.msi`）
- **配置文件**: `tauri.conf.json` → `bundle.windows.nsis` / `bundle.windows.wix`
- **钩子脚本**: NSIS `.nsh` 格式
- **构建脚本**: PowerShell (`deploy.ps1`)
- **GPU 特性**: `--features gpu-directml`（AMD 5700XT）

## 实现方案

### 为什么选择 NSIS 作为主安装程序

- **Tauri 2.0 内置**：无需额外安装工具，`npm run tauri build` 自动调用
- **体积小**：NSIS 安装包比 MSI 更小，安装更快
- **多语言**：单一安装包包含所有语言，MSI 多语言需要生成多个包
- **灵活安装模式**：支持 `currentUser`/`perMachine`/`both`
- **中文支持**：原生支持 `SimpChinese` 语言

### 配置策略

1. 在 `tauri.conf.json` 的 `bundle` 下添加 `windows` 配置节点
2. NSIS 配置：中英双语 + 语言选择器 + both 安装模式 + 嵌入 WebView2 引导 + 允许降级
3. 通用配置：`webviewInstallMode` 设为 `embedBootstrapper`（+1.8MB，确保离线可用）
4. 创建 NSIS 钩子脚本处理安装前后的自定义逻辑

### 构建流程

```
tauri.conf.json 配置 → deploy.ps1 执行 → npm run tauri build --features gpu-directml
→ 产物：src-tauri/target/release/bundle/nsis/*.exe + msi/*.msi
→ 收集到 releases/ 目录
```

## 实现注意事项

- NSIS 工具由 Tauri bundler 自动下载（首次构建时），如网络受限需手动放置到 `%LOCALAPPDATA%/tauri/` 目录
- WiX Toolset 同理，首次构建自动下载
- `embedBootstrapper` 模式会增加约 1.8MB 安装包体积，但确保用户无 WebView2 时仍可安装
- NSIS 的 `languages` 使用 NSIS 语言名（`SimpChinese`），不是 BCP 47 格式
- `installMode: "both"` 在安装时会弹出 UAC 提示（选择"所有用户"时）

## 目录结构

```
d:\workspace\repo\domain-scanner-app\
├── src-tauri/
│   ├── tauri.conf.json          # [MODIFY] 添加 bundle.windows 配置（NSIS + WebView2）
│   └── nsis-hooks.nsh           # [NEW] NSIS 安装钩子脚本（安装前后自定义操作）
├── deploy.ps1                   # [MODIFY] 更新构建脚本，添加版本号读取和 NSIS 产物验证
└── releases/                    # [OUTPUT] 安装程序输出目录
    ├── Domain Scanner_0.1.0_x64-setup.exe   # NSIS 安装程序
    └── Domain Scanner_0.1.0_x64_en-US.msi   # MSI 安装包
```

### 文件变更详情

**`src-tauri/tauri.conf.json`** [MODIFY]

- 在 `bundle` 节点下添加 `windows` 子配置
- NSIS 配置：`languages: ["SimpChinese", "English"]`, `displayLanguageSelector: true`, `installMode: "both"`, `allowDowngrades: true`, `installerHooks: "nsis-hooks.nsh"`
- WebView2 配置：`webviewInstallMode: { type: "embedBootstrapper" }`
- `targets` 保持 `"all"`（同时输出 NSIS + MSI）

**`src-tauri/nsis-hooks.nsh`** [NEW]

- 定义 `NSIS_HOOK_PREINSTALL` 宏：检查旧版本安装路径、清理残留
- 定义 `NSIS_HOOK_POSTINSTALL` 宏：安装完成后注册文件关联或创建快捷方式
- 定义 `NSIS_HOOK_PREUNINSTALL` 宏：卸载前提示用户确认数据保留

**`deploy.ps1`** [MODIFY]

- 从 `package.json` 动态读取版本号替代硬编码 `v0.1.0`
- 添加 NSIS/WiX 工具下载检测和手动安装提示
- 构建完成后验证 NSIS 和 MSI 产物是否生成
- 增加产物哈希校验（SHA256）
- 添加 `-Target` 参数支持仅构建 NSIS 或 MSI