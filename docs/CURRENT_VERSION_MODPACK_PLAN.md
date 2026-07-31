# 当前官方版本的管理器整合方案

探索日期：2026-07-30

目标：将 NightreignPLUS 一类高维护非官方大整合作为备用研究线，主线优先推荐
当前官方游戏版本下结构简单、来源明确、可以回退、适合管理器检查的组合。

本轮只进行了网页资料核对、本机文件结构与哈希审计，没有下载新 Mod、运行游戏
或修改游戏目录。以下“可行”表示来源和结构满足候选条件，不等于已经完成当前
版本真实启动与联机回归。

## 一、当前基线

Bandai Namco 当前最新官方说明为：

```text
App Ver.        1.03.2
Regulation Ver. 1.03.5
```

来源：[ELDEN RING NIGHTREIGN – Hotfix 1.03.5](https://en.bandainamcoent.eu/elden-ring/news/elden-ring-nightreign-hotfix-1035)

管理器的建议页不能只显示“最新版”。应显示上述两个版本，并要求用户以游戏
标题画面右下角为准。以后官方更新时，所有方案先进入“待重新验证”，而不是继续
显示绿色兼容。

## 二、主线规则

当前版本的公开建议采用五条规则：

1. 一个方案只能有一个网络组件：无、SeamlessCoop 或 Server Redirector；
2. 一个简单方案只能有一个 `regulation.bin` 所有者；
3. 纯 `parts`、贴图或声音资源可以叠加，但相对路径不能冲突；
4. 翻译必须绑定对应主体和版本，不能作为独立玩法 Mod；
5. 作者提供完整预合并包时，不再加载被它包含的单体 Mod。

对应到管理器状态：

| 检查 | 通过 | 警告/阻止 |
| --- | --- | --- |
| 游戏版本 | App 1.03.2 / Regulation 1.03.5 | 版本未知或不同 |
| 网络组件 | 正好一个 | Seamless 与 Redirector 同时存在 |
| regulation | 0 或 1 个所有者 | 2 个以上且没有已知合并产物 |
| 翻译 | 主体存在、版本匹配 | 缺主体或两份翻译同时启用 |
| 资源覆盖 | 无同路径，或有方案白名单 | 未解释的同路径不同哈希 |
| 联机一致性 | 双方清单、顺序、哈希一致 | 缺失、顺序或配置不同 |

## 三、建议方案分级

### S0：管理器轻量示范包

建议名称：

```text
Nightreign Manager Starter — Duchess Unmasked
```

组成：

- [Duchess Unmasked 1.0](https://www.nexusmods.com/eldenringnightreign/mods/203)
- ME3
- 可选：当前 SeamlessCoop，但不作为资源包的一部分重新分发

选择理由：

- 只有 5 个 `parts/*.partsbnd.dcx`；
- 没有 `regulation.bin`、DLL、脚本或安装程序；
- 画面变化明确，适合视频前后对比；
- 当前管理器可以通过目录结构推断为 package；
- 开关通过目录重命名即可回退；
- 原页面允许在署名的前提下上传到其他站点。

本机现有样本：

```text
Game\mods\女爵去除面罩
```

共 5 个文件、127,616,992 字节，无 `regulation.bin` 和 DLL。现有 ZIP：

```text
SHA-256 2c1a1f2836570102102a42417c503bfcd885e0223a51709ad3eb4a8210a807f2
```

该哈希只能标识本机样本，尚未与 2026-07-30 重新下载的 Nexus 原档进行同源
校验，不能直接写成官方发布哈希。

已知边界：

- 原 Mod 最后更新于 2025-06-22，没有明确写出 1.03.5；
- 纯模型替换通常比 regulation Mod 更耐版本变化，但仍需当前版本烟雾测试；
- 本机样本没有 `_l` 镜像文件，不应在验证前宣称队友视角也能正确显示；
- 不在官方 EAC/匹配环境中建议启用 Mod；测试使用离线或独立 Mod 联机环境。

这个包最适合成为第一条“管理器从安装到回退”的短教程，因为失败面小，也能
真实展示扫描、启停、profile 生成、冲突检查和回退。

### S1：轻量 Seamless 联机包

组成：

- [Seamless Co-op (Nightreign)](https://www.nexusmods.com/eldenringnightreign/mods/3)
  主文件 v1.1.3，上传于 2026-07-02；
- S0 的 Duchess Unmasked，或另一个经过相对路径审计的纯外观包；
- 不启用任何 `regulation.bin` 大修。

使用建议：

- `nrsc.dll` 必须只有一份并提前加载；
- 双方使用相同 Seamless 版本和 `nrsc_settings.ini`；
- 外观包双方安装相同文件；
- 管理器导出相对路径、大小和 SHA-256，不导出绝对路径和账号信息；
- 外观的 `_l` 远端显示规则作为健康检查，不自动生成或覆盖作者文件。

S1 是现有管理器最值得做的实际回归：它同时覆盖 package、native early load、
联机配置和双方一致性，但不引入 regulation 合并。

### S2：4–6 人专用方案

组成：

- Seamless Co-op v1.1.3；
- [Nightreign 6 Player Fixes 1.1.1](https://www.nexusmods.com/eldenringnightreign/mods/596)；
- ME3；
- 不加入 MMV、MoreWeapons、Skin Overhaul 或其他 regulation Mod。

候选理由：

- 6 Player Fixes 页面明确写明更新到 Nightreign 1.03.5；
- 作者给出了 Seamless、主 package 和 `LilyHook.dll` 的 ME3 配置；
- 很适合检验管理器能否同时识别 package、native 和 early-load 依赖。

限制：

- 作者明确提示它与大多数其他 Mod 不兼容；
- 它修改 map、script、event 和 regulation，不是纯粹的小补丁；
- Seamless 已在 7 月更新，仍需验证与 4 月发布的 6 Player Fixes 组合；
- 只应作为“多人专用方案”，不能成为默认轻量包。

测试顺序：

1. 4 人大厅；
2. 6 人大厅；
3. 地图事件、绑架/传送事件；
4. Boss 进场和阶段切换；
5. 掉落可见性、复活、退出和重连；
6. 双方/全员配置哈希不一致时，确认管理器能够阻止启动或明确警告。

### A1：当前内容扩展高级方案

组成：

- [More Map Variations and Weapons Mod Merge 2.1.7.1](https://www.nexusmods.com/eldenringnightreign/mods/578?tab=files)；
- [More Map Variations 简体中文翻译](https://www.nexusmods.com/eldenringnightreign/mods/602?tab=files)，
  选择 2026-07-22 上传的 314 KB 主文件；页面顶部仍显示 2.1.6，但 Changelog
  明确标注同步 2.1.7.1；
- 作者随包提供的 Server Redirector；
- Forsaken Hollows DLC；
- 不加载独立 MoreWeapons、不加载第二份汉化、不加载 SeamlessCoop。

当前依据：

- 2.1.7.1 完整预合并包上传于 2026-07-03，大小 2.5 GB；
- 文件说明要求完整卸载旧版本，并包含 Server Redirector 稳定性修复；
- 2026-07-22 的中文主文件说明称已同步地图和武器，支持单体或合并版；Changelog
  明确记录同步 2.1.7.1 新增武器被动词条。仍需记录文件哈希并进游戏抽查。

这个方案内容吸引力最高，但不能用当前管理器的普通 ZIP 安装来承诺“一键使用”。
它应等待以下能力：

- 外部作者 `.me3` 原样注册；
- package 同时支持 `source` 与 `path`；
- 不重写未知字段和作者依赖；
- 方案级选择 Server Redirector，并禁止自动注入 `nrsc.dll`；
- 工作目录保持在作者要求的位置；
- 启动前检查 DLC、版本、旧版残留和中文覆盖层；
- 直接使用作者 profile，管理器只负责检查、启动和诊断。

此前出现的 `More Map Variations Seamless Coop patch` 页面现已被 Nexus staff
移除，不应作为当前公开推荐或下载来源。

### B1：超级整合备用研究线

NightreignPLUS 和 MMV/Weapons/Skin/Seamless 的人工大合并继续保留，但定位为：

```text
compatibility-lab / unsupported-community-merge
```

用途：

- 研究 ERBM 参数合并；
- 提取材质冲突白名单；
- 验证管理器的外部 profile 和构建溯源；
- 为未来有授权、可重复构建的社区兼容分支积累方法。

它不出现在新用户默认推荐中，也不与 S0/S1/A1 共用“兼容”绿色状态。

## 四、暂不推荐的候选

### nighter / 深夜解锁

Nexus 的公开文件仍是 2025-08-28 版本；搜索到的更新版主要来自第三方网盘和
视频说明，缺少稳定的官方版本页、源码或可验证发布链。不同说明还存在
`load_before nrsc.dll` 与 `load_after nrsc.dll` 的差异。

处理方式：

- 已有用户可以作为“自定义 DLL”注册；
- 管理器显示来源未知、版本未知、加载依赖待确认；
- 不进入当前版本默认整合包；
- 等找到作者稳定发布页并完成 1.03.5 回归后再升级为推荐项。

### 独立 MoreWeapons + MMV

不推荐。直接使用官方 2.1.7.1 预合并包。

### MMV + 非官方 Seamless 补丁

不推荐。相关 Nexus 页面已被 staff 移除，而且与 Team Daybreak 的当前
Server Redirector 路线相冲突。

### 两份中文翻译同时启用

禁止。它们覆盖相同的 `msgbnd.dcx`，只能选择一份。

## 五、管理器中的使用建议文案

设置或首页可以显示：

> 初次使用建议从一个纯外观 Mod 开始。它不修改 `regulation.bin`，便于确认
> ME3、管理器和回退流程正常。需要联机时，再单独建立 Seamless 方案并确保
> 所有玩家的 Mod、顺序和配置一致。大型玩法包请使用作者提供的预合并版本和
> 启动配置，不要把多个 `regulation.bin` 仅靠排序同时启用。

当检测到多个 regulation 时：

> 当前方案包含多个 `regulation.bin`。加载顺序只能覆盖文件，不能合并参数。
> 请改用作者预合并包、已验证的合并产物，或拆分为不同方案。

当检测到 MMV 与 Seamless 时：

> 当前 MMV 版本使用 Server Redirector。请停用 SeamlessCoop，或改用单独的
> Seamless 方案。管理器不会自动加载已被移除的非官方兼容补丁。

## 六、实施顺序

1. 用本机 Duchess Unmasked 做 S0 当前版本启动/回退验证；
2. 下载并校验 Seamless v1.1.3，在独立方案完成 S1 双人验证；
3. 实现清单导出/比较和 `_l` 外观健康提示；
4. 用 6 Player Fixes 做 S2 私有 4–6 人压力测试；
5. 实现外部 profile 原样运行；
6. 获取 MMV + Weapons 2.1.7.1 和 7 月中文翻译，完成 A1；
7. 最后再决定是否恢复 NightreignPLUS 大合并实验。

## 七、内容发布顺序

建议对应三条视频：

1. **入门短教程**：一个纯外观 Mod，从 ZIP 安装、启用、启动到一键回退；
2. **联机一致性教程**：Seamless 双方为什么必须版本、顺序和哈希一致；
3. **高级内容方案**：MMV + Weapons 官方预合并版中文实测，说明为什么不用
   独立 MoreWeapons 和 Seamless。

大整合拆解作为第四条研究内容，不与前三条的稳定使用建议混在一起。
