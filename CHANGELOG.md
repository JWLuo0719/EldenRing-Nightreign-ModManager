# Changelog

本项目的显著变更会记录在此文件中。

格式参考 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，版本号参考 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)。

## [0.1.0] - 2026-05-23

### Added

- 初始 Tauri v2 + React + Rust 桌面应用。
- 首次设置向导：配置游戏目录、ME3 目录和启动程序。
- Mod 仓库：扫描 `Game\mods`、安装 ZIP、启用/停用、删除到回收站。
- 外部 Mod 支持：注册外部 Mod 目录和外部 DLL。
- DLL 配置编辑：可在管理器内修改已识别的 JSON/INI 配置文件。
- 配置方案管理：创建、激活、删除和保存方案状态。
- 启动台：生成 ME3 profile 并通过 ME3 启动 Nightreign。
- SeamlessCoop/Spacewar 启动链路支持。
- 深夜解锁 `nighter.dll` 检测与加载。
- 诊断页面：查看 ME3 profile、启动脚本、日志和文件冲突。
- v0.1.0 Windows 安装包：NSIS `.exe` 与 MSI。

### Changed

- 将早期单页 UI 重构为启动台、Mod 仓库、配置方案、诊断和设置页面。
- 扩大默认窗口尺寸并优化 Mod 仓库和启动台布局。
- 生成 profile 时保留加载顺序并按规范化路径去重。
- 解析 Mod 自带 `.me3` 时跳过不存在的 package/native 路径。

### Fixed

- 修复 `Game\mods` 根目录 DLL 不进入管理器的问题。
- 修复 `nighter.dll` 可能重复写入 ME3 profile 的问题。
- 修复 Windows `\\?\` 路径前缀导致外部 Mod 加载不稳定的问题。
- 添加启动前进程保护，避免残留 `nightreign.exe` 或 `me3-launcher.exe` 抢占文件句柄。

[0.1.0]: https://github.com/JWLuo0719/EldenRing-Nightreign-ModManager/releases/tag/v0.1.0
