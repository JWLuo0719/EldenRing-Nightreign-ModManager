# 当前状态与新会话交接

更新时间：2026-07-16
当前分支：`master`
本轮改动起点：`6e09440 docs: add release documentation`

本文件记录当前实现的短期交接状态。长期规则和已验证启动链路以仓库根目录 `AGENTS.md` 为准；视频研究与更新脚本以 `docs/VIDEO_UPDATE_PLAN.md` 为准。

## 新会话先做什么

1. 阅读 `AGENTS.md`、本文件和 `git status --short`。
2. 保留当前未提交改动，不要执行 `git reset --hard`，不要恢复与当前任务无关的文件。
3. 先完成最新工作树的真实启动回归，再继续开发 P1 功能。
4. 如果继续 UI 工作，先在默认 1160×760 和最小 960×640 窗口检查现状，再修改布局。

## 提交后的预期工作树

本轮前端 UI、Rust/Tauri 后端、权限、依赖、开发脚本、README、CHANGELOG 和项目记忆改动随本文件一并提交。新增并纳入版本控制的内容包括：

- `app/src-tauri/.cargo/config.toml`
- `app/src-tauri/Cargo.lock`
- `app/src-tauri/icons/`
- `docs/VIDEO_UPDATE_PLAN.md`
- 本文件

`release/v0.1.0/nightreign-mod-manager_0.1.0_x64-setup.exe` 的本地删除不属于本轮提交。提交后它仍会显示为已删除；发布前必须由用户决定是恢复旧安装包、保留删除，还是用新构建产物替换。不要擅自恢复或提交。

## 本轮已完成

### 启动与诊断

- 保留已经由用户确认成功的 Nightreign + ME3 + SeamlessCoop/Spacewar 启动链路。
- 增加不会启动游戏的启动前检查，检查游戏/ME3 路径、真实启动目标、Steam、残留进程、联机组件、深夜解锁和启用 Mod。
- 残留进程检测改为精确匹配 `nightreign.exe`、`me3-launcher.exe` 等受保护进程名，并显示 PID，避免宽泛匹配造成误报。
- 启动日志超过 2 MB 时轮转，诊断页只读取末尾 512 KB。

### 安全与正确性

- 本地 Mod 的切换与删除限制在 `Game\mods` 直属项；外部 Mod 只修改注册记录。
- 配置方案 ID 增加路径穿越防护。
- Tauri 权限收敛到实际使用的 dialog 能力，并启用生产 CSP。
- profile 生成继续保留 package/native 加载顺序和 Windows 路径规范化行为。

### 性能与内存

- Mod 扫描、ZIP 解压、冲突分析转移到 blocking worker。
- 冲突分析增加扫描/结果上限，并使用紧凑所有者索引。
- Rust dev/test 调试信息和 Cargo 并发已收敛；开发脚本把编译临时文件放到仓库根目录 `.build-tmp/`，避免 Tauri watcher 因临时文件反复重启。
- WebView2 当前通过 `--disable-gpu` 降低静态界面的空闲私有内存。它属于软件运行时配置，并非只影响当前工作电脑；如果以后加入 WebGL 或重动画，需要重新测量内存、CPU 和流畅度。
- 此前开发阶段看到的高内存主要还包含 Cargo、链接器、Node/Vite 等开发进程，不能直接等同于发布版管理器的运行内存。

### UI

- 界面已收敛为低噪声黑金“作战控制台”方向。
- 侧栏约 216px，活动页使用金色状态轨道；页面标题和卡片层级已减少。
- `LaunchPage.tsx` 导出的 `PageFrame` 作为页面骨架，`.panel-card` 作为共享面板样式。
- 启动台重新组织为方案、启动控制、检查报告、运行环境和加载清单。
- Mod、配置方案、诊断、设置页已针对 1160×760 默认窗口改用 `lg` 双栏。

### 产品与视频方向

- 已复盘两条 Bilibili 首发视频的公开数据和评论，结论记录在 `docs/VIDEO_UPDATE_PLAN.md`。
- 下一阶段优先级是 Mod 健康检查、联机一致性清单和配置方案交互，不优先做在线下载。
- 更新视频发布前必须重新刷新数据；联机清单未实现前不能在视频中宣称支持自动比对。

## 2026-07-16 验证结果

以下命令已在当前工作树重新执行并通过：

```powershell
cd app
npm run lint
npm run build

cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check
cargo test
```

Rust 测试结果：11 passed，0 failed。

尚未在本次交接前重新验证：

- `npx tauri build` 生产打包；
- 新安装包的安装/卸载；
- 最新 UI 和进程检测修改后的完整真实游戏启动；
- 最新代码下 SeamlessCoop 双人联机；
- 大型真实 Mod ZIP 的结构组合与冲突扫描性能。

历史上用户已确认游戏可以通过管理器启动且 Mod 生效，但这不能替代最新工作树的回归。

## 建议实施顺序

### P0：最新工作树回归

1. 完全退出并重新打开 Codex，使更新后的全局 Skill 生效。
2. 从仓库根目录运行 `.\dev.bat`，确认 Tauri 不再反复启动退出。
3. 先点“启动前检查（不会启动游戏）”，确认无残留进程误报。
4. 只点击一次普通启动，确认游戏、ME3 控制台、SeamlessCoop 和至少一个资源包 Mod 正常。
5. 退出游戏后再次检查进程保护和日志。
6. 检查 1160×760、960×640 两个窗口尺寸的主要页面。

### P1：Mod 健康检查

- 检测压缩包多套一层、无法识别的入口、`.me3` 引用缺失、DLL/资源包类型和已知依赖。
- UI 区分“目录已启用”和“预计可以加载”，并提供可执行的修复提示。
- 继续用真实 Mod 覆盖单根目录、多根目录、`.me3`、DLL-only、资源包-only 和混合型 ZIP。

### P1：联机一致性清单

- 导出脱敏清单：管理器/游戏版本、Mod 名称、类型、加载顺序、关键文件大小与哈希。
- 导入好友清单后显示相同、缺少、版本不同和顺序不同。
- 不导出绝对个人路径、用户名、账号或存档信息。

### P1：配置方案交互

- Mod 卡片直接支持加入/移出当前方案。
- 增加批量启用、禁用和同步目录状态。
- 保留拖拽顺序，并让联机清单明确体现加载顺序。

### P2：大型目录与发布

- 为扫描和冲突分析增加进度、取消和基于目录修改时间的缓存。
- 完成生产构建、安装包回归和版本号决策后，再录制更新视频。
- 处理当前 release 安装包删除状态，并整理可提交的发布产物。

## 开发环境与 Skill 状态

- 系统全局 `frontend-design` 已更新为 Anthropic 当前版本；它不会限制模型能力，只作为设计质量检查和方向参考。
- 已更新或统一的常用全局技能包括 `find-skills`、`tauri-v2`、`playwright`、`react-vite-best-practices`、`changelog-automation`、`e2e-testing-patterns` 和 `jianying-editor`。
- Lark CLI 当前为 1.0.70，Lark 技能已同步。
- 项目 `.agents/skills/` 只保留 `desktop-app`，过期和完全重复的项目副本已清理。
- GitHub 在技能更新后段出现连接中断；`humanizer-zh`、`douyin-video-summary`、`powershell-windows`、`rust-pro`、`md-to-zhihu` 保留原可用版本，后续网络恢复后可再检查。
- 当前启用的 Codex 插件共 12 个，未发现需要卸载的重复启用项；OpenAI 内置插件市场和 ChatCut 已核对为当前缓存/提交。

## 关键文件

- 长期项目记忆：`AGENTS.md`
- 当前交接：`docs/CURRENT_STATUS.md`
- 视频研究与更新方案：`docs/VIDEO_UPDATE_PLAN.md`
- 前端状态与 Tauri 调用：`app/src/hooks/useModManager.ts`
- 启动台与共享页面骨架：`app/src/pages/LaunchPage.tsx`
- 主题与共享面板：`app/src/index.css`
- Rust Mod/启动逻辑：`app/src-tauri/src/commands/mod_manager.rs`
- 配置方案逻辑：`app/src-tauri/src/commands/profile.rs`
- Tauri command 注册：`app/src-tauri/src/lib.rs`
