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
- 两个脚本都会把本项目编译临时文件定向到仓库根目录 `.build-tmp/`，避免占用系统盘 `%TEMP%`，同时不要把不断变化的临时文件放入 `src-tauri/`，以免开发监听器误触发重启；Cargo 并发在 `app/src-tauri/.cargo/config.toml` 中限制为 2，以降低 Windows 全量编译内存峰值。
- 不要在脚本中强杀 `node.exe` 或全局清理用户进程。

## 目录说明

```text
app/
  src/                         React 前端
    App.tsx                    根组件与页面路由装配
    hooks/useModManager.ts     集中调用 Tauri invoke，维护工作区状态、toast、确认弹层
    components/                AppShell、Titlebar、SetupGuide、ModCard、Feedback
    pages/                     启动台、Mod、配置方案、诊断、设置页面
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
docs/                          当前状态交接、视频更新方案等项目文档
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

前端只通过 `@tauri-apps/api/core` 的 `invoke()` 调用 Rust command；当前没有全局状态库、数据库或事件总线。主状态和工作区操作集中在 `app/src/hooks/useModManager.ts`，`App.tsx` 负责页面装配。

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

- `mod_manager.rs`：路径配置、Mod 扫描/ZIP 安装、本地 Mod 启停/删除、外部 Mod/DLL 注册、配置文件编辑、ME3 profile 生成、启动、不会启动游戏的 `get_launch_preflight`、日志、诊断、联机补丁状态和文件冲突分析
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
- `toggle_mod` 和 `uninstall_mod` 只能操作规范化后位于 `{game_path}\mods` 下的直属文件夹或 DLL；外部 Mod 必须只通过 `toggle_external_mod` / `remove_external_mod` 修改注册记录，不能重命名或删除原文件。
- 无 `.me3` 的资源包 Mod 会根据 `parts/`、`chr/`、`sfx/`、`map/`、`regulation.bin`、`.dcx` 等结构推断为 package。
- DLL-only Mod 会推断为 native。
- `.me3` 解析同时兼容 `[[packages]]` 和 `[[package]]`。
- 生成 `active-nightreign.me3` 时优先按 active profile 中 enabled 的 Mod 和 `loadOrder` 排序；如果 active profile 没有启用项，则回退到目录级 enabled 状态。
- package/native 去重时必须保留顺序，不能改成纯排序输出，否则会破坏 profile 加载顺序。

## 已实现功能

- 首次设置向导：选择游戏目录、ME3 目录、可选启动程序。
- 自定义暗色标题栏：拖动、最小化、最大化、关闭。
- 整体界面采用低噪声黑金“作战控制台”方向：窄侧栏、单层页面标题、明确主操作和紧凑状态面板；共享面板外观由 `app/src/index.css` 的 `.panel-card` 管理。
- 左侧配置方案列表：可折叠，可创建和激活方案。
- 配置方案启用状态同步：切换 Mod 会写入 active profile，切换方案会按方案快照同步目录启用状态。
- Mod 工作台：搜索、按类型过滤、状态卡片、潜在同名顶层项冲突提示。
- Mod 卡片：显示类型、说明、文件数量，支持开关和删除。
- 应用内 toast 和确认弹层，删除前确认。
- ZIP 安装到 `{game_path}\mods\`，带单根目录剥离、安全路径检查、重复目录自动编号。
- Rust 后端：配置读写、Mod 扫描、profile JSON 存储、ME3 profile 生成、bat 启动脚本、启动日志。
- 诊断页已支持生成/查看 profile、启动脚本、日志、启动诊断和真实文件级冲突分析。
- 启动台已支持不会真正启动游戏的启动前检查，检查游戏/ME3/启动目标、Steam、残留进程、联机组件、深夜解锁和启用 Mod；`error` 阻止启动，`warning` 只提醒。
- 配置方案页已支持拖拽调整已记录 Mod 的加载顺序。
- `last-launch.log` 超过 2 MB 时轮转为 `last-launch.log.1`；诊断页最多读取日志末尾 512 KB，避免把无限增长的日志整体复制进 WebView。

## 当前已知问题和待办

优先级较高：

- 根据 2026-07-16 对两条首发教学视频评论的复盘，下一批优先处理 Mod 结构/依赖健康检查，以及可供联机双方比较的 Mod 清单、版本指纹和加载顺序；不要先扩展在线下载等外围功能。
- ZIP 安装仍需要更多真实 Mod 测试，包括单根目录、多根目录、带 `.me3`、DLL-only、资源包-only、混合型。
- 配置方案已有拖拽排序，但仍缺少更直观的添加到方案、移出方案交互。
- 文件级冲突分析已有 50 万扫描文件和 1 万冲突结果的安全上限；后续可增加进度与取消能力。
- Mod 扫描、ZIP 解压和冲突分析已放入 blocking worker；后续可增加基于目录修改时间的扫描缓存。
- 当前窗口通过 `additionalBrowserArgs` 禁用 WebView2 GPU 加速，以降低静态管理界面的空闲私有内存；若未来加入重动画或 WebGL 页面，应重新测量流畅度、CPU 与内存后再决定是否保留。

后续功能方向：

- 导出并比较联机双方的 Mod 清单、文件指纹和加载顺序，降低“双方看到不同敌人/模型”的排障成本。
- 对安装包结构、缺失依赖、无法识别的文件类型给出明确健康提示，避免只显示“已启用”却实际未生效。
- 支持更直观的 Mod 添加/移出方案和批量状态更新。
- 为大型 Mod 扫描和冲突检测增加进度、取消与增量缓存。
- 完善路径校验：游戏目录应包含 `nightreign.exe`，ME3 目录应包含 `me3.exe` 或 `bin\me3.exe`。
- 在线浏览/下载 Mod 可以作为后续阶段，不要影响本地管理核心流程。

## 会话交接与状态来源

新会话进入仓库后按以下顺序恢复上下文：

1. 阅读本文件，确认长期架构、启动链路和安全边界。
2. 阅读 `docs/CURRENT_STATUS.md`，确认当前工作树、最新验证结果、尚未回归的风险和下一步顺序。
3. 涉及更新视频或评论需求时，再阅读 `docs/VIDEO_UPDATE_PLAN.md`；其中视频数据是带日期的快照，发布前必须重新获取。
4. 执行 `git status --short`，不要假设文档记录之后工作树没有继续变化。

`docs/CURRENT_STATUS.md` 是短期交接文件，应在重要功能完成、验证结论变化或准备新会话时更新；本文件只保存长期有效的规则。不要把普通开发日志或系统级 Skill 安装明细持续堆进 `AGENTS.md`。

## 开发约定

- 不要提交或批量修改 `mods/`、`me3/`、`reference/` 下的内容；这些是本地大文件和参考材料。
- 不要把开发机绝对路径写入源码或默认配置。可以在文档中作为示例说明。
- 修改 Tauri command 时同步检查 `app/src-tauri/src/lib.rs` 的注册和前端 `invoke()` 参数名。
- Tauri `invoke` 参数使用 camelCase，Rust command 参数用 snake_case 时由 Tauri 做映射，例如前端传 `modPath` 对应 Rust 的 `mod_path`。
- 前端 UI 以中文为主；修复或新增中文文本时确认文件编码为 UTF-8。
- 当前 UI 没有引入图标库，已有组件多用内联 SVG。若后续引入图标库，应统一替换，不要混用过多风格。
- Tailwind v4 主题变量集中在 `app/src/index.css`，新增颜色优先在那里定义。
- 默认窗口是 1160×760 且左侧栏约 216px；主要页面双栏应优先使用 `lg` 断点。不要用 `xl` 作为默认双栏入口，否则日常窗口会退化为长单列。
- 金色强调色只用于主操作、当前状态轨道和关键数字；普通辅助操作保持深色表面，避免所有卡片和按钮争夺注意力。
- 页面骨架复用 `LaunchPage.tsx` 导出的 `PageFrame`，面板优先复用 `.panel-card`，不要再叠加多层重复标题和边框。
- 项目本地 `.agents/skills/` 只保留项目专用 `desktop-app`；通用 UI、React、Tauri 和测试技能使用系统全局版本，不要再次复制进仓库。Skill 是工作清单和领域参考，不替代对现有代码、用户需求和实际运行结果的判断。
- 对文件系统操作要保守，删除 Mod 前应有确认；不要默认扫描或删除用户游戏目录里的非 Mod 文件。
- Tauri 前端只保留实际使用的 dialog 权限；不要重新开放 `fs:read-all`、`fs:write-all` 或 shell 执行权限，除非有明确需求、最小 scope 和安全审查。生产 CSP 不得设为 `null`。
- `Cargo.lock` 和 `app/src-tauri/icons/` 是可复现构建和打包所需文件，应提交；`target/`、`dist/` 继续忽略。
- 新增涉及 profile、Mod 安装、启动游戏的功能时，优先加 Rust 单元/集成测试，至少执行 `cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo check`、`cargo test`、`npm run build`、`npm run lint`。
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
