# AGENTS.md

本文件给后续在本仓库工作的编码代理使用。目标是让新会话可以快速接上当前项目状态，尤其是已经验证成功的 Nightreign + ME3 + SeamlessCoop 启动链路，并避免误动本地大型参考文件、ME3 工具和用户 Mod。

## 项目目标

这是一个面向《艾尔登法环：黑夜君临 / Elden Ring: Nightreign》的图形化 Mod 管理器，使用 Tauri v2 + React + Rust 构建桌面应用。应用负责管理用户选择的游戏目录、ME3(Mod Engine 3)目录、启动程序、Mod 列表和配置方案，最终通过 ME3 启动游戏并加载当前方案中的 Mod。

重要产品约定：

- 用户第一次使用时必须先选择游戏安装目录和 ME3 目录。
- 安装路径必须由用户自定义，不要在源码或默认配置里写死本机路径。
- 当前开发机示例游戏目录：`D:\Game\ERN\Elden.Ring.Nightreign.v20251217-P2P\Game`。
- 当前开发机 ME3 目录：`D:\Project\Game-create\EldenRing-Nightreign-ModManager\me3`。
- 当前用户的 SeamlessCoop/联机补丁文件已放在游戏根目录的 `SeamlessCoop\` 中，关键文件包括 `nrsc.dll`、`nighter.dll`、`nrsc_settings.ini`。
- 当前用户保留的联机补丁源目录示例：`D:\Game\ERN\联机补丁\Game`。不要自动覆盖游戏目录文件，除非用户明确要求。
- 正版 Steam 官方游玩与 SeamlessCoop/Spacewar 游玩不共用启动方式，也不共用存档。当前已验证成功的是第二种 SeamlessCoop/Spacewar 环境。

## 技术栈

- 桌面框架：Tauri 2.x
- 前端：React 19、TypeScript 6、Vite 8
- 样式：Tailwind CSS v4，主题在 `app/src/index.css` 的 `@theme` 中定义，没有 `tailwind.config` 文件
- 后端：Rust 2021、Tauri commands、`serde`、`serde_json`、`toml`、`dirs`、`zip`
- UI 语言：中文界面，暗色主题，自定义标题栏

## 常用命令

所有 npm/Tauri 命令都在 `app/` 目录下运行：

```powershell
cd app
npm run dev
npx tauri dev
npm run build
npm run lint
```

Rust 单独检查：

```powershell
cd app/src-tauri
cargo check
cargo test
```

生产打包：

```powershell
cd app
npx tauri build
```

根目录有两个辅助脚本：

```powershell
.\dev.bat
powershell -ExecutionPolicy Bypass -File .\start.ps1
```

脚本约定：

- `dev.bat` 用于快速进入 `app/` 并执行 `npx tauri dev`。
- `start.ps1` 是菜单脚本，包含 Tauri dev、Vite dev、build、lint、cargo check、cargo test 等操作。
- 不要在脚本中强杀 `node.exe` 或全局清理用户进程。

## 目录说明

```text
app/
  src/                         React 前端
    App.tsx                    根组件，集中调用 Tauri invoke，维护主状态、toast、确认弹层
    components/                Titlebar、Sidebar、SetupGuide、ModList、ModCard
    types/mod.ts               前端共享类型
    index.css                  Tailwind v4 主题与全局样式
  src-tauri/                   Rust/Tauri 后端
    src/lib.rs                 插件注册和 invoke handler 注册
    src/commands/mod_manager.rs 游戏路径、ME3 路径、启动 exe、Mod 扫描/安装/开关/删除/profile 生成/启动
    src/commands/profile.rs    配置方案 CRUD、激活方案、active profile 中 Mod 状态更新
    capabilities/default.json  Tauri 权限
    tauri.conf.json            窗口与打包配置
me3/                           本地 ME3 工具，已在 .gitignore 中忽略
mods/                          本地示例 Mod 和压缩包，已在 .gitignore 中忽略
reference/                     旧参考管理器，已在 .gitignore 中忽略
docs/                          项目文档预留
assets/                        项目资源预留
config/                        项目配置预留
```

本地 Nmodm 源码位置：

```text
D:\Project\Game-create\Nmodm
```

重要 Nmodm 参考文件：

- `D:\Project\Game-create\Nmodm\src\ui\pages\mods_page.py`
- `D:\Project\Game-create\Nmodm\src\ui\pages\quick_launch_page.py`
- `D:\Project\Game-create\Nmodm\src\utils\dll_manager.py`
- `D:\Project\Game-create\Nmodm\src\config\mod_config_manager.py`

## 当前架构

前端只通过 `@tauri-apps/api/core` 的 `invoke()` 调用 Rust command；当前没有全局状态库、数据库或事件总线。主状态集中在 `app/src/App.tsx`。

配置存储在系统配置目录：

```text
{dirs::config_dir()}/nightreign-mod-manager/config.json
{dirs::config_dir()}/nightreign-mod-manager/profiles/*.json
{dirs::config_dir()}/nightreign-mod-manager/active_profile.txt
{dirs::config_dir()}/nightreign-mod-manager/active-nightreign.me3
{dirs::config_dir()}/nightreign-mod-manager/launch/launch-nightreign.bat
{dirs::config_dir()}/nightreign-mod-manager/launch/last-launch.log
```

当前开发机通常展开为：

```text
C:\Users\34590\AppData\Roaming\nightreign-mod-manager\config.json
C:\Users\34590\AppData\Roaming\nightreign-mod-manager\active-nightreign.me3
C:\Users\34590\AppData\Roaming\nightreign-mod-manager\launch\launch-nightreign.bat
C:\Users\34590\AppData\Roaming\nightreign-mod-manager\launch\last-launch.log
```

Rust command 当前注册在 `app/src-tauri/src/lib.rs`，主要分两组：

- `mod_manager.rs`：`get_game_path`、`set_game_path`、`get_me3_path`、`set_me3_path`、`get_launch_exe_path`、`set_launch_exe_path`、`get_mods_dir`、`scan_mods`、`get_mod_info`、`install_mod_from_zip`、`uninstall_mod`、`toggle_mod`、`generate_me3_profile`、`launch_game`、`diagnose_launch_game`
- `profile.rs`：`get_profiles`、`create_profile`、`delete_profile`、`activate_profile`、`get_active_profile`、`update_profile`、`update_active_profile_mod`

## 已验证成功的启动链路

本轮关键结果：用户确认游戏已成功启动，Mod 也已成功加载。

成功链路不是把 `nrsc_launcher.exe` 直接交给 ME3，而是：

```text
管理器启动按钮
-> generate_me3_profile()
-> 写入 active-nightreign.me3
-> 写入 launch\launch-nightreign.bat
-> cmd /K launch-nightreign.bat，使用 CREATE_NEW_CONSOLE 打开独立控制台
-> me3.exe launch --exe nightreign.exe --skip-steam-init --online --game nightreign -p active-nightreign.me3
-> ME3 注入并加载 profile 中的 packages/natives
-> SeamlessCoop/nrsc.dll、nighter.dll 和资源包 Mod 生效
```

已验证成功的命令形态：

```text
cd /d "{me3_path}\bin"
"{me3_path}\bin\me3.exe" launch --exe "{game_path}\nightreign.exe" --skip-steam-init --online --game nightreign -p "{config_dir}\active-nightreign.me3"
```

关键经验：

- ME3 管理器入口是 `me3.exe`，通常位于 `{me3_path}\bin\me3.exe`，也兼容用户直接选择 `bin` 目录。不要再按旧文档查找 `me3-launcher.exe` 作为外部入口。
- `me3-launcher.exe` 是 ME3 内部 injector，不应作为本管理器调用入口。
- SeamlessCoop/Spacewar 环境下，直接双击 `nrsc_launcher.exe` 可以正常进游戏，但通过 ME3 加载 Mod 时不能把 `nrsc_launcher.exe` 作为 `--exe` 传入。
- 如果用户设置了 `nrsc_launcher.exe`，当前实现会在 ME3 启动链路中自动改用同目录的 `nightreign.exe`，并通过 profile 加载 `SeamlessCoop\nrsc.dll`。
- `generate_me3_profile()` 会自动检测游戏根目录下的 `SeamlessCoop\nrsc.dll` 和 `SeamlessCoop\nighter.dll`。存在时加入 `[[natives]]`，其中 `nrsc.dll` 必须 `load_early = true`。
- 当前启动参数 `--skip-steam-init --online --game nightreign` 是参考 Nmodm 后在用户当前环境中验证成功的组合。
- 不要再使用 `cmd /C start "Nightreign-ME3" ...` 这类写法。Windows 对 `start` 的 title 参数解析容易导致类似找不到 `VNightreign-ME3\` 或 `WNightreign-ME3\` 的错误。
- 当前稳定写法是生成 bat，然后 `cmd /K <bat>`，并通过 Windows `CREATE_NEW_CONSOLE` 打开独立控制台。
- 从 `dev.bat` 启动 Tauri 后，ME3 输出不一定会出现在原终端。启动诊断应优先看 `launch\last-launch.log` 和独立控制台。

当前生成的 `.me3` 关键形态：

```toml
profileVersion = "v1"

[[supports]]
game = "nightreign"

[[natives]]
path = "D:\\Game\\ERN\\Elden.Ring.Nightreign.v20251217-P2P\\Game\\SeamlessCoop\\nrsc.dll"
optional = false
enabled = true
load_before = []
load_after = []
load_early = true

[[natives]]
path = "D:\\Game\\ERN\\Elden.Ring.Nightreign.v20251217-P2P\\Game\\SeamlessCoop\\nighter.dll"
optional = false
enabled = true
load_before = []
load_after = []
load_early = false

[[packages]]
enabled = true
path = "D:\\Game\\ERN\\Elden.Ring.Nightreign.v20251217-P2P\\Game\\mods\\duchessunmask"
load_after = []
load_before = []
```

启动失败时优先检查：

- `C:\Users\34590\AppData\Roaming\nightreign-mod-manager\launch\last-launch.log`
- `C:\Users\34590\AppData\Roaming\nightreign-mod-manager\launch\launch-nightreign.bat`
- `C:\Users\34590\AppData\Roaming\nightreign-mod-manager\active-nightreign.me3`
- ME3 自身日志，例如 `C:\Users\34590\AppData\Local\garyttierney\me3\data\logs\active-nightreign\*.log`
- bat 中是否仍然是 `--exe nightreign.exe`，并包含 `--skip-steam-init`、`--online`、`--game nightreign`、`-p active-nightreign.me3`

## Mod 扫描与 profile 生成

- `scan_mods` 扫描 `{game_path}\mods\` 下的子目录，同时识别普通目录和 `.disabled` 目录。
- `toggle_mod` 使用目录重命名到 `.disabled` 的策略实现启用/禁用。
- 无 `.me3` 的资源包 Mod 会根据 `parts/`、`chr/`、`sfx/`、`map/`、`regulation.bin`、`.dcx` 等结构推断为 package。
- DLL-only Mod 会推断为 native。
- `.me3` 解析同时兼容 `[[packages]]` 和 `[[package]]`。
- 生成 `active-nightreign.me3` 时优先按 active profile 中 enabled 的 Mod 和 `loadOrder` 排序；如果 active profile 没有启用项，则回退到目录级 enabled 状态。
- package/native 去重时必须保留顺序，不能改成纯排序输出，否则会破坏 profile 加载顺序。

## 已实现功能

- 首次设置向导：选择游戏目录、ME3 目录、可选启动程序。
- 自定义暗色标题栏：拖动、最小化、最大化、关闭。
- 左侧配置方案列表：可折叠，可创建和激活方案。
- 配置方案启用状态同步：切换 Mod 会写入 active profile，切换方案会按方案快照同步目录启用状态。
- Mod 工作台：搜索、按类型过滤、状态卡片、潜在同名顶层项冲突提示。
- Mod 卡片：显示类型、说明、文件数量，支持开关和删除。
- 应用内 toast 和确认弹层，删除前确认。
- ZIP 安装到 `{game_path}\mods\`，带单根目录剥离、安全路径检查、重复目录自动编号。
- Rust 后端：配置读写、Mod 扫描、profile JSON 存储、ME3 profile 生成、bat 启动脚本、启动日志。
- 启动失败诊断命令 `diagnose_launch_game` 已注册，但 UI 还未暴露成按钮。

## 当前已知问题和待办

优先级较高：

- ZIP 安装仍需要更多真实 Mod 测试，包括单根目录、多根目录、带 `.me3`、DLL-only、资源包-only、混合型。
- 冲突提示目前主要是前端基于顶层同名项的启发式提示，不是真实文件级覆盖分析。
- 配置方案已有启用状态和 `loadOrder` 字段，但 UI 还没有完整的拖拽排序、添加到方案、移出方案交互。
- 启动日志已有，但 UI 还没有“打开日志”“复制启动命令”“诊断启动”按钮。
- `diagnose_launch_game` 已注册，建议后续在 UI 中暴露，失败后把命令、stdout、stderr、日志路径直接展示给用户。
- 需要考虑 `last-launch.log` 的清理或轮转，避免长期无限增长。

后续功能方向：

- 支持 Mod 添加/移出方案、拖拽排序、真实 profile 级启用状态。
- 后端递归生成 Mod 文件清单，实现真实文件级冲突检测。
- 增加日志查看器、启动脚本查看器、复制诊断信息。
- 完善路径校验：游戏目录应包含 `nightreign.exe`，ME3 目录应包含 `me3.exe` 或 `bin\me3.exe`。
- 在线浏览/下载 Mod 可以作为后续阶段，不要影响本地管理核心流程。

## 开发约定

- 不要提交或批量修改 `mods/`、`me3/`、`reference/` 下的内容；这些是本地大文件和参考材料。
- 不要把开发机绝对路径写入源码或默认配置。可以在文档中作为示例说明。
- 修改 Tauri command 时同步检查 `app/src-tauri/src/lib.rs` 的注册和前端 `invoke()` 参数名。
- Tauri `invoke` 参数使用 camelCase，Rust command 参数用 snake_case 时由 Tauri 做映射，例如前端传 `modPath` 对应 Rust 的 `mod_path`。
- 前端 UI 以中文为主；修复或新增中文文本时确认文件编码为 UTF-8。
- 当前 UI 没有引入图标库，已有组件多用内联 SVG。若后续引入图标库，应统一替换，不要混用过多风格。
- Tailwind v4 主题变量集中在 `app/src/index.css`，新增颜色优先在那里定义。
- 对文件系统操作要保守，删除 Mod 前应有确认；不要默认扫描或删除用户游戏目录里的非 Mod 文件。
- 新增涉及 profile、Mod 安装、启动游戏的功能时，优先加 Rust 单元/集成测试，至少执行 `cargo check`、`cargo test`、`npm run build`、`npm run lint`。
- Windows 文件操作不要用字符串拼接构造删除/移动命令；优先使用 Rust 标准库或 PowerShell 原生命令，并确认目标路径。
- 当前 worktree 可能包含大量未提交改动。不要 `git reset --hard`，不要 revert 与当前任务无关的文件。

## 参考资料

- ME3 文档：<https://me3.help/en/latest/>
- me3-manager 项目：<https://github.com/2Pz/me3-manager>
- me3-manager 帮助：<https://me3-manager.github.io/me3-manager-help/>
- Nmodm 项目：<https://github.com/QykXczj/Nmodm>
- 本地 Nmodm 源码：`D:\Project\Game-create\Nmodm`
- 本地参考：`reference\Me3_Manager_1.4.5`、`reference\Nmodm_v3.1.4`

## 参考记忆

历史 Claude/Codex 会话记忆位于：

```text
C:\Users\34590\.claude\projects\d--Project-Game-create\memory\session_2026-05-22_mod_manager.md
C:\Users\34590\.claude\projects\d--Project-Game-create\memory\session_2026-05-23_mod_manager_codex.md
C:\Users\34590\.claude\projects\d--Project-Game-create\memory\MEMORY.md
```

如果 PowerShell 输出出现中文乱码，以源码实际 UTF-8 内容和当前仓库状态为准。
