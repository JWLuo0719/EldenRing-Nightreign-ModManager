# Changelog

本项目的显著变更会记录在此文件中。

格式参考 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，版本号参考 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)。

## [Unreleased]

## [0.2.1] - 2026-08-16

### Added

- 设置页新增始终可见的 MMV 社区兼容模式开关；不再要求先安装或注册特定 Mod 才能预先设置。
- 启动前检查现在显示实际采用的作者 `.me3` 文件名，便于确认启动计划而非仅看 Mod 卡片启用状态。

### Fixed

- 修复深夜解锁状态只检查固定根目录的问题；现在按本次生成启动计划中的 `nighter.dll` 判断是否会加载，并能区分“已安装”与“本次会加载”。
- 修复训练场（Boss Arena）同目录 Sandbox/Progression 双 Profile 被错误叠加而无法生成启动配置的问题；仅在恰好一个候选满足单一 `regulation.bin` 门槛时自动采用该安全 Profile。

## [0.2.0] - 2026-08-02

### Added

- 启动台新增“启动前检查（不会启动游戏）”，一次检查游戏目录、ME3、实际启动目标、Steam、残留进程、SeamlessCoop、OnlineFix/Spacewar、深夜解锁和当前启用 Mod。
- 启动前检查按错误、提醒、通过分级；仅错误会阻止启动，未安装可选联机组件不会误拦截离线使用。
- 区分纯正版 Steam、正版 Steam + Seamless、Spacewar + Seamless 和未知混合环境，并按环境生成正确启动参数。
- 支持作者 `.me3` 启动配置的语义保真导入、MMV 社区 Seamless 兼容模式和唯一玩法数据文件门禁。
- 新增服装 Mod 结构检查、无效外观 ID 风险提醒、队友视角 `_l` 资源配对检查和失效外部路径重新定位。
- 新增脱敏联机一致性清单，可比较双方完整 Mod 内容树、DLL、加载顺序、玩法数据、中文层和 Seamless 设置。
- 安装目录附带“【双击】--更多游戏内容.url”快捷方式。

### Changed

- 统一整体 UI 为低噪声黑金“作战控制台”风格，收敛卡片层级、强调色和页面标题密度。
- 侧栏宽度由 256px 收敛至 216px，活动页面改用金色轨道而非整块高亮，并压缩重复的工作区说明。
- 启动台重排为方案概览、启动控制、检查报告、运行环境和加载清单；主操作与辅助操作不再混排。
- Mod 仓库、配置方案、诊断和设置页在默认 1160×760 窗口下使用有效双栏，避免 `xl` 断点造成的长单列和过度滚动。
- 统一面板、数字、焦点可见状态和轻量页面进入动效；修正停用开关圆点定位不明确的问题。
- 玩家界面优先使用“启动配置、资源型 Mod、功能插件、玩法数据文件、联机一致性清单”等中文说法，技术原词保留在高级详情。
- 游戏启动改为无终端窗口的后台方式，退出后不会留下可见命令窗口；启动日志统一写入诊断目录。
- 暂时隐藏尚未完成发布回归的“应用联机补丁”入口。

### Security

- 将本地 Mod 的启停和删除限制在 `Game\mods` 直属项，拒绝任意路径操作。
- 校验配置方案 ID，阻止路径穿越。
- 移除未使用的 Tauri fs/shell/log/tray 能力并启用生产 CSP。

### Performance

- Mod 扫描、ZIP 解压和文件冲突分析改为 blocking worker，避免阻塞 IPC。
- 启动日志增加 2 MB 轮转，诊断页只读取最后 512 KB。
- 文件冲突索引改用紧凑的 Mod 下标，并限制最大扫描量、结果量和 UI 渲染量。
- DLL 类型探测改为命中即停止的递归扫描，并限制最大目录深度。
- 降低 Rust dev/test 调试信息级别，减少后续构建缓存和链接内存。
- 对当前静态管理界面禁用 WebView2 GPU 加速，降低空闲时的私有内存占用。

### Fixed

- 修复激活配置方案时可能重命名外部 Mod 原文件的问题。
- 修复 ESLint 误扫描 `src-tauri/target` 的问题。
- 开发脚本将编译临时文件移出 `src-tauri/`，避免文件监听器误触发 Tauri 重启。
- 明确提示启动诊断会真实启动游戏，并将残留进程检测改为精确进程名与 PID 输出。
- 修复外部 Mod 路径失效后界面仍显示已启用、实际启动配置却没有加载内容的问题。
- 修复大型玩法包因少量人物部件而被误判为服装扩展的问题。
- 统一前端、Tauri 和 Rust 包版本为 `0.2.0`。

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

[Unreleased]: https://github.com/JWLuo0719/EldenRing-Nightreign-ModManager/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/JWLuo0719/EldenRing-Nightreign-ModManager/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/JWLuo0719/EldenRing-Nightreign-ModManager/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/JWLuo0719/EldenRing-Nightreign-ModManager/releases/tag/v0.1.0
