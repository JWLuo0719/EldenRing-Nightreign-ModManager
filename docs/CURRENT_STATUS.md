# 当前状态与新会话交接

更新时间：2026-08-02
当前分支：`master`
当前发布基线：`v0.2.0`（第二版，2026-08-02）

本文件记录当前实现的短期交接状态。长期规则和已验证启动链路以仓库根目录 `AGENTS.md` 为准；视频研究与更新脚本以 `docs/VIDEO_UPDATE_PLAN.md` 为准。

## 新会话先做什么

1. 阅读 `AGENTS.md`、本文件和 `git status --short`。
2. 如果新会话目标是 Spacewar + Seamless 模组整合，再阅读
   `docs/SPACEWAR_SEAMLESS_MODPACK_HANDOFF.md`，它是该专项的第一入口。
3. 保留当前未提交改动，不要执行 `git reset --hard`，不要恢复与当前任务无关的文件。
4. 专项研究阶段先做只读来源核对和候选包拆解，不要放宽 Server Redirector +
   Spacewar 硬门禁，也不要覆盖实际游戏目录。
5. 如果继续 UI 工作，先在默认 1160×760 和最小 960×640 窗口检查现状，再修改布局。

## 工作树背景与约束

本轮前端 UI、Rust/Tauri 后端、权限、依赖、开发脚本、README、CHANGELOG 和项目记忆改动随本文件一并提交。新增并纳入版本控制的内容包括：

- `app/src-tauri/.cargo/config.toml`
- `app/src-tauri/Cargo.lock`
- `app/src-tauri/icons/`
- `docs/VIDEO_UPDATE_PLAN.md`
- 本文件

`release/v0.1.0/nightreign-mod-manager_0.1.0_x64-setup.exe` 的本地删除不属于本轮提交。提交后它仍会显示为已删除；发布前必须由用户决定是恢复旧安装包、保留删除，还是用新构建产物替换。不要擅自恢复或提交。

2026-08-01 阶段验收交接时，`AGENTS.md`、本文件、
`docs/SPACEWAR_SEAMLESS_MODPACK_HANDOFF.md` 和 savefile 复盘 JSON 有未提交文档更新；
`app/` 没有未提交源码改动。新会话应保留这些文档改动，并继续把安装程序删除视为
用户的无关变更，除非用户明确决定如何处理。

## 本轮已完成

### 2026-08-02 v0.2.0 第二版发布

- 前端、Tauri 和 Rust 版本统一升级为 `0.2.0`，正式发布目录为 `release/v0.2.0/`。
- 生成并验证 NSIS `nightreign-mod-manager_0.2.0_x64-setup.exe` 与简体中文 MSI
  `nightreign-mod-manager_0.2.0_x64_zh-CN.msi`；发布目录包含说明和 SHA-256 清单。
- `【双击】--更多游戏内容.url` 随两个安装包写入程序目录，并作为独立发布附件提供，
  内容为 `https://link3.cc/voyagekit`。NSIS 隔离安装与 MSI 管理提取均已回读文件名、内容和
  产品版本；NSIS 测试安装随后通过自带卸载程序清理。
- 发布前验证通过：`npm run lint`、`npm run build`、`cargo fmt --all -- --check`、
  `cargo clippy --all-targets --all-features -- -D warnings`、`cargo check`、`cargo test`；
  Rust 测试结果为 44 passed、8 ignored、0 failed。
- 第二版的玩家侧重点是无终端启动、可执行的启动前检查、玩家化中文术语、运行环境隔离、
  外部启动配置语义保真、玩法数据冲突门禁、存档备份、服装与 `_l` 结构检查，以及完整脱敏
  联机一致性清单。“应用联机补丁”入口继续隐藏。
- 两组视频可见成果必须分开表述：MMV + MoreWeapons + 602 已在 Spacewar + Seamless 社区
  路线中验证地图/敌人、武器、中文、登录和好友邀请；Skin Overhaul + 派生中文层已在
  Skin-only + Seamless 中验证模型与中文名称。两组玩法数据不能直接同时启用，第二位玩家
  实际加入同房也尚未验收。
- 发布提交为 `ea5a9e2`，标签为 `v0.2.0`，GitHub Release：
  `https://github.com/JWLuo0719/EldenRing-Nightreign-ModManager/releases/tag/v0.2.0`。
- 视频交接已写入
  `D:\Project\Video\CrossProjectVideoStudio\campaigns\nightreign-mod-manager-v0-2-release\`；
  活动状态为 `script-ready`，包含三条横版稿、三条竖版短稿、声明台账、镜头表与录制交接。
- 本轮发布与视频交接的结构化复盘保存在
  `docs/reviews/2026-08-02-nightreign-mod-manager-v0-2-0-2e6fd594.review.json`。

### 2026-08-02 更多服装未加载定位与阶段修复

- 用户 00:12 的实际启动并不是服装资源加载后显示失败：生成的
  `active-nightreign.me3`、`last-gameplay-profile.json` 和本轮 ME3 日志只包含
  MMV package 与 early-load `nrsc.dll`，没有 Skin Overhaul package。
- 根因是外部注册仍指向已经不存在的
  `D:\Game\ELDEN RING NIGHTREIGN\mods\Skin Overhaul`，实际目录为
  `...\mods\SkinOverhaul`。旧实现允许失效记录保持启用，却会在生成启动配置时静默
  产生零 package，造成“界面似乎启用、游戏里完全没加载”的假象。
- 管理器现在会显示“原文件夹已失效”，禁用其启用开关，并提供“重新定位文件夹”；
  重新定位会保留外部注册状态，并把所有配置方案中的路径派生 Mod ID 一并迁移。
  后端也拒绝启用不存在的外部目录，生成启动配置遇到已选中的失效路径会直接报出
  玩家可执行的修复说明，不再静默跳过。
- 路径修正后仍不能把当前 Skin Overhaul 直接叠加到 MMV 2.1.7.1：前者
  `regulation.bin` 为 2,785,376 字节、SHA-256
  `B06275A641CAA50E08E235E7F5D2C8BA545FDE7BE5D70AFDB693D2920D6B51FD`；当前 MMV
  文件为 3,191,600 字节、SHA-256
  `D36B9960E19C748112F2A8D0D4C00D33A2BEC8AE9BB1707975516C3DBB64F579`。
  两者不是同一玩法数据。启动前检查现对所有方案显示来源并硬阻止多份
  `regulation.bin`；普通启动和诊断启动的 Rust 后端也执行同一门禁，不能通过跳过
  UI 检查绕过。
- 真实目录只读测试通过：Skin Overhaul 仍识别为 228 个本机 parts + 228 个 `_l`
  完整配对、含扩展服装参数和 `01_Online.bat`；管理器没有运行脚本。女爵去面罩
  样本仍识别为 5 个纯替换 parts、无 regulation。由于当前 MMV/Skin 尚无适配
  2.1.7.1 的单一合并 regulation，本阶段没有再次启动游戏宣称服装已生效。

用户随后完成 Skin-only + Seamless 真实启动：新增服装列表和人物模型均正常显示，
证明 Skin package、regulation 与本机服装 parts 已实际加载；本轮尚未由第二位玩家
观察 `_l` 队友视角，不能把本机模型成功扩大表述为联机外观已验收。中文界面下新增
条目全部显示 `?GoodsName?`，原因也已由文件结构确认：Skin 只提供
`msg\engus\item_dlc01.msgbnd.dcx`，没有 `msg\zhocn`。

2026-08-02 又审计了公开候选 `ERN VINS CN 1.5`：Nexus 下载 ZIP 为 318,908 字节，
SHA-256 `103CF4A60E44F9754529ABE494F77C2043FA3301C54DC85F94064994ACD1A836`；包内同时
含完整 `zhocn/item_dlc01.msgbnd.dcx` 与 `menu_dlc01.msgbnd.dcx`，不是可直接叠加的
“服装名称小补丁”。WitchyBND 条目级比对显示，Skin 当前英文包相对 602 新增 76 个
`GoodsName`：该候选只有 50 个对应 ID，其中仅 8 个已写成中文、42 个仍是英文、
26 个完全缺失。因此不得安装它替换 602，也不能把它当作可用的 Skin 简中补丁。

已按正确路线生成本地派生包，并默认停用安装到
`D:\Game\ELDEN RING NIGHTREIGN\Game\mods\SkinOverhaul-602-服装中文兼容补丁.disabled`：
以 602 的 `item_dlc01` 为底，仅合入 76 个 Skin Overhaul `GoodsName`；产物为
177,920 字节、SHA-256 `E19B438301CCECBB91C6EB2A02F66FDE6518285A08B1DB03F17A8D850127112A`，
重新解包回读 76 / 76 个 ID 均与映射一致。用户已在 Skin-only + Seamless 实际游戏中确认
抽查的新增服装名称（含 2B、双月骑士蕾菈娜、米勒的鲁卡提耶、防火女、梅琳娜）正常显示，
不再是 `?GoodsName?`。2026-08-02 随后在 `CL_MenuText` 401001、401002 的原有破晓团队
页脚下追加 `B站 学游分享：基于破晓团队汉化补丁制作的 Skin Overhaul 服装中文兼容补丁`；
新 `menu_dlc01` 为 147,984 字节、SHA-256
`D19A2CEE02E0F1E11FED271FC21D093FE4949FE4470AC5DE3C2A5065DE5DF4C4`，独立解包回读两个
条目均成功。映射、页脚配置和可复现构建脚本分别为
`tools/skin-overhaul-602-zhocn-names.json`、`tools/skin-overhaul-602-zhocn-footer.json` 与
`tools/build-skin-overhaul-602-zhocn-item.ps1`；原始 602 和 Skin 文件没有改动。启用测试时
只能启用此派生中文层，原始 602/559 和其它完整中文层保持停用。页脚尚待下一次实际进游戏
截图确认，且本地名称显示不能替代联机中文层验收。

MMV + Skin 仍需要通过可审计的参数级合并生成适配当前 MMV 2.1.7.1 的唯一
`regulation.bin`，不得用加载顺序或简单覆盖冒充合并。

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
- 下一阶段优先级是 Mod 健康检查、联机清单的进度/缓存和配置方案交互，不优先做在线下载。
- 更新视频发布前必须重新刷新数据；联机清单现已支持精确导出/比较，但在第二台真实客户端完成互换验收前，只能宣称“功能与本机真实样本通过”，不能宣称双人联机已实测。
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
- 新下载的 `SkinOverhaul\regulation.bin` 哈希为 `b06275a6...d6b51fd`，与 NightreignPLUS 5.30 中 MMV + Skin 的旧 ERBM 合并输出一致，不是可直接作为独立 Skin 方案判断的干净 regulation。早期审计时尚未生成 `_l`；2026-08-01 当前目录的只读复核已确认 228 个本机 parts 与 228 个同名 `_l` 全部配对，但这不改变它与当前 MMV 2.1.7.1 的 regulation 冲突边界，不要直接叠加启用。
- 2026-07-30 已实现 MMV 作者 Profile 兼容层，详见 `docs/MMV_MANAGER_COMPATIBILITY.md`：生成副本保留 `savefile/start_online` 和未知 TOML 字段，规范 `nightrein` 为 `nightreign`，检测 Server Redirector 后禁止自动注入根目录 `nrsc.dll/nighter.dll`，并在混用时阻止启动。真实下载目录与当前工作区的只读集成测试均已通过，生成的 active profile 仅包含 MMV package + `cl_server_redirector.dll`，保留 `NR0000.co2`。
- 2026-07-31 真实启动回归确认 MMV package、Server Redirector 和作者指定的 `NR0000.co2` 均已被 ME3 正确加载，但当前 `Game` 目录是 OnlineFix/Spacewar 混合环境，游戏报“无法登录游戏服务器”。日志无 ME3 或 Redirector 注入错误，根因边界是管理器错误沿用了 `--skip-steam-init`，且该游戏目录不满足作者要求的正版 Steam 环境。当时同时观察到的女爵空白后来证实是无效自定义服装 ID，与 Redirector 无关。`NR0000.co2` 与 Seamless 默认存档同名，不是独立存档。
- P0 环境隔离现已完成：配置和启动台区分纯正版 Steam、正版 Steam + Seamless、Spacewar + Seamless、自动/未知混合环境；只有 Spacewar 路线标记为用户已实测。纯正版与正版 Seamless 不使用 `--skip-steam-init/--online`，Spacewar 保留两项已验证参数，MMV Server Redirector 仅使用 `--online`。
- 正版两种模式和 MMV 必须匹配 Steam `appmanifest_2622380.acf`；OnlineFix/模拟层文件、后端错配、未知自动检测结果会在实际启动前硬阻止。`nighter.dll` 已从 Seamless 后端判定中剥离，单独存在不再触发 Seamless 参数。
- 启动/诊断启动前会按有效存档名备份所有账号目录中的存档及 `.bak`；普通正版 Mod 默认使用 `NR0000.nmm`，Seamless 从 `nrsc_settings.ini` 推断扩展名，MMV 的 `NR0000.co2` 会显示同名风险警告。
- OnlineFix 补丁安装现为事务操作：仅 Spacewar + Seamless 可用，Steam 正版目录硬阻止；覆盖 8 个固定目标前生成清单和原文件备份，失败自动回滚，启动台可恢复最近备份。
- ME3 启动前检查会读取版本；低于建议的 0.12.1 只警告、不阻断，以保留当前 0.11.0 Spacewar 已验证链路。
- MMV 的 Spacewar + Seamless 社区路线已完成本机阶段验收（地图/敌人、更多武器、602、登录和好友邀请）；MMV 作者支持的 Server Redirector 路线仍必须由具备正版环境的测试者指向干净 Steam `Game`、确认 Forsaken Hollows DLC 后另行验证。两条路线不得互相代替，也不能把本机阶段验收表述为作者路线或双人实战通过。
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
- 602 真实 ZIP 已通过管理器安装到当前 Game 的 `mods` 目录并完成游戏内观察。
  原 `NR0000.co2` 曾在多种 Mod 组合下表现为女爵空白；最终由用户确认根因是存档选择了
  “更多服装”提供、游戏本体不存在的服装。换回默认或本体已有服装后女爵恢复，说明
  MMV、602、Seamless、女爵面具 Mod 和存档主体均不是直接原因。此类问题应先恢复有效
  外观，再做同源存档 A/B；不能仅凭人物空白推断 regulation 或注入失败。
- A/B 期间发现 ME3 Profile 的自定义 `savefile` 在 Spacewar + Seamless 下并未
  隔离存档，实际仍写入 `NR0000.co2`。换回有效服装后的当前验收基线已备份，
  `NR0000.co2` 与 `.bak` 的 SHA-256 均为
  `F03D51FA08B3D92EAF1B72B0D365408899D832420DA02202694ACC247E72E379`；
  原始异常服装状态和各轮结果仍保留在受管备份中，没有删除用户进度。
- 管理器现改为：Seamless 环境始终按 `nrsc_settings.ini` 推断真实存档名；存档复制
  后执行 SHA-256 回读校验；记录上次管理器启动的 regulation 指纹；再次启动时若
  玩法参数变化或旧存档来源未知，启动前检查会提示人物/武器不显示风险。该保护
  不会自动删除或替换用户进度，是否迁移到新角色仍由用户决定。
- 诊断页已实现双方联机一致性清单。导出内容不含绝对路径、用户名、账号或存档，
  但会比较完整 package 树、加载顺序、`regulation.bin`、602 item/menu、native
  哈希与 early-load、`nightreign.exe`、三个 Spacewar 关键 DLL 和
  `nrsc_settings.ini`。好友清单的 schema、2 MB 大小上限和总体内容指纹会在比较前
  校验，手工篡改或损坏会被拒绝。
- 真实 MMV + Weapons + 602 + nrsc 目录已完成 2.74 GB 全量读取：MMV package
  1,439 个文件、2,740,381,704 bytes，树指纹
  `ECCCD4F2614E7F364F169A64B1EFEE7CB1ED6F08D8E53660C46EB736FA9A3F72`；
  602 package 2 个文件、324,608 bytes，树指纹
  `3790898ACC858DFBD8FC6E4AECB64D0391573FAA4E0FA4FB0492C2D358000219`。
  最终脱敏清单仅 2,310 bytes，总体指纹
  `4AEAC4B791976335C1CD44C0F608BF6585072373C7194156004D232D27DD1BF0`，
  没有绝对路径；完整计算耗时约 81 秒。

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

Rust 单元测试结果：44 项（38 passed，6 ignored，0 failed）。六项 ignored
分别用于真实下载 MMV 作者 Profile、真实 MMV 社区转换、当前工作区只读集成、
602 真实 ZIP 安装和完整 MMV/602 联机清单，需显式提供本机环境变量。本轮已显式
运行两项 MMV Profile 测试和完整联机清单测试，均通过；602 真实 ZIP 此前已由
同一安装函数完成实际安装与哈希验证；
此前当前工作区检查同时确认自动识别为
Spacewar + Seamless，并因 MMV Server Redirector 环境错配而阻止启动。
`npx tauri build --debug --no-bundle` 已通过独立
`.build-tmp/mmv-seamless-tauri-target` 完成桌面应用构建，产物为
`debug/nightreign-mod-manager.exe`。本轮未结束用户进程，也未改写游戏目录、
外部 Mod 注册状态或启用状态。

### 2026-07-31 暂停交接

用户因额度即将不足要求在此暂停并转入新会话。暂停前已经完成以下闭环：

- 管理器新增“双方联机一致性”清单导出/比较；本机当前 `Spacewar + Seamless + 602`
  方案通过真实 UI 导出并回读同一 JSON，结果为阻断差异 0、提醒 0，总体指纹
  `2CB0CCC59DC295828F050336A71D560E3DE909CD6FF0415946E044E22820FCFE`。
- 完整 MMV + Weapons（2.74 GB）+ 602 + Seamless 样本此前已完成两次全树扫描，
  总体语义指纹为
  `4AEAC4B791976335C1CD44C0F608BF6585072373C7194156004D232D27DD1BF0`；
  暂停时发起的第三次重复扫描被用户主动中止，不代表测试失败，且已确认没有遗留
  `cargo`、管理器、ME3 或游戏进程。
- 最终静态回归已经通过：`cargo fmt`、`clippy -D warnings`、`cargo check`、
  44 项 Rust 测试、`npm run build` 和 `npm run lint`。
- 原始联机存档 `NR0000.co2` 与 `.bak` 均已恢复，SHA-256 都是
  `74E36DCC7F4A0A9065043F5DBD972C6659479A5F88B918E26C81551150685B8D`。
  暂停当时保持 MMV 禁用、602 启用；不要把这条历史状态当成当前状态。

新会话应直接读取本文件和 `docs/SPACEWAR_SEAMLESS_MODPACK_HANDOFF.md`，不要重复下载
或重新做已通过的静态验证。第一项同源存档真实游戏验收已于 2026-08-01 完成；剩余
联机验收是第二位玩家用自己的管理器导出清单并互换比较，再真正加入同一 Seamless
房间观察。只有该项完成后才能宣称双端一致性和双人实战通过。

### 2026-08-01 同存档验收恢复点

- 用户指出新存档不能用于证明原人物正常，后续游戏内操作改由用户人工完成并发送截图；
  代理只准备同源存档、Mod 组合、Profile、日志和哈希证据。
- 刚才被游戏写入的新存档已另存到
  `backups/saves/manual-new-save-after-game-20260801-1940/76561199403037768/`，
  SHA-256 为 `91B89F779643E2B27CE60086027D90CC2FD03B481A43BEC7CFBB594F00C9CC98`。
- 当时活动 `NR0000.co2` 与 `.bak` 已从同一原始备份恢复，SHA-256 均为
  `74E36DCC7F4A0A9065043F5DBD972C6659479A5F88B918E26C81551150685B8D`。
- 发现此前误启动了 2026-05-23 的旧安装程序；旧程序在切换外部 Mod 时会丢掉它不认识的
  `profile_mode` 字段，导致生成副本同时包含 Server Redirector 与 `nrsc.dll`。该次
  “无法连接服务器”不能作为社区模式失败证据。
- 已基于提交 `2680136` 重建并替换实际安装目录程序。旧 EXE 备份在
  `D:\Game\ELDEN RING NIGHTREIGN\nightreign-mod-manager\backups\app\20260801-2018\`；
  新安装 EXE SHA-256 为
  `C3A3744C6BB486BDB35CB893C7553E72EE18195221B70F9A8F1A04AE55724CCF`。
- 新程序的“不启动游戏”预检显示可以启动、Spacewar + Seamless、Server Redirector
  未使用、唯一 MMV `regulation.bin` 和唯一 602 中文层；实际重新生成的
  `active-nightreign.me3` 只含 early-load `nrsc.dll`、MMV + Weapons package 与 602，
  不含 `cl_server_redirector.dll`。
- 用户随后完成真实游戏观察：游戏服务器登录成功且 Steam 好友邀请可用，说明新版
  社区 Profile 的 `nrsc.dll` 联机链路成功，旧程序混入 Server Redirector 才是此前
  “无法连接服务器”的直接原因。除女爵外其他角色正常；女爵故障最终确认来自
  “更多服装”留下的非本体服装选择，换回默认或本体已有服装后立即恢复。
- 本次测试后的 `NR0000.co2` 与 `.bak` 已归档到
  `backups/saves/manual-duchess-finding-after-test-20260801-2055/76561199403037768/`，
  SHA-256 均为 `CA3A1081F047DD346539741722B3DE8091E0AFDADCA13E831E77F5CCD186150E`。
  活动双文件已再次恢复到同一原始哈希 `74E36D...85B8D`；下一轮已关闭
  MMV + Weapons、保留 602 + Seamless，只由用户验证女爵能否显示以及能否切走再切回。
- 上述“602 + Seamless、关闭 MMV”对照仍复现：女爵空白，切换到其他角色后无法切回。
  本轮 `active-nightreign.me3` 与 ME3 `2026-08-01_20-55-33.log` 明确只含 early-load
  `nrsc.dll` 和 602 package；运行时只覆盖 602 的 `item_dlc01.msgbnd.dcx`、
  `menu_dlc01.msgbnd.dcx`，没有 MMV package、`regulation.bin` 或任何女爵 parts。
  标题页脚显示的“MMV 2.1.7.1 / 武器模组兼容补丁”来自 602 菜单文本，不能作为 MMV
  主包加载证据。MMV 已从当前直接原因中排除。
- `女爵去除面罩.disabled` 仅含五个女爵 body parts（5030/5130/5230/5330/5530），
  没有 regulation，且 Profile/日志未引用；直接残留加载不成立。为验证这一点做过的
  纯 Seamless 隔离没有修改任何 Mod 文件内容。审计后已把面具 Mod 恢复到 `.disabled`、
  恢复 602 与 MMV 社区模式；当前使用换回有效服装后的 `F03D51FA...2E379` 存档作为
  新一轮完整整合验收基线。

### 2026-08-01 Spacewar 整合阶段验收完成

- 用户用修正服装后的同一存档完成两轮真实游戏测试。22:07 的 ME3 日志
  `2026-08-01_22-07-21.log` 同时记录 early-load `nrsc.dll`、MMV + Weapons package、
  602 package，并实际覆盖 MMV `regulation.bin` 与 602 的 item/menu 两个消息包。
- 中文开启时，新增武器“蕾菈娜的对剑”、战技“月与火的架势”和新增被动词条显示为
  中文；女爵使用本体有效服装时模型正常。这证明更多武器、602 文本层和人物渲染可
  在当前 Spacewar + Seamless 社区链路中同时工作。
- 22:14 关闭 602 后，生成 Profile 与 `2026-08-01_22-14-04.log` 只含 MMV package
  和 `nrsc.dll`。相同新增武器仍存在，但名称/战技/词条变成 `?WeaponName?`、
  `ArtsName(5170)`、`AttachEffectName(9040600)`；随后实际进入本体原先没有的白金之子
  主城。该 A/B 同时证明 MMV/Weapons 不依赖 602 才能加载，602 只负责对应文本。
- 此阶段可标记为：本机 Spacewar + Seamless 下，管理器生成社区 Profile、服务器登录、
  Steam 好友邀请、MMV 新地图/敌人、更多武器和 602 简中均已实测。仍未验证第二位玩家
  真正加入同一房间后的双人实战，也未逐项遍历所有地图、敌人和武器。
- 当前停留状态是 MMV + Weapons 启用、`mmv_seamless_community`、602 关闭、女爵面具
  Mod 保持 `.disabled`。当前 `active-nightreign.me3` 仅含 `nrsc.dll` 与 MMV package。
  验收后 `NR0000.co2` 与 `.bak` 已备份到
  `backups/saves/manual-stage1-mmv-validated-20260801-2230/76561199403037768/`，SHA-256
  均为 `C658A54655F621E78DFE20C451A8D584961D104BEFD979B1CD0C827D60AB96CD`。
- 下一对话不要重复下载或重新证明 MMV/602 是否加载。

### 2026-08-01 服装安全与玩家术语适配

- Rust 扫描器现在识别六类服装 parts 前缀、同名 `_l` 队友视角资源、根级
  `regulation.bin`、外观 ID 和作者附带的联机生成脚本。脚本只作为提示，不会执行。
- 纯 parts 显示为“服装替换”；parts + regulation 显示为“扩展服装”。扩展服装 ZIP
  先解压到暂存目录，结构检查完成后以 `.disabled` 安装；重名目标仍保留停用后缀。
  首次添加外部扩展服装目录也默认停用，重复添加已存在记录时保持原启停状态。
- 停用扩展服装、删除其记录或切换会停用它的配置方案前，界面要求玩家确认已换回本体
  服装；启用缺失/部分 `_l` 的服装时提醒“自己可能正常、队友可能异常”。
- Mod 卡片显示本机/队友 parts 数、配对状态、玩法数据文件、检测到的外观 ID 和手动
  脚本提示；启动前检查会汇总已启用服装及 `_l` 缺口。新增“服装”筛选。
- 主页面术语已改为“启动配置、资源型 Mod、功能插件、玩法数据文件、联机一致性清单、
  队友视角资源”，折叠的“高级技术详情”保留 Profile/package/native/regulation.bin/
  manifest 对照，便于排错和阅读作者文档。
- 两个本地样本只读验证通过：女爵去面罩为 5 个本机 parts、无 regulation、缺 5 个
  `_l`；SkinOverhaul 为 228 个本机 parts + 228 个 `_l` 全配对，并含 regulation 与
  `01_Online.bat`。验证没有运行 BAT，也没有改变任何 Mod 启停状态。
- 验证命令均通过：`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo check`、`cargo test`（41 passed，7 ignored）、真实服装样本 ignored test、`npm run lint`、`npm run build`。

2026-08-02 根据实际界面截图修正了一处分类误判：MMV package 虽含
`bd/hd_m_6030` 两组本机与 `_l` 人物部件，但同时包含 `map/chr/event/action/script/sfx`
等完整玩法目录，不应显示为“扩展服装”，更不应要求玩家在停用 MMV 前换回本体服装。
扫描器现在先判断 package 语义：大型玩法整合包仍接受玩法数据和文件冲突检查，但不进入
服装筛选、服装标签或外观 ID 存档提示。缩小测试和真实 MMV 目录测试均已通过；
SkinOverhaul 仍识别为 228 组扩展服装，女爵去面罩仍识别为纯服装替换。

用户确认 SkinOverhaul 实际位置为
`D:\Game\ELDEN RING NIGHTREIGN\mods\SkinOverhaul`。本机外部注册记录已从错误的
`Skin Overhaul` 修正为该路径，并同步迁移非当前配置方案中的路径派生 Mod ID；保持停用，
未启动游戏。运行中的旧窗口需要刷新；若不是 Tauri dev 热重载实例，则需重新启动新构建
才能看到 MMV 分类修正。

### 2026-08-01 启动体验与检查指引

- 旧启动路径使用 `cmd /K` 和 `CREATE_NEW_CONSOLE`，因此会弹出终端并在游戏退出后停留在
  提示符。现已改为后台 `cmd /C` + `CREATE_NO_WINDOW`；ME3 输出仍写入
  `launch\last-launch.log`，启动页和成功提示均指向诊断页，不再让玩家处理终端。
- 启动前检查现在按玩家语言显示“发生了什么 / 下一步怎么做”，红色为必须处理、黄色为
  风险提醒；原始路径、文件名、版本和后端报错收进“系统详情”。新增“启动说明”窗口，
  解释目录、联机方式、启动配置、资源型 Mod、功能插件、玩法数据文件、`_l` 资源和日志。
- 启动台、设置页和运行环境卡进一步替换裸露术语：用“社区联机插件”“社区联机运行组件”
  “作者指定的联机方式”等玩家名称，并在说明或高级详情保留 Seamless、Spacewar、
  Server Redirector、ME3 等对照。
- “应用联机补丁”和“恢复最近一次联机补丁备份”已从启动控制隐藏；后端命令暂保留，等待
  完整事务和发布回归后再评估，不要对当前用户目录自动复制或覆盖补丁文件。
- 开发窗口的启动说明和启动前检查已做只读界面回归；本轮没有点击启动游戏，也没有改变
  用户的 Mod、存档或联机文件。

尚未在本次交接前重新验证：

- `npx tauri build` 生产打包；
- 新安装包的安装/卸载；
- 第二位玩家加入同一 Seamless 房间后的双人实战；本机登录、Steam 好友邀请入口已通过；
- 纯正版 Steam、正版 Steam + Seamless 和 MMV Server Redirector 真实启动（当前用户无此环境，只完成静态规则与自动化测试）；
- 大型真实 Mod ZIP 的结构组合与冲突扫描性能。

最新安装程序已由用户完成真实启动并确认 MMV/Weapons/602 生效；生产安装包本身仍需另做安装/卸载回归。

## 建议实施顺序

### P0：最新工作树回归

1. 完全退出并重新打开 Codex，使更新后的全局 Skill 生效。
2. 从仓库根目录运行 `.\dev.bat`，确认 Tauri 不再反复启动退出。
3. 先点“启动前检查（不会启动游戏）”，确认无残留进程误报。
4. 只点击一次普通启动，确认游戏在后台启动、没有可见终端窗口、SeamlessCoop 和至少一个资源型 Mod 正常。
5. 退出游戏后再次检查进程保护和日志。
6. 检查 1160×760、960×640 两个窗口尺寸的主要页面。

### P1：Mod 健康检查

- 检测压缩包多套一层、无法识别的入口、`.me3` 引用缺失、DLL/资源包类型和已知依赖。
- UI 区分“目录已启用”和“预计可以加载”，并提供可执行的修复提示。
- 继续用真实 Mod 覆盖单根目录、多根目录、`.me3`、DLL-only、资源包-only 和混合型 ZIP。
- 服装首轮结构检查已经完成；继续补充半套 `_l`、多部位命名和多个 regulation 组合的
  真实 ZIP。当前只识别作者脚本，不执行或代替作者生成队友资源。

### P1：联机一致性清单增强

- 已完成脱敏导出、好友 JSON 导入、内容完整性校验和阻断差异分类。
- 后续增加大型目录的进度、取消和可靠缓存，以及按差异定位对应 Mod 的一键建议。
- 第二位玩家必须使用自己的管理器导出清单并与本机互换，才能把“双端一致”标记为真实验收完成。

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
