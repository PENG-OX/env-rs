# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

Env Switcher 是一个 Rust 实现的多版本环境管理工具，支持 Node.js 和 Java 的版本切换。通过 PowerShell Hook 自动检测路径变化来切换环境变量。

## 常用命令

```bash
# 构建发布版本
cargo build --release

# 运行测试
cargo test

# 运行 GUI 应用（构建后）
./target/release/env-switcher.exe
```

## 架构

### 核心模块 (src/lib.rs)

- `config.rs` - 配置管理，JSON 格式存储于 `%LOCALAPPDATA%\env-switcher\config.json`
- `matcher.rs` - 路径匹配逻辑，支持精确匹配和子目录继承
- `injector.rs` - PowerShell Hook 注入器，管理 PowerShell profile 中的 Hook 脚本
- `node.rs` - Node.js 版本管理，支持从 nodejs.org 下载和安装
- `java.rs` - Java/JDK 版本管理，支持从 Adoptium 下载和安装

### GUI (src/gui.rs)

使用 `iced` 框架实现，Windows 11 Fluent 暗色主题风格。入口点为 `src/main.rs`。

### Hook 机制

Hook 脚本注入到 PowerShell profile：

- PowerShell 7: `Documents\PowerShell\7\Microsoft.PowerShell_profile.ps1`
- Windows PowerShell: `Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1`

通过包装 `Set-Location`/`Push-Location`/`Pop-Location` 实现目录切换时自动切换环境。

## 关键配置结构

```json
{
  "node_versions": { "18.18.0": { "path": "...", "version": "18.18.0" } },
  "java_versions": { "11": { "path": "...", "version": "11" } },
  "path_mappings": [{ "path": "C:\\projects\\app", "node_version": "18.18.0", "java_version": "11" }]
}
```

## 平台特性

- Windows only（使用 winreg 注册表操作、PowerShell hook、Windows 路径）
- GUI 使用 iced + iced_aw（Bootstrap Icons）
- 中文 UI（使用系统黑体 `C:/Windows/Fonts/simhei.ttf`）


