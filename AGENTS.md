# AGENTS.md

本文件给后续在本仓库工作的编码代理使用。目标是让新会话可以快速接上当前项目状态，并避免误动本地的大型参考文件、ME3 工具和用户 Mod。

## 项目目标

这是一个面向《艾尔登法环：黑夜君临 / Elden Ring: Nightreign》的图形化 Mod 管理器，使用 Tauri v2 + React + Rust 构建桌面应用。应用负责管理用户选择的游戏目录、ME3(Mod Engine 3)目录、Mod 列表和配置方案，最终通过 ME3 启动游戏。

重要产品约定：

- 用户第一次使用时必须先选择游戏安装目录和 ME3 目录。
- 安装路径必须用户自定义，不要在代码里写死本机路径。
- 当前开发机示例游戏目录：`D:\Game\ERN\Elden.Ring.Nightreign.v20251217-P2P\Game`。
- 当前开发机 ME3 目录：`D:\Project\Game-create\EldenRing-Nightreign-ModManager\me3`。
- 当前示例 Mod：`mods\【更多地图】More Map Variations-578-2-0-5-1774306525`。

## 技术栈

- 桌面框架：Tauri 2.x
- 前端：React 19、TypeScript 6、Vite 8
- 样式：Tailwind CSS v4，主题在 `app/src/index.css` 的 `@theme` 中定义，没有 `tailwind.config` 文件
- 后端：Rust 2021、Tauri commands、`serde`、`serde_json`、`toml`、`dirs`
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
```

生产打包：

```powershell
cd app
npx tauri build
```

首次运行 `npx tauri dev` 会编译大量 Rust 依赖，耗时可能较长。

## 目录说明

```text
app/
  src/                         React 前端
    App.tsx                    根组件，集中调用 Tauri invoke
    components/                Titlebar、Sidebar、SetupGuide、ModList、ModCard
    types/mod.ts               前端共享类型
    index.css                  Tailwind v4 主题与全局样式
  src-tauri/                   Rust/Tauri 后端
    src/lib.rs                 插件注册和 invoke handler 注册
    src/commands/mod_manager.rs 游戏路径、ME3 路径、Mod 扫描/开关/删除/安装/启动
    src/commands/profile.rs    配置方案 CRUD 和激活方案
    capabilities/default.json  Tauri 权限
    tauri.conf.json            窗口与打包配置
me3/                           本地 ME3 工具，已在 .gitignore 中忽略
mods/                          本地示例 Mod 和压缩包，已在 .gitignore 中忽略
reference/                     参考管理器，已在 .gitignore 中忽略
docs/                          项目文档预留
assets/                        项目资源预留
config/                        项目配置预留
```

## 当前架构

前端只通过 `@tauri-apps/api/core` 的 `invoke()` 调用 Rust command；当前没有全局状态库、数据库或事件总线。主状态集中在 `app/src/App.tsx`。

配置存储在系统配置目录：

```text
{dirs::config_dir()}/nightreign-mod-manager/config.json
{dirs::config_dir()}/nightreign-mod-manager/profiles/*.json
{dirs::config_dir()}/nightreign-mod-manager/active_profile.txt
```

Rust command 当前注册在 `app/src-tauri/src/lib.rs`，主要分两组：

- `mod_manager.rs`：`get_game_path`、`set_game_path`、`get_me3_path`、`set_me3_path`、`get_mods_dir`、`scan_mods`、`get_mod_info`、`install_mod_from_zip`、`uninstall_mod`、`toggle_mod`、`generate_me3_profile`、`launch_game`
- `profile.rs`：`get_profiles`、`create_profile`、`delete_profile`、`activate_profile`、`get_active_profile`、`update_profile`

## ME3 集成要点

ME3 工具入口当前按 `{me3_path}/bin/me3-launcher.exe` 查找。不要假设 ME3 被安装在项目目录；项目内 `me3/` 只是开发机上的示例。

`.me3` profile 是 TOML 格式。Nightreign 的 profile 至少包含：

```toml
profileVersion = "v1"

[[supports]]
game = "nightreign"
```

资源包和 DLL 的常见条目：

```toml
[[packages]]
path = "mod"

[[natives]]
path = "mod/SeamlessCoop/nrsc.dll"
load_early = true
```

注意事项：

- 本仓库示例 `More Map Variations.me3` 里 `supports.game` 写成了 `nightrein`，后续生成 profile 时应使用 `nightreign`。
- 参考文件中可能同时出现 `[[package]]` 和 `[[packages]]`，实现前应优先对照官方 ME3 文档和当前 ME3 行为验证。
- 当前 `scan_mods` 扫描的是 `{game_path}/mods/` 下的子目录，并通过目录内 `.me3` 判断类型。
- 当前 `toggle_mod` 使用目录重命名到 `.disabled` 的策略，`scan_mods` 会同时识别普通目录和 `.disabled` 目录。

官方与参考资料：

- ME3 文档：<https://me3.help/en/latest/>
- me3-manager 项目：<https://github.com/2Pz/me3-manager>
- me3-manager 帮助：<https://me3-manager.github.io/me3-manager-help/>
- Nmodm 项目：<https://github.com/QykXczj/Nmodm>
- 本地参考：`reference\Me3_Manager_1.4.5`、`reference\Nmodm_v3.1.4`

## 已实现功能

- 首次设置向导：选择游戏目录和 ME3 目录
- 自定义暗色标题栏：拖动、最小化、最大化、关闭
- 左侧配置方案列表：可折叠、可创建基础方案
- Mod 列表：搜索、按类型过滤、刷新
- Mod 卡片：显示类型、说明、文件数量，支持开关和删除
- Rust 后端：基础配置读写、Mod 扫描、配置方案 JSON 存储、ME3 启动器调用

## 当前已知问题和待办

优先级较高：

- 当前 `npm run build`、`npm run lint`、`cargo check` 均已通过。
- `Profile`/`ProfileMod` 的 Rust 序列化已输出 camelCase，同时保留 snake_case alias 兼容旧 JSON。
- `install_mod_from_zip` 已能把 ZIP 解压到 `{game_path}/mods/`，但根目录识别、重复处理和结构校验仍然需要更多真实 Mod 测试。
- `scan_mods` 已识别 `.disabled` 目录和启用状态，但冲突提示、批量操作和 profile 级启用状态还未完成。
- `launch_game` 已生成 `active-nightreign.me3` 并调用 `me3.exe launch -p`，但还没有和配置方案的 Mod 集合/排序绑定。
- 配置方案尚未真正管理 Mod 集合、加载顺序、启用状态和 profile 文件生成。

后续功能方向：

- 根据当前配置方案动态生成 `.me3` profile。
- 支持 Mod 添加/移出方案、拖拽排序、冲突提示。
- 完善 ZIP 安装：解压、识别根目录、处理重复、支持 `.me3` 和常见 Mod 目录结构。
- 增加路径校验：游戏目录应包含 Nightreign 可执行文件，ME3 目录应包含 `bin/me3-launcher.exe`。
- 增加用户反馈：错误 toast、确认删除、空状态、加载状态。
- 考虑在线浏览/下载 Mod，但这应作为后续阶段，不要影响本地管理核心流程。

## 开发约定

- 不要提交或批量修改 `mods/`、`me3/`、`reference/` 下的内容；这些是本地大文件和参考材料。
- 不要把开发机绝对路径写入源码或默认配置。可以在文档中作为示例说明。
- 修改 Tauri command 时同步检查 `app/src-tauri/src/lib.rs` 的注册和前端 `invoke()` 参数名。
- Tauri `invoke` 参数使用 camelCase，Rust command 参数用 snake_case 时由 Tauri 做映射，例如前端传 `modPath` 对应 Rust 的 `mod_path`。
- 前端 UI 以中文为主；修复或新增中文文本时确认文件编码为 UTF-8。
- 当前 UI 没有引入图标库，已有组件多用内联 SVG。若后续引入图标库，应统一替换，不要混用过多风格。
- Tailwind v4 主题变量集中在 `app/src/index.css`，新增颜色优先在那里定义。
- 对文件系统操作要保守，删除 Mod 前应有确认；不要默认扫描或删除用户游戏目录里的非 Mod 文件。
- 新增涉及 profile、Mod 安装、启动游戏的功能时，优先加 Rust 单元/集成测试或至少做 `cargo check` + `npm run build`。

## 参考 Claude 记忆

历史 Claude Code 会话记忆位于：

```text
C:\Users\34590\.claude\projects\d--Project-Game-create\memory\session_2026-05-22_mod_manager.md
C:\Users\34590\.claude\projects\d--Project-Game-create\memory\MEMORY.md
```

这些文件在当前 PowerShell 输出中可能出现中文乱码；以源码实际状态为准。
