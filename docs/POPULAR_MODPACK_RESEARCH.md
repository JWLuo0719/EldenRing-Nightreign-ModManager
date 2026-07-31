# 热门模组整合探索：地图/敌人、武器、汉化与皮肤

探索日期：2026-07-30

本轮候选：

- [More Map Variations（用户称“更多敌人”）](https://www.nexusmods.com/eldenringnightreign/mods/578)
- [More Map Variations 简体中文翻译](https://www.nexusmods.com/eldenringnightreign/mods/602)
- [SotE and ds3 weapons（更多武器）](https://www.nexusmods.com/eldenringnightreign/mods/554)
- [更多武器简体中文翻译](https://www.nexusmods.com/eldenringnightreign/mods/559)
- [Add More Skins for Nightreign - Skin Overhaul](https://www.nexusmods.com/eldenringnightreign/mods/289)

结论性质：基于 2026-07-30 公开页面、本机已有翻译文件和当前管理器代码的兼容性审计。本轮没有下载或重新分发上述大型 Mod，也没有修改游戏目录。

## 一、结论

不能把这五项当成五个普通 Mod 按加载顺序直接叠加，但社区维护的
NightreignPLUS 已证明：通过预合并资源、参数级合并、人工删除冲突皮肤和持续
联机回归，可以制作特定版本的“一包全开”兼容分支。

建议拆成两个互斥的整合方案：

### 方案 A：Daybreak 玩法扩展中文方案

组成：

1. 使用 More Map Variations 页面提供的官方 `More Map Variations and Weapons Mod Merge` 完整预合并包；
2. 只选择一个简体中文翻译，第一候选为 602；
3. 使用作者随包提供的 Server Redirector；
4. 不加载独立 MoreWeapons、不加载两份汉化、不加载 SeamlessCoop、不加载 Skin Overhaul。

这是五个候选里最值得先验证的组合。它把地图/敌人变化和武器扩展放进作者维护的同一兼容版本，避免管理器尝试“用加载顺序冒充文件合并”。

### 方案 B：Skin Overhaul 外观方案

组成：

1. Skin Overhaul 1.9.0；
2. 可选的作者武器文件；
3. 可评估 2026-07-02 发布的社区更新 [Skin Overhaul UPDATE and Italian](https://www.nexusmods.com/eldenringnightreign/mods/737)；
4. 按原作者说明运行 `01_Online.bat`；
5. 在线环境使用 SeamlessCoop。

该方案应独立于方案 A。Skin Overhaul 不只是替换贴图：它添加新皮肤、新武器、修改 `regulation.bin`，并依赖批处理生成在线使用所需的第二份文件。当前没有发现它与 More Map Variations 官方合并包兼容的作者说明。

### 社区兼容分支：NightreignPLUS

用户提供的 5.30 旧包和 [6.21 更新视频](https://www.bilibili.com/video/BV1zmV56CEJq/)
构成了一个可核验的反例。其机制不是普通加载顺序：

1. 使用已经合并的 `More Map and Weapons`；
2. 用 ERBM 合并该玩法包与 Skin Overhaul 的 `regulation.bin`；
3. 把合并输出放进后加载的 `SkinOverhaul`；
4. 对材质冲突人工移除皮肤；
5. 用 ME3 profile 固定 SeamlessCoop、nighter 和两个最终 package。

详细本机只读审计见
[`docs/NIGHTREIGNPLUS_AUDIT.md`](NIGHTREIGNPLUS_AUDIT.md)。

### 本项目暂不直接发布“一包全开”

技术可行不等于本项目已经具备发布条件。在出现以下任一条件前，不制作或公开
分发地图/武器/皮肤的非官方全合并包：

- Team Daybreak 或 Skin Overhaul 作者发布正式兼容包；
- 取得各方明确许可并完成真正的参数、事件、资源和文本合并；
- 有可重复的双人联机、存档回退和跨版本回归结果。

该方向保留为备用兼容性实验，不作为当前官方版本的默认管理器建议。当前主线
方案见 [`docs/CURRENT_VERSION_MODPACK_PLAN.md`](CURRENT_VERSION_MODPACK_PLAN.md)。

## 二、版本与权限快照

页面顶部版本有时与文件区版本不一致。整合清单应以实际下载文件条目的版本、日期和哈希为准。

| 项目 | 当前公开文件 | 日期/大小 | 关键要求 | 再分发 |
| --- | --- | --- | --- | --- |
| More Map Variations | 官方单体 2.1.7.1 | 2026-07-03 / 2.3 GB | 必须拥有 Forsaken Hollows DLC；使用 ME3；当前改用 Server Redirector | 禁止上传到其他站点 |
| MMV + MoreWeapons 官方合并 | 2.1.7.1 | 2026-07-03 / 2.5 GB | 安装前完整移除旧版本；不应再叠加独立 MoreWeapons | 禁止上传到其他站点 |
| More Map 简中 602 | 页面顶部 2.1.6；Changelog 2.1.7.1 | 2026-07-22 / 314 KB | 已同步地图、武器、合并版及 2.1.7.1 新增武器被动词条 | 可转载但必须署名 |
| MoreWeapons 单体 | 0.3 | 2026-03-23 / 246 MB | 使用 ME3；已被 MMV 官方合并版包含 | 禁止上传到其他站点 |
| MoreWeapons 简中 559 | 0.3 | 2026-03-24 / 303 KB | 0.27.1 后也包含 MMV 中文；覆盖原 Mod 的 `msg` | 可转载但必须署名 |
| Skin Overhaul | 1.9.0 | 2026-01-25 / 1.5 GB | 标注 Game 1.03.2、Calibration 1.03.4；运行 `01_Online.bat`；在线需 SeamlessCoop | 禁止上传到其他站点 |
| Skin Overhaul 社区更新 | 1.0.0 | 2026-07-02 / 326.7 MB | 先完整安装原版，再覆盖并运行 `02_UPD_Online` | 需另查该页面权限后决定 |

三个原始内容包均禁止上传到其他站点，因此公开整合方案只能提供原始页面链接、版本、哈希和安装验证，不能把它们重新压进网盘包。两份汉化允许署名转载，但为了保持作者下载统计、更新来源和权限边界，首轮仍建议只发链接。

## 三、为什么 More Map 和 MoreWeapons 必须用官方合并版

More Map Variations 当前页面明确写明：

- Forsaken Hollows DLC 为必需；
- 不再支持 SeamlessCoop，应使用随包提供的 Server Redirector；
- 不建议使用 Mod Manager 或 Auto-Merger；
- 不为非官方合并和管理器造成的问题提供支持；
- 文件必须放在实际 Steam 游戏目录之外，再运行作者提供的 `.me3`；
- 2.1.7.1 文件区已经提供 2.5 GB 的 `More Map Variations and Weapons Mod Merge`。

这意味着“两个 package 谁放后面”并不能解决冲突。两个大修会共同修改参数、事件、资源和文本，后加载只会覆盖同名文件，不会合并其中的数据行。

对本管理器的直接影响：

- 当前 ZIP 安装固定解压到 `{game_path}\mods`，与作者要求“不要放在实际游戏目录内”冲突；
- 当前管理器会重新收集 package/native 并生成自己的 `active-nightreign.me3`，不能保证保留作者配置；
- 当前只要游戏根目录存在 `SeamlessCoop\nrsc.dll`，生成 profile 时就会自动注入并设置 `load_early = true`；
- More Map Variations 已明确不支持 SeamlessCoop，当前自动注入会制造一个用户看不见的冲突；
- 当前冲突分析可以报告同名文件，但不能生成真正的参数合并包。

因此，方案 A 需要的不是增强 ZIP 解压，而是“外部作者配置”运行模式。

## 四、中文翻译只能二选一

两份汉化不是前后互补关系。

602 页面说明：

- 路径为 `msg/zhocn`；
- 可用于 More Map Variations、MoreWeapons 或两者合并版；
- 2026-07-22 的当前文件说明称已同步更新地图和武器，Changelog 标注 2.1.7.1。

559 页面说明：

- 通过覆盖原 Mod 的 `msg` 同级目录安装；
- 版本 0.3 对应 MoreWeapons 0.3；
- 0.27.1 后已经合并 More Map Variations 的简中翻译。

本机现有两个禁用汉化样本也证明它们会争用相同逻辑文件：

```text
item_dlc01.msgbnd.dcx
menu_dlc01.msgbnd.dcx
```

两组文件大小和 SHA-256 不同，说明后一份会真实覆盖前一份，而不是无损叠加。

第一轮建议：

- 默认测试 602，因为更新日期更晚且明确声称适配官方合并版；
- 将 559 作为替代翻译，重点比较武器名称、战技、护符和庇佑文本覆盖；
- 不允许同时启用；
- 清单中使用 `conflictsWith` 表达互斥；
- 602 页面顶部仍显示 2.1.6，但 2026-07-22 的 314 KB 主文件和 Changelog 已同步
  2.1.7.1；版本记录必须同时保存上传日期和文件哈希，不能只读顶部版本号。

## 五、为什么 Skin Overhaul 必须单独成包

Skin Overhaul 1.9.0：

- 添加大量新皮肤，而不是简单替换一个 `parts` 文件；
- 同时添加 11 件新武器；
- 更新历史明确提到修改 `regulation.bin`；
- 带独立 `.me3`；
- 需要运行 `01_Online.bat`，为在线识别复制和重命名皮肤文件；
- 在线流程要求把 SeamlessCoop 放入 Skin Overhaul 的工作目录；
- 页面只提供 Elden Vins 的兼容版本，没有提供 More Map Variations/MoreWeapons 兼容版本。

它与方案 A 至少存在四类风险：

1. `regulation.bin` 和参数表冲突；
2. 新武器 ID、物品表、名称和图标冲突；
3. `.me3` 与运行目录结构冲突；
4. SeamlessCoop 与 Server Redirector 的联机环境冲突。

此外，当前本机 `nightreign.exe` 文件元数据为 `1.3.3.0`，但该数值不能替代游戏标题画面的 App/Regulation
版本。Bandai Namco 的 [1.03.5 官方热修说明](https://en.bandainamcoent.eu/elden-ring/news/elden-ring-nightreign-hotfix-1035)
列出的当前组合是 App 1.03.2 / Regulation 1.03.5，而 Skin Overhaul 主文件标注 Game 1.03.2 /
Calibration 1.03.4。App 版本一致、规则版本落后一档，必须把它标为兼容性警告，并通过真实启动、换肤、掉落、
联机和回退验证。

## 六、兼容矩阵

| 组合 | 判断 | 原因/处理 |
| --- | --- | --- |
| MMV 单体 + MoreWeapons 单体 | 禁止自行叠加 | 使用作者官方 2.1.7.1 合并版 |
| MMV + Weapons 官方合并版 + 602 | 首选候选 | 602 声称支持合并版；仍需验证页面版本差异 |
| MMV + Weapons 官方合并版 + 559 | 备选候选 | 武器文本可能更完整，但更新时间更早 |
| 602 + 559 | 互斥 | 覆盖相同 `msgbnd.dcx` 文件 |
| MMV/Weapons + SeamlessCoop | 不支持 | MMV 作者明确要求改用 Server Redirector |
| MMV/Weapons + Skin Overhaul | 社区已实现，但高维护 | NightreignPLUS 通过参数合并和人工裁剪实现；无官方支持，不纳入首轮自动化 |
| Skin Overhaul + SeamlessCoop | 作者说明支持 | 按 Skin Overhaul 独立目录和批处理流程 |
| Skin Overhaul + 其可选武器 | 作者说明支持 | 必须基于 Skin Overhaul 1.9.0 覆盖 |
| Skin Overhaul + MoreWeapons | 不建议 | 两者都添加武器并修改相关参数 |

## 七、管理器需要的新模式

为了尊重作者安装方式，同时保留本管理器的检查和视频展示价值，建议新增“外部作者配置”模式：

### 注册而不接管

- 用户选择作者提供的 `.me3` 和工作目录；
- 管理器只保存引用，不移动、重命名或删除作者文件；
- 禁用本地 ZIP 安装、目录 `.disabled` 重命名和自动 package 推断；
- 显示“作者配置 / 管理器托管配置”的明显区别。

### 运行环境隔离

每个方案声明：

```text
runtime = vanilla | seamless-coop | server-redirector
```

- `server-redirector` 方案禁止自动注入 `nrsc.dll`；
- `seamless-coop` 方案检查双方的 `nrsc.dll` 和设置；
- 启动前显示本次实际使用的网络组件；
- 同一方案不能同时启用 SeamlessCoop 和 Server Redirector。

### 保留作者 profile

- 默认直接启动作者 `.me3`；
- 不重新解释、排序或重写作者的 packages/natives；
- 只有汉化覆盖层可作为显式、可回退的本地补丁；
- 修改前保存文件清单和哈希，不直接覆盖唯一副本。

### 整合方案健康检查

方案 A 至少检查：

- Forsaken Hollows DLC 状态；
- 游戏版本和 regulation 版本；
- 官方合并包版本为 2.1.7.1；
- 旧版本是否未完整卸载；
- Server Redirector 是否存在；
- `nrsc.dll` 是否会被误注入；
- 602/559 是否同时存在；
- 关键文件哈希与双方是否一致；
- 工作目录是否位于游戏目录之外。

## 八、首轮验证顺序

### A0：干净基线

1. 备份当前 profile、游戏配置和存档；
2. 不删除现有 Mod，只创建独立外部工作目录；
3. 禁用 SeamlessCoop 自动注入；
4. 只运行官方 MMV + Weapons 2.1.7.1 合并版；
5. 验证启动、圆桌、新地图、敌人、武器掉落和正常退出。

### A1：汉化

1. 在可恢复副本上覆盖 602；
2. 检查新增武器、战技、护符、地图 NPC 和敌人名称；
3. 记录缺失文本、英文回退和乱码；
4. 恢复后改测 559；
5. 选出一份默认翻译并固定 SHA-256。

### A2：双人

1. 双方使用相同游戏、DLC、合并包、汉化和哈希；
2. 使用 Server Redirector，不启动 SeamlessCoop；
3. 验证大厅、掉落、敌人、Boss、地图事件和退出；
4. 故意改变一份汉化或一个关键文件，确认一致性检查能发现；
5. 问题由整合方案维护者复现，不把管理器问题推给原作者。

### B：皮肤独立方案

1. 使用独立工作目录和独立 profile；
2. 安装 Skin Overhaul 1.9.0；
3. 运行 `01_Online.bat`；
4. 可选覆盖社区更新并运行 `02_UPD_Online`；
5. 先验证单人所有职业皮肤，再验证 SeamlessCoop 双方显示；
6. 不加载 MMV、MoreWeapons 或 Server Redirector。

## 九、视频定位

官方支持路线仍适合拆成两条内容线；同时可以新增一条“成功超级整合包机制
拆解”，明确区分官方支持组合与社区维护兼容分支。

### 视频 1：玩法扩展

建议标题：

> 黑夜君临热门玩法整合：89 种野外 Boss + 60 种入侵者 + 50 多件新武器｜官方合并版中文实测

前 20 秒直接展示：

- 不同城堡布局；
- 新敌人/夜间 Boss；
- 新武器掉落；
- 管理器显示“官方合并版、单一汉化、Server Redirector、双方哈希一致”。

必须讲清：

- 需要 Forsaken Hollows DLC；
- 使用作者官方预合并版；
- 不使用 SeamlessCoop；
- 不重新分发原作者文件；
- 管理器目前若未实现外部 profile 模式，不能宣称一键导入。

### 视频 2：外观扩展

建议标题：

> 黑夜君临 Skin Overhaul：全职业新增皮肤实测｜无缝联机双方能否正确显示

重点展示：

- 每个职业的代表性皮肤；
- `01_Online.bat` 前后的文件检查；
- 双方是否看到同一模型；
- 与玩法大修分开配置的原因；
- 一键回退到原版/其他方案。

## 十、下一步决策

这轮最值得先实现的不是自动下载，而是：

1. 外部作者 `.me3` 注册与只读启动；
2. 每方案独立选择 `seamless-coop` 或 `server-redirector`；
3. 关闭全局自动注入 SeamlessCoop，改成方案级显式选择；
4. 汉化互斥与覆盖层哈希检查；
5. 官方合并包版本和旧版本残留检查；
6. 双方整合清单比较。

完成这些能力后，再下载官方 2.1.7.1 合并包做 A0 私有验证。未完成前，可以录制研究/预告，但不应发布“一键整合包已兼容”的结论。
