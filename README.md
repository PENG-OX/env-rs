# Env Switcher

一个基于 Rust 的多版本环境管理工具，支持 Node.js 和 Java 的无限项目环境隔离。

> 当前版本为 CLI 版本。GUI 版本可使用 Tauri 或 egui 后续开发。

## 功能特性

- ✅ **无限项目支持** -  
- ✅ **无配置文件污染** -  
- ✅ **自动环境切换** - PowerShell hook 自动检测路径变化并切换环境变量
- ✅ **子目录继承** - 配置路径的子目录自动继承父目录的环境设置
- ✅ **作用域隔离** - 仅在配置路径下生效，离开路径自动恢复系统环境变量
- ✅ **Node.js 管理** - 支持多版本切换
- ✅ **Java 管理** - 支持多 JDK 版本管理

## 快速开始

### 构建

```bash
cargo build --release
```


### Hook 安装位置

Hook 脚本会注入到：
- PowerShell 7: `Documents\PowerShell\7\Microsoft.PowerShell_profile.ps1`
- Windows PowerShell: `Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1`

## 配置示例

配置文件位于 `%LOCALAPPDATA%\env-switcher\config.json`:

```json
{
  "node_versions": {
    "18.18.0": {
      "path": "C:\\env\\node-18.18.0",
      "version": "18.18.0"
    },
    "20.9.0": {
      "path": "C:\\env\\node-20.9.0",
      "version": "20.9.0"
    }
  },
  "java_versions": {
    "11": {
      "path": "C:\\env\\jdk-11",
      "version": "11"
    },
    "21": {
      "path": "C:\\env\\jdk-21",
      "version": "21"
    }
  },
  "path_mappings": [
    {
      "path": "C:\\projects\\web-app-v1",
      "node_version": "18.18.0",
      "java_version": "11"
    },
    {
      "path": "C:\\projects\\backend-service",
      "node_version": "20.9.0",
      "java_version": "21"
    }
  ]
}
```


## 使用示例

```powershell
# 进入配置的项目目录，自动切换环境
cd C:\projects\web-app-v1
# 输出：🟢 Node: 18.18.0
# 输出：🟢 Java: 11

# 进入子目录，继承父目录配置
cd C:\projects\web-app-v1\src\components
# 环境变量保持不变

# 离开配置路径，恢复系统环境
cd C:\other\project
# 输出：⚪ Using system environment
```

## PowerShell 函数

Hook 安装后可使用以下函数：

- `Get-EnvSwitcherStatus` - 查看当前环境状态

```powershell
PS> Get-EnvSwitcherStatus

CurrentPath  : C:\projects\web-app-v1
NodeHome     : C:\env\node-18.18.0
JavaHome     : C:\env\jdk-11
ConfigPath   : C:\Users\xxx\AppData\Local\env-switcher\config.json
```


## 注意事项

1. **首次安装 Hook 后需要重启 PowerShell** 才能生效
2. **执行策略** - 如果 Hook 不生效，可能需要设置 `Set-ExecutionPolicy RemoteSigned`
3. **环境变量作用域** - 仅在当前 PowerShell 会话生效，不影响系统全局环境变量

## License

MIT
