# 当前状态与新会话交接

更新时间：2026-07-31
当前分支：`master`
本轮改动起点：`6e09440 docs: add release documentation`

本文件记录当前实现的短期交接状态。长期规则和已验证启动链路以仓库根目录 `AGENTS.md` 为准；视频研究与更新脚本以 `docs/VIDEO_UPDATE_PLAN.md` 为准。

## 新会话先做什么

1. 阅读 `AGENTS.md`、本文件和 `git status --short`。
2. 如果新会话目标是 Spacewar + Seamless 模组整合，再阅读
   `docs/SPACEWAR_SEAMLESS_MODPACK_HANDOFF.md`，它是该专项的第一入口。
3. 保留当前未提交改动，不要执行 `git reset --hard`，不要恢复与当前任务无关的文件。
4. 专项研究阶段先做只读来源核对和候选包拆解，不要放宽 Server Redirector +
   Spacewar 硬门禁，也不要覆盖实际游戏目录。
5. 如果继续 UI 工作，先在默认 1160×760 和最小 960×640 窗口检查现状，再修改布局。

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
- 已完成“模组整合包 × 管理器 × 视频”新一轮探索，结论记录在 `docs/MODPACK_CONTENT_STRATEGY.md`。
- 整合包第一阶段按“可验证清单/配方”设计：记录来源、版本、依赖、哈希和加载顺序，默认不重打包第三方 Mod；只有自制、许可证允许或取得明确授权的文件才可随包分发。
- 首个建议实验是轻量稳联验证方案，用一个可见 package、一个有顺序要求的 native 和 SeamlessCoop 环境串联健康检查、联机一致性与视频验证，不先做大而全在线 Mod 商店。
- 已审计 More Map Variations、MoreWeapons、两份简中翻译和 Skin Overhaul，详细结论见 `docs/POPULAR_MODPACK_RESEARCH.md`。官方支持路线应使用作者提供的 MMV + Weapons 2.1.7.1 预合并版与 Server Redirector，并把 Skin Overhaul 作为单独的 SeamlessCoop 方案；社区兼容分支另按整合包自身规则验证。
- MMV 作者 Profile 兼容层已按网络 native 自动选择运行环境：Server Redirector 方案不会再注入游戏根目录的 `SeamlessCoop\nrsc.dll/nighter.dll`，外部作者 `.me3` 保持只读；仍不要把 MMV ZIP 解压进实际 `Game` 目录。
- 已只读审计用户保留的 `5.30 NightreignPLUS` 旧整合包，结论见 `docs/NIGHTREIGNPLUS_AUDIT.md`。该包证明 MMV/MoreWeapons/Skin/Seamless 的社区兼容分支技术上可行：先使用 More Map and Weapons，再用 ERBM 参数级合并 Skin regulation，并人工裁剪材质冲突；它不是普通加载顺序。
- 旧包约 7.12 GiB，但实际 profile 只引用两个 native 和两个最终 package，其余多为上游源件、工具和中间产物。不要把包内每个目录都当作独立 Mod。
- 当前 `.me3` 解析只读取 package 的 `path`，而 NightreignPLUS 使用 `source`；重新生成 profile 还会丢失 native `id/load_after`，并可能重复注入 `nrsc.dll`。因此当前管理器不能原样导入此类整合包，P1 外部 profile 模式必须保留作者配置和依赖关系。
- 已把非官方大整合明确降为备用兼容性实验，并完成当前官方 App 1.03.2 / Regulation 1.03.5 下的分级建议，见 `docs/CURRENT_VERSION_MODPACK_PLAN.md`。
- 当前主推 S0/S1 是 Duchess Unmasked 纯 `parts` 外观包与可选 Seamless v1.1.3；它们最适合现有管理器验证安装、package 推断、native early load、联机清单和回退。本机 Duchess 样本无 regulation/DLL，但仍需 1.03.5 启动与 `_l` 队友视角回归。
- 4–6 人方案只使用 Seamless + Nightreign 6 Player Fixes，后者虽标注适配 1.03.5，但作者明确称与多数其他 Mod 不兼容，不能与玩法大修混合。
- 高级内容主线为 MMV + Weapons 官方预合并 2.1.7.1、602 简中和 Server Redirector；602 当前 314 KB 主文件上传于 2026-07-22，Changelog 明确标注同步 2.1.7.1 的新增武器被动词条，虽然页面顶部仍显示 2.1.6。作者 Profile 的管理器生成与启动前检查已完成真实样本验证，待游戏内地图/敌人/武器、中文、存档行为和 Redirector 联机验收后进入正式推荐。已被 Nexus staff 移除的 MMV Seamless 补丁不得推荐。
- nighter 当前公开 Nexus 文件陈旧，更新版缺少稳定可信的发布链且加载依赖说明不一致；只保留自定义 DLL/实验项，不进入默认整合包。
- 已审计用户新下载的 MMV 2.1.7 Hotfix 1 + Weapons 与 `SkinOverhaul` 两个目录，详见 `docs/DOWNLOADED_PACKS_MANAGER_AUDIT.md`。审计时发现的 `savefile/start_online` 丢失和 Seamless/Server Redirector 冲突已由后述 MMV 作者 Profile 兼容层解决；Skin 结论不变。
- 新下载的 `SkinOverhaul\regulation.bin` 哈希为 `b06275a6...d6b51fd`，与 NightreignPLUS 5.30 中 MMV + Skin 的旧 ERBM 合并输出一致，不是可直接作为独立 Skin 方案判断的干净 regulation；同时尚未生成在线 `_l` 文件。不要启用或与当前 MMV 2.1.7.1 叠加。
- 2026-07-30 已实现 MMV 作者 Profile 兼容层，详见 `docs/MMV_MANAGER_COMPATIBILITY.md`：生成副本保留 `savefile/start_online` 和未知 TOML 字段，规范 `nightrein` 为 `nightreign`，检测 Server Redirector 后禁止自动注入根目录 `nrsc.dll/nighter.dll`，并在混用时阻止启动。真实下载目录与当前工作区的只读集成测试均已通过，生成的 active profile 仅包含 MMV package + `cl_server_redirector.dll`，保留 `NR0000.co2`。
- 2026-07-31 真实启动回归确认 MMV package、Server Redirector 和作者指定的 `NR0000.co2` 均已被 ME3 正确加载，但当前 `Game` 目录是 OnlineFix/Spacewar 混合环境，游戏报“无法登录游戏服务器”，并出现人物不显示。日志无 ME3 或 Redirector 注入错误，根因边界是管理器错误沿用了 `--skip-steam-init`，且该游戏目录不满足作者要求的正版 Steam 环境。`NR0000.co2` 与 Seamless 默认存档同名，不是独立存档。
- P0 环境隔离现已完成：配置和启动台区分纯正版 Steam、正版 Steam + Seamless、Spacewar + Seamless、自动/未知混合环境；只有 Spacewar 路线标记为用户已实测。纯正版与正版 Seamless 不使用 `--skip-steam-init/--online`，Spacewar 保留两项已验证参数，MMV Server Redirector 仅使用 `--online`。
- 正版两种模式和 MMV 必须匹配 Steam `appmanifest_2622380.acf`；OnlineFix/模拟层文件、后端错配、未知自动检测结果会在实际启动前硬阻止。`nighter.dll` 已从 Seamless 后端判定中剥离，单独存在不再触发 Seamless 参数。
- 启动/诊断启动前会按有效存档名备份所有账号目录中的存档及 `.bak`；普通正版 Mod 默认使用 `NR0000.nmm`，Seamless 从 `nrsc_settings.ini` 推断扩展名，MMV 的 `NR0000.co2` 会显示同名风险警告。
- OnlineFix 补丁安装现为事务操作：仅 Spacewar + Seamless 可用，Steam 正版目录硬阻止；覆盖 8 个固定目标前生成清单和原文件备份，失败自动回滚，启动台可恢复最近备份。
- ME3 启动前检查会读取版本；低于建议的 0.12.1 只警告、不阻断，以保留当前 0.11.0 Spacewar 已验证链路。
- MMV 尚未完成最终游戏内验收。下一次必须由具备正版环境的测试者把管理器指向干净的 Steam 正版 `Game` 目录，并确认 Forsaken Hollows DLC，再验证人物、地图、敌人、武器、存档行为和联机。当前用户只能提供 Spacewar + Seamless 环境，因此不得将 MMV 标记为完整实测通过。
- MMV、MoreWeapons 和 Skin Overhaul 的 Nexus 权限均禁止上传到其他站点，因此不能作为管理器/视频附件重新分发；可分享清单、来源链接、哈希、管理器生成配置，以及按要求署名的 602 翻译。
- 已建立 `docs/SPACEWAR_SEAMLESS_MODPACK_HANDOFF.md` 作为下一会话专项交接。
  最新启动前检查确认：当前 Game 在停用 MMV 后可正确识别为已实测的
  Spacewar + Seamless 且联机组件就绪；启用 MMV 作者包后，因为其绑定
  Server Redirector 而被正确阻止。下一阶段目标是寻找或构建真正使用 Seamless
  的“地图/敌人 + 武器 + 单一中文层”社区兼容分支，而不是删除门禁强行启动。
- 2026-07-31 已结合 Bilibili `BV1zmV56CEJq` 的简介、公开评论/UP 回复，
  Nexus 578/602、Reddit Spacewar 实例和 GitHub 公开 Profile 找到可复现的
  社区机制：继续加载 MMV + Weapons 的单一预合并 package，但在生成副本中
  移除 Server Redirector，改由现有 `nrsc.dll` early load；ME3 自定义启动目标
  仍是 `nightreign.exe`。这与本项目已实测的 Spacewar 启动参数链一致，但不受
  MMV 作者支持，也不能重新分发作者资产。
- 管理器已新增外部作者 Profile 的显式“社区 Seamless 兼容”开关。切换操作只写
  管理器注册配置；生成时验证明确的 Seamless 运行环境、唯一
  `regulation.bin`、现有 `nrsc.dll`，并保留原作者文件只读。启动前检查新增
  社区风险提示、regulation SHA-256、单一完整 `msg\zhocn` 层及其两个关键文件
  SHA-256；多份中文覆盖会硬阻止。
- 本地 `More Map Variations 2.1.7-hotfix1 & Weapons Mod` 已通过新增的只读
  集成测试：1440 个文件、唯一 `regulation.bin`、作者 Redirector Profile 均能
  解析，生成的社区副本只使用 Seamless 后端，原 `.me3` 与 DLL 未变化。现有两份
  5 月旧汉化目录结构/版本均不满足当前候选要求，继续保持停用；在下载 602 于
  2026-07-22 上传的 314 KB 主文件、记录哈希并完成真实单人/双人启动前，不能
  标记整套方案已实测成功。
- 继续审计 602 安装入口时发现并修复了通用 ZIP 单根剥离错误：旧实现会把唯一的
  `msg`、`parts` 目录甚至单个 `regulation.bin` 当成包装层删除。新实现只剥离
  所有文件共同拥有且不是游戏语义根的真实包装目录；汉化的 `zhocn\item/menu`
  或根级两个文本文件会规范到 `msg\zhocn\`，遇到多套/不完整布局则回滚整个安装。
  完整中文层在 Mod 卡片上会显示专用说明。
- 已在实际 Tauri 调试窗口进行只读 UI 回归：启动台正确显示
  `Spacewar + Seamless · 已实测`，MMV 外部卡片显示作者 Profile、
  Server Redirector 和“改用社区 Seamless 兼容”入口；确认弹层完整说明只改
  生成副本、原文件不变及社区非官方风险。验证后点击取消，没有改写外部 Mod 配置、
  启用 Mod 或启动游戏。
- 602 下载阻塞已于 2026-07-31 解除。用户在 Codex 内置浏览器完成 Nexus 登录后，
  自动化通过精确 `file_id=4318` 进入官方确认页，唯一点击 `Slow download`，
  下载事件在 49.194 秒内触发，未刷新或绕过站点验证。实际文件为
  `zhocn 602 2.1.7.1 2026-07-22T10-56Z ij0FavW9k.zip`，大小 321,793 字节，
  SHA-256 为
  `3166254E167551F8F8B85B2897070371D6D58C57407687F4607CC91493946B58`。
  ZIP 只含 `zhocn/item_dlc01.msgbnd.dcx` 与
  `zhocn/menu_dlc01.msgbnd.dcx`，与管理器新增的 `msg\zhocn` 规范化路径一致。
  已将原 ZIP 与 evidence JSON 复制到
  `D:\Project\Game-create\_downloads\elden-ring-nightreign\`，系统 Downloads
  原件保留；以后跨项目真实样本使用 `_downloads\<game-or-project>` 暂存区，
  不直接散落在 `D:\Project\Game-create` 根目录。
  本轮只把文件带到安装选择器并填写路径，未点击“打开”，因此没有安装、启用或
  改写游戏目录；后续游戏内验收仍未完成。
- 已检查官方客户端替代路径：环境变量中没有 Nexus API Key，也没有 Nexus CLI、
  Vortex 或 Nexus Mods App；系统的 `nxm://` 协议虽然注册给
  `D:\MO\ModOrganizer\nxmhandler.exe`，但现有 MO2 只有 Sekiro 和 Skyrim
  实例，没有 Nightreign 实例。不要为了这次下载自动创建或改写 MO2 实例；用户在
  网页完成登录并下载 ZIP 是影响最小、可审计的恢复方式。
- 官方文件页的当前 7 月主文件“版本历史”链接暴露标识 `4318`，可用
  `https://www.nexusmods.com/eldenringnightreign/mods/602?tab=files&file_id=4318`
  在登录后直接定位，避免误选 6 月的 313 KB 旧文件。`file_id` 不是下载授权；
  免费下载链接仍需 Nexus 登录后生成的时效参数。
- 已结合 `Cyberpunk2077-ModManager` 的真实样本下载复盘，确认可复用的开发期
  流程是“已登录内置浏览器 + 精确 file_id + 唯一 Slow download + 80 秒以上
  下载事件 + Downloads 落盘检查 + SHA-256 证据”，而不是把 Nexus API 接入
  产品。本次已用 602 官方真实文件验证该方法。复盘也暴露出跨项目迁移问题：
  Nightreign 专项没有在开始时先查 ReviewHub，已有方法直到用户提醒后才被使用。
  现已将本项目注册到 `D:\Project\CodexReviewHub`，为 Cyberpunk 原卡写入一次
  `validated` 复用事件，并新增
  `docs/reviews/2026-07-31-cross-project-mod-manager-knowledge-transf-7c6a2e65.review.json`。
  后续非平凡任务应先检索 ReviewHub，再把命中的知识分成可复用规则、领域差异和
  本项目新增项。

## 2026-07-31 验证结果

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

Rust 单元测试结果：35 项（32 passed，3 ignored，0 failed）。三项 ignored
分别用于真实下载 MMV 作者 Profile、真实 MMV 社区转换和当前工作区只读集成
验证，需显式提供本机环境变量。本轮已显式运行前两项本地样本测试，均通过；
此前当前工作区检查同时确认自动识别为
Spacewar + Seamless，并因 MMV Server Redirector 环境错配而阻止启动。
`npx tauri build --debug --no-bundle` 已通过独立
`.build-tmp/mmv-seamless-tauri-target` 完成桌面应用构建，产物为
`debug/nightreign-mod-manager.exe`。本轮未结束用户进程，也未改写游戏目录、
外部 Mod 注册状态或启用状态。

尚未在本次交接前重新验证：

- `npx tauri build` 生产打包；
- 新安装包的安装/卸载；
- 最新 UI 和进程检测修改后的完整真实游戏启动；
- 最新 P0 代码下 Spacewar + Seamless 的完整真实游戏启动与双人联机；
- 纯正版 Steam、正版 Steam + Seamless 和 MMV Server Redirector 真实启动（当前用户无此环境，只完成静态规则与自动化测试）；
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
- 整合包与内容协同探索：`docs/MODPACK_CONTENT_STRATEGY.md`
- 热门模组整合兼容性审计：`docs/POPULAR_MODPACK_RESEARCH.md`
- NightreignPLUS 5.30 整合机制审计：`docs/NIGHTREIGNPLUS_AUDIT.md`
- 当前官方版本管理器整合方案：`docs/CURRENT_VERSION_MODPACK_PLAN.md`
- 已下载整合包的管理器适配审计：`docs/DOWNLOADED_PACKS_MANAGER_AUDIT.md`
- Spacewar + Seamless 玩法整合专项交接：`docs/SPACEWAR_SEAMLESS_MODPACK_HANDOFF.md`
- 前端状态与 Tauri 调用：`app/src/hooks/useModManager.ts`
- 启动台与共享页面骨架：`app/src/pages/LaunchPage.tsx`
- 主题与共享面板：`app/src/index.css`
- Rust Mod/启动逻辑：`app/src-tauri/src/commands/mod_manager.rs`
- 配置方案逻辑：`app/src-tauri/src/commands/profile.rs`
- Tauri command 注册：`app/src-tauri/src/lib.rs`
