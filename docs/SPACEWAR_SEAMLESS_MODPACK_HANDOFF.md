# Spacewar + Seamless 模组整合专项交接

更新时间：2026-07-31

## 新会话目标

在用户唯一可以真实测试的 `Spacewar + Seamless` 环境中，探索并最终验证一套：

```text
更多地图 / 更多敌人
+ 更多武器
+ 简体中文
+ SeamlessCoop / OnlineFix 联机
+ 当前管理器启动与方案管理
```

研究阶段已经找到有社区实例支持、可由管理器安全表达的兼容机制。仍不要直接删除
或覆盖作者包中的 DLL，也不要覆盖游戏目录；管理器只在生成 Profile 副本中做
显式、可逆的网络后端替换。最终目标仍是可复现配方、哈希和双方一致性验证。

## 已确认的当前环境

当前实际游戏与工具：

```text
游戏 Game：
D:\Game\ELDEN RING NIGHTREIGN\Game

ME3：
D:\Game\ELDEN RING NIGHTREIGN\me3

当前 Spacewar/OnlineFix 标记：
OnlineFix.ini
OnlineFix64.dll
steam_emu.ini
dlllist.txt
winmm.dll

SeamlessCoop：
Game\SeamlessCoop\nrsc.dll
Game\SeamlessCoop\nrsc_settings.ini
```

管理器自动检测结果为 `spacewar_seamless`，并标记为“已实测”。当前 ME3 检测为
`0.11.0`；管理器建议 `0.12.1` 或更高，但为了保留此前成功的 Spacewar 链路，
目前只警告、不阻止。

用户只能提供 `Spacewar + Seamless` 真实启动环境。纯正版 Steam、正版 Steam +
Seamless 和 MMV Server Redirector 不能宣称已经由用户实测。

## 2026-07-31 两张检查截图的结论

### 启用当前 MMV + Weapons 作者包

启用：

```text
D:\Game\ELDEN RING NIGHTREIGN\mods\More Map Variations 2.1.7-hotfix1 & Weapons Mod
```

管理器正确识别：

- 运行环境：`Spacewar + Seamless`；
- 作者 Profile：`savefile=NR0000.co2`、`start_online=true`；
- 网络后端：`cl_server_redirector.dll / Server Redirector`；
- 当前 Game 同时存在 OnlineFix/Spacewar 文件；
- Server Redirector 与当前环境冲突，启动被硬阻止；
- `NR0000.co2` 与 Seamless 默认存档同名，不是独立存档。

此前曾在未阻止的旧逻辑下实际启动：ME3 成功加载 MMV package 和 Redirector，
随后游戏显示无法登录服务器，并出现人物不显示。这个结果不能算 MMV 通过，只能
证明资源和 DLL 被注入。当前硬门禁是正确行为。

### 停用当前 MMV + Weapons 作者包

停用后启动台显示：

- 当前启用 Mod 为 0；
- `Spacewar + Seamless` 已实测；
- SeamlessCoop 已安装；
- OnlineFix/Spacewar 已安装；
- Server Redirector 未使用；
- 联机组件就绪。

因此当前阻塞点是 MMV 作者包选择的 Server Redirector，不是 Spacewar、
Seamless、OnlineFix 或管理器基础启动链路本身。

## 目标 Mod 与来源

- 更多地图/敌人原址：
  <https://www.nexusmods.com/eldenringnightreign/mods/578>
- 更多地图/敌人简中：
  <https://www.nexusmods.com/eldenringnightreign/mods/602>
- 更多武器原址：
  <https://www.nexusmods.com/eldenringnightreign/mods/554>
- 更多武器简中：
  <https://www.nexusmods.com/eldenringnightreign/mods/559>
- 参考整合视频：
  <https://www.bilibili.com/video/BV1zmV56CEJq/>
- 社区生成 Profile 示例：
  <https://github.com/DanMLWQ1/MMV-Seamless-Coop-Patch>
- Spacewar + ME3 + 合并 package 实例：
  <https://www.reddit.com/r/CrackSupport/comments/1rnfob5/how_to_install_more_map_variations_and_more/>

视频标题宣称 6 月 21 日版本可将 DLC 武器、Boss/敌人/地形扩容、资源点、6 人
Seamless 联机和深度自定义放在同一整合包中。2026-07-31 已核对简介和公开评论：

- UP 明确要求只使用已经合并的“更多地图 + 武器”包，不要把两个独立包叠加；
- 6 月 21 日大更新因变更很多要求完整重装；
- UP 把“无球/七仙女”等问题归因于 Seamless，并另配 4–6 人修复；
- 学习版/Spacewar 的 ME3 自定义启动目标仍是实际 `nightreign.exe`，不是
  `nrsc_launcher.exe`；
- 7 月初因官方更新出现的注入失败已由新 Seamless 补丁解决，不应采用替换旧游戏
  EXE 的危险做法；
- 普通 Seamless 版先于随机匹配服务器版发布，说明该整合不是必须依赖 Redirector。

视频评论是社区证据而非上游授权或长期兼容保证，必须继续用当前本地文件实测。

## 2026-07-31 社区兼容机制结论

当前可复现的核心不是重做 MMV/Weapons 的参数合并，而是：

```text
当前 MMV + Weapons 2.1.7 Hotfix 1 的单一预合并 package
+ 游戏目录现有 SeamlessCoop\nrsc.dll（load_early）
+ Spacewar 已验证启动参数
+ 一份与合并包同步的 602 简中覆盖
```

社区公开 Profile 和 Spacewar 实例均使用相同结构：package 指向合并后的 `mod`，
native 指向 `nrsc.dll` 并 early load，自定义 EXE 指向 `nightreign.exe`。公开
GitHub 示例仓库没有许可证、维护量很低，关联补丁也曾因原作者权限被移除，所以
只能把它作为机制证据；不得下载镜像后重新分发或把它包装成 MMV 官方支持。

Nexus 578 当前官方说明相反：2.0+ 要求 DLC，作者不再支持 Seamless，并要求使用
Server Redirector。管理器因此保留作者模式的严格 Steam 门禁，把社区转换做成
用户显式选择的独立模式，不会静默改变作者 Profile。

Nexus 602 当前主文件上传于 2026-07-22（314 KB），说明称覆盖地图与武器文本，
可用于两者或合并包；Changelog 明确记录同步 2.1.7.1 新增的几个武器被动词条。
页面顶部仍显示 2.1.6，因此要以上传日期和文件哈希识别当前文件。作者表示 4–6
人修复合并因未获得 MMV 作者许可只私下分享。当前专项先完成 3 人以内基础
Seamless 验证，不把 6 人修复混入首轮。

## 已有本地研究样本

### 当前 MMV + Weapons 作者包

```text
D:\Game\ELDEN RING NIGHTREIGN\mods\More Map Variations 2.1.7-hotfix1 & Weapons Mod
```

结构和作者 Profile 已被管理器正确解析。包内关键运行项是：

```text
package: mod
native:  mod\Server Redirector\cl_server_redirector.dll
savefile = "NR0000.co2"
start_online = true
```

该包只含 `engus/jpnjp` 文本，没有 `msg\zhocn`，本身不是中文方案。

2026-07-31 只读样本基线：

```text
文件数：1440
总大小：2,740,381,932 bytes
mod\regulation.bin：
D36B9960E19C748112F2A8D0D4C00D33A2BEC8AE9BB1707975516C3DBB64F579
mod\Server Redirector\cl_server_redirector.dll：
9413C4E6C4CC6D958E4B3DD4756548168749622AD2E0D5A4BD9AD3295FF2350C
Game\SeamlessCoop\nrsc.dll：
243EEC929A97B71E1E2E3B4215778B89C37D629436B8DD5403E830593D3CE24E
```

这些哈希只描述本机当前样本，不代表官方固定版本。实际联机双方仍应比较各自生成的
清单；上游更新后哈希变化是正常的，但必须双方一致。

### 旧 NightreignPLUS 5.30

```text
D:\Game\ELDEN RING NIGHTREIGN\mods\5.30 NightreignPLUS
```

它是最重要的本地机制证据，但已经过时，不能直接用于当前游戏。已确认它通过：

1. 预合并的 `More Map and Weapons` package；
2. profile 注入 `SeamlessCoop\nrsc.dll`；
3. 单一合并后的 `regulation.bin`；
4. 固定的 package/native 顺序；
5. 人工处理资源和材质冲突；

实现过“地图/敌人 + 武器 + Seamless”。这说明技术上存在社区兼容分支，但不能
证明当前 MMV 2.1.7 Hotfix 1 与当前游戏仍可按相同步骤工作。

详细审计见：

- `docs/NIGHTREIGNPLUS_AUDIT.md`
- `docs/DOWNLOADED_PACKS_MANAGER_AUDIT.md`

### SkinOverhaul

```text
D:\Game\ELDEN RING NIGHTREIGN\mods\SkinOverhaul
```

本专项目标暂不包含 Skin。该目录包含旧 MMV + Skin 合并 regulation，不是干净的
独立 Skin 包，必须保持停用，不能拿它覆盖新的玩法整合。

## 中文层已知边界

602 与 559 不是可同时叠加的“互补汉化”：

- 两者会争用同一批 `msgbnd.dcx`；
- 602 更新时间更晚，并声称覆盖地图、武器和官方合并版，是第一候选；
- 559 作为备选，用于比较武器名、战技、护符和庇佑文本；
- 首轮只测试一份汉化；
- 汉化应作为最终覆盖层，记录每个覆盖文件的哈希；
- 页面版本号与实际文件更新时间可能不一致，必须进游戏抽查。

如果候选社区整合本身包含中文，仍要查清它基于 602、559、自制翻译还是旧版文本，
不能再叠加另一份汉化。

本机当前两份停用汉化均来自 2026-05-29：一份把两个文件直接放在 package 根，
另一份缺少标准 `msg\zhocn` 层。它们早于 602 的 2026-07-22 当前主文件，不能
用于本轮成功验收，必须继续停用。

## 下一会话的研究顺序

### 1. 先研究，不改游戏目录

检索以下位置：

- Bilibili 视频简介、置顶评论、作者回复和最近更新；
- Nexus 578/554/602/559 的 Description、Files、Posts、Bugs、Changelogs；
- Nexus 中已删除的 Seamless patch 只记录历史，不重新推荐来路不明的镜像；
- Reddit、GitHub、Discord 公告、Steam 社区或其他玩家讨论中的当前版本反馈；
- 重点关键词：
  `MMV Seamless`, `More Map Variations OnlineFix`,
  `More Map and Weapons Seamless`, `NightreignPLUS`,
  `Server Redirector Spacewar`, `黑夜君临 MMV 无缝联机`。

每条社区结论记录发布日期、游戏版本、MMV 版本、Seamless 版本、下载来源和至少
一个实际反馈；旧版成功不能外推到当前版。

### 2. 只读拆解候选包

若找到候选下载，先放在实际 `Game` 目录之外。检查：

- `.me3` 中 package/native、`load_before/load_after`；
- 是否仍含 `cl_server_redirector.dll`；
- 是否含 `nrsc.dll`，版本是否与当前 Seamless 一致；
- 是否只有一个最终 `regulation.bin`；
- 是否包含 `msg\zhocn`；
- 是否引用 DLC 文件；
- 是否有 BAT/EXE 安装器以及它实际会复制什么；
- 所有关键文件 SHA-256。

未经检查不要运行候选包中的 EXE/BAT。

### 3. 最小化验证顺序

候选路线确认后，在可恢复副本或外部 Mod 目录中按以下顺序测试：

1. Spacewar + Seamless，无玩法 Mod；
2. 在 Mod 卡片为当前 MMV + Weapons 外部包选择“社区 Seamless 兼容”，确认
   启动前检查只显示 `nrsc.dll`、唯一 `regulation.bin` 及其 SHA-256；
3. 确认人物显示、主菜单、单人进入地图和存档正常；
4. 验证更多地形/敌人；
5. 验证更多武器；
6. 安装并只启用 602 于 2026-07-22 上传的 314 KB 中文主文件，确认路径是
   `msg\zhocn\item_dlc01.msgbnd.dcx` 与 `menu_dlc01.msgbnd.dcx`，启动前检查
   显示唯一中文层和两个哈希；进游戏抽查 MMV 2.1.7.1 新增武器被动词条；
7. 双方使用完全相同的 package、DLL、regulation、中文文件和加载顺序后联机；
8. 每一步退出后检查日志和存档备份。

若第 2 步需要把 `start_online=true` 或 `--online` 当作官方匹配解锁使用，必须先
确认它不会绕回 Server Redirector；Spacewar 路线应继续由管理器使用已验证的
`--skip-steam-init --online`，网络 native 只加载 Seamless `nrsc.dll`。

## 候选方案分级

| 方案 | 当前判断 | 下一步 |
|---|---|---|
| 当前 MMV 2.1.7 Hotfix 1 + Weapons 作者包原样加载 | 不可行 | 它绑定 Server Redirector |
| 管理器生成副本中移除 Redirector，改用现有 nrsc.dll | 有社区证据、本地静态验证通过 | 待 602 2026-07-22 主文件与真实单人/双人游戏验收 |
| 旧 NightreignPLUS 5.30 原样加载 | 不可行 | 版本过时，只作为机制证据 |
| 当前维护的 NightreignPLUS/社区 Seamless fork | 尚未找到可信、获授权且持续维护的发行物 | 不使用已删除补丁镜像 |
| 自行把当前 MMV/Weapons 合并为 Seamless 分支 | 技术上可能但维护成本最高 | 只有在找不到可信社区分支后再做 |
| 602 + 559 同时启用 | 不可行 | 二选一，首测 602 |

## 当前恢复点

截至 2026-07-31，本机下载目录和实际游戏 Mod 目录均没有 602 于
2026-07-22 上传的 314 KB 主文件。Nexus 文件下载需要用户登录，自动浏览器访问
文件页时站点关闭连接；不要从未经作者授权的镜像取得文件。用户登录 Nexus 下载后，
只需提供 ZIP 绝对路径，即可从上面的最小化验证顺序第 6 步继续。管理器会在安装时
规范 `msg\zhocn` 布局，并在启动前显示中文关键文件 SHA-256。

本机虽有 `D:\MO\ModOrganizer\nxmhandler.exe` 并注册了 `nxm://`，但没有
Nightreign 的 MO2 实例，也没有 Nexus API Key、Vortex 或 Nexus Mods App。
不要为绕过这一步自动创建 MO2 实例或读取其账号凭据。

当前 7 月主文件的 Nexus 版本历史标识为 `4318`。登录后可用
`https://www.nexusmods.com/eldenringnightreign/mods/602?tab=files&file_id=4318`
直接定位；仍需核对 2026-07-22 和 314 KB，不能把 `file_id` 当成无需登录的下载
令牌。

### 复用 Cyberpunk 项目的已验证下载流程

参考：
`D:\Project\Game-create\Cyberpunk2077-ModManager\docs\reviews\2026-07-12-in-app-browser-download-real-sample-loop.review.json`

1. 用户在 Codex 内置浏览器完成 Nexus 登录和可能出现的 Cloudflare 人机验证；
2. 继续控制同一标签页，不要重新打开或刷新验证页；
3. 使用精确 `file_id=4318` 进入当前文件的下载确认页；
4. 同时核对 `zhocn`、2026-07-22、314 KB；
5. 唯一定位 Slow download，点击时同步等待下载事件，免费通道允许至少 80 秒；
6. 下载事件超时后先查 `Downloads` 是否落盘，不要立刻刷新页面；
7. 计算 ZIP 的 SHA-256，记录来源 URL、文件标识、日期、大小、目录结构和人工复核
   状态，再交给管理器安装。

这只用于开发期获取真实测试样本。管理器产品本身继续保持本地 ZIP 导入，不新增
Nexus Cookie、账号、API Key 或在线下载权限。

## 管理器当前支持与待补能力

已经完成：

- 显式运行环境识别；
- Spacewar 启动参数；
- Server Redirector/Seamless 冲突硬门禁；
- 作者 `.me3` 语义保真解析；
- 启动前存档备份；
- OnlineFix 补丁事务备份和恢复；
- 文件级同名冲突分析。
- 外部作者 Profile 的显式社区 Seamless 兼容开关；
- 生成副本转换，原 `.me3`、DLL 和 package 保持只读；
- 唯一 `regulation.bin` 门禁和 SHA-256；
- 单一完整 `msg\zhocn` 中文层检查与 SHA-256；
- 本地 MMV 2.1.7 Hotfix 1 + Weapons 真实目录的只读集成测试。
- ZIP 安装器保留 `msg/parts/regulation.bin` 等语义根，只剥真实包装目录；
- 602 常见的根级或 `zhocn\` 两文件布局会自动规范成标准 `msg\zhocn\`，
  多套/半套布局会整包回滚。

本专项后续需要新增：

- 方案级关键文件哈希清单；
- 双方联机清单比较；
- 中文覆盖层来源/版本元数据，而不只依赖目录结构与哈希；
- 可选的 4–6 人修复独立方案（只有获得合法文件和当前版本实测后）。

不要为了让当前 MMV 通过而放宽 Server Redirector + Spacewar 的硬门禁。作者模式
仍必须被阻止；只有用户显式选择、通过结构验证的社区模式才会在生成副本中替换
网络 native。

## 权限与发布边界

- MMV、MoreWeapons、Skin Overhaul 当前记录的 Nexus 权限均不允许直接上传到
  其他站点；
- 602/559 允许署名转载，但仍优先保留作者来源和更新入口；
- 视频附件可以发布管理器、配置配方、哈希清单和经授权文件；
- 未获许可的第三方资源不能重新压包公开分发；
- 社区网盘整合包即使能下载，也不自动获得二次发布权限。

## 新会话建议开场

可直接向新会话发送：

> 阅读 `AGENTS.md`、`docs/CURRENT_STATUS.md` 和
> `docs/SPACEWAR_SEAMLESS_MODPACK_HANDOFF.md`。继续只读探索如何在当前
> Spacewar + Seamless 环境实现“更多地图/敌人 + 更多武器 + 简体中文”。
> 优先检查 Bilibili BV1zmV56CEJq 的简介/评论/作者回复，以及 Nexus
> 578/554/602/559 的 Posts、Files、Bugs 和 Changelog。先形成带版本、日期、
> 来源和可信度的候选清单；不要修改游戏目录，不要运行未知 EXE/BAT，也不要
> 放宽管理器现有的 Server Redirector + Spacewar 硬门禁。
