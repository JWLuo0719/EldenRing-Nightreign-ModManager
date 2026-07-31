# NightreignPLUS 5.30 整合机制审计

审计日期：2026-07-30

审计对象：

- 视频：[【黑夜君临MOD】6.21大更！可联机超级缝合整合包](https://www.bilibili.com/video/BV1zmV56CEJq/)
- 本机旧包：`D:\Game\ELDEN RING NIGHTREIGN\mods\5.30 NightreignPLUS`

本轮只读取目录、文本配置、日志、文件元数据和 SHA-256，没有运行包内的
`exe`、`dll`、`cmd` 或 `bat`，也没有修改游戏目录。

## 一、修正后的结论

NightreignPLUS 证明了“更多地图/敌人 + 更多武器 + Skin Overhaul +
SeamlessCoop”在社区维护的特定版本中可以一起运行。

它并不推翻“加载顺序不能代替合并”，反而提供了实证：

1. 地图/敌人与武器先使用已经合并过的 `More Map and Weapons`；
2. 再用 ERBM 对玩法包和 Skin Overhaul 的 `regulation.bin` 做参数级合并；
3. 将合并输出放回后加载的 `SkinOverhaul` 目录；
4. 对材质和皮肤冲突人工取舍，移除已知不兼容皮肤；
5. 用一个手写 ME3 profile 固定 native 依赖和两个最终 package 的覆盖顺序；
6. 游戏或上游 Mod 更新后重新下载、重写、合并并做联机回归。

因此，准确表述应是：

> “一包全开”技术上可行，但它是一个人工维护的兼容分支，而不是把若干原始
> Mod 交给通用管理器后自动排序就能得到的结果。

## 二、视频侧证据

视频简介明确写到作者花费约 10 小时重新测试兼容性、重写/重新下载 Mod，
并使用合并工具处理多个 Mod。公开评论中的更新记录还显示：

- 6.7、6.13 版本需要替换 `More Map and Weapons` 等目录；
- 6.21 因改动很多，旧版用户被建议完整重下；
- 为解决材质冲突，移除了若干守夜人/敌人皮肤；
- 联机双方需要相同版本，旧 SeamlessCoop 残留必须清理；
- 评论中持续出现模型缺失、Boss/掉落不同步、路径大小写、启动闪退等问题。

这些信息表明兼容性来自持续维护和人工裁剪，而不是一次合并后永久有效。

## 三、本机 5.30 包概况

| 指标 | 结果 |
| --- | ---: |
| 文件 | 3,308 |
| 目录 | 2,334 |
| 总大小 | 约 7.12 GiB |
| 主要 `.dcx` 文件 | 2,457 |
| DLL | 29 |

主要目录大小：

| 目录 | 文件数 | 大小 |
| --- | ---: | ---: |
| `SkinOverhaul` | 470 | 约 3.04 GiB |
| `More Map and Weapons` | 1,341 | 约 1.93 GiB |
| `More Map Variations` | 1,219 | 约 1.71 GiB |
| `MoreWeaponsTM` | 128 | 约 248 MiB |
| `Map and Weapons MergePatch` | 12 | 约 156 MiB |
| `ERBMv1.1` | 42 | 约 19 MiB |
| `6 player fixes` | 76 | 约 17 MiB |

7.12 GiB 不等于启动时同时加载了所有目录。包中保留了原始组件、兼容补丁、
合并工具、合并输出和最终运行目录，带有明显的“整合工作台”性质。

## 四、实际启动图

`Launch NightreignPLUS.me3` 只引用了四个运行项：

```text
native:  ./NightreignPLUS/SeamlessCoop/nrsc.dll
         load_early = true

native:  ./NightreignPLUS/DepthChange/nighter.dll
         load_after = nrsc.dll

package: ./NightreignPLUS/More Map and Weapons
package: ./NIghtreignPLUS/SkinOverhaul
```

也就是说，`More Map Variations`、`MoreWeaponsTM`、`Map and Weapons
MergePatch`、`ERBMv1.1`、`6 player fixes` 等目录不是该 profile 中的独立
package。它们更多是上游源件、补丁或制作过程的留档。

profile 中 `NIghtreignPLUS` 的大小写与实际目录 `NightreignPLUS` 不一致。
Windows 常见文件系统通常不区分大小写，但这仍是应被健康检查报告的路径质量
问题，视频评论中也出现过相关报错。

## 五、`regulation.bin` 的合并证据

关键文件：

| 相对路径 | 大小 | SHA-256 |
| --- | ---: | --- |
| `More Map and Weapons\regulation.bin` | 3,176,672 | `84d1269d...6ae88de` |
| `SkinOverhaul\regulation.bin` | 2,785,376 | `b06275a6...d6b51fd` |
| `ERBMv1.1\regulation_merged\regulation.bin` | 2,780,928 | `48433285...b2a7af` |
| `6 player fixes\regulation.bin` | 2,780,928 | `48433285...b2a7af` |

ERBM 日志记录了三次关键合并：

### 2026-05-30：玩法包 + 皮肤

```text
File1 = More Map and Weapons\regulation.bin
File2 = SkinOverhaul\regulation.bin
priority = PreferFile1
integrity = PASS
output SHA-256 = b06275a6...d6b51fd
```

输出哈希与当前 `SkinOverhaul\regulation.bin` 完全一致。这证明制作者将 ERBM
的合并结果放回了后加载的 Skin package。最终由 Skin 目录提供的不是原始
Skin regulation，而是保留玩法包高优先级后的合并版本。

### 2026-05-31：玩法包 + 6 人修复参数

```text
File1 = More Map and Weapons\regulation.bin
File2 = 6 palyer fixes\regulation.bin
priority = PreferFile1
integrity = PASS
output SHA-256 = 48433285...b2a7af
```

日志同时警告：

```text
BaseVersion  = 10340000
File2Version = 10350000
```

说明合并器认为结构完整，但输入基准版本并不一致。当前输出只与
`6 player fixes\regulation.bin` 相同，而 5.30 profile 并未引用该目录；
不能仅凭文件存在就断言这部分在该 profile 中生效。它可能是后续更新制作过程
留下的中间产物。

## 六、资源覆盖与人工裁剪

最终两个 package 共有三个相对路径冲突，且内容全部不同：

```text
material/allmaterial.matbinbnd.dcx
msg/engus/item_dlc01.msgbnd.dcx
regulation.bin
```

profile 把 `SkinOverhaul` 放在 `More Map and Weapons` 之后，因此这些文件由
Skin package 提供。其中：

- `regulation.bin` 已经是经过 ERBM 合并的版本；
- `allmaterial.matbinbnd.dcx` 无法靠参数合并解决，必须选择一个最终版本并
  围绕它删减冲突皮肤；
- 英文物品文本也发生覆盖，中文翻译还需要单独验证。

视频更新记录中移除 Ornstein、Smough、Rennala 等皮肤的做法，与本机文件
冲突结构相互印证。

## 七、为什么 5.30 已经过时

本机包最后修改时间停留在 2026-05-31，且至少存在以下陈旧信号：

- 内置 SeamlessCoop `version.json` 为 v1.1.2；
- ERBM 内置基准为 Game 1.03.1 / Calibration 1.03.3；
- 5 月 31 日的合并已经出现 1.03.4 与 1.03.5 binder 版本警告；
- 视频评论说明 6.21 改动很大，推荐旧用户完整重下；
- 公开视频搜索结果还可见后续 7.2 修复/更新内容。

因此，这个目录适合用于研究机制，不适合作为当前游戏版本的直接可运行包。
在未取得当前版本源件、变更记录和回归证据前，不应启动或覆盖到现有游戏环境。

## 八、对当前管理器的直接影响

当前管理器不能无损导入这个 profile：

1. package 解析只读取 `path`，而该 profile 使用 `source`；
2. 重新生成 profile 时会丢弃 native 的 `id`、`load_after`、`load_before`；
3. 只按路径去重，可能同时注入游戏根目录和整合包内的两份 `nrsc.dll`；
4. 当前会把目录视为普通独立 Mod，无法区分“源件/制作工具/最终运行 package”；
5. 冲突分析只能发现同名文件，不能确认 `regulation.bin` 是否完成参数级合并。

如果把整个 5.30 目录注册为一个外部 Mod，解析器会识别两个 native，却忽略
两个 `source` package；由于已经识别到 native，又不会进入目录推断回退，
最终生成的 profile 会缺少核心玩法和皮肤资源。

## 九、适合管理器的新抽象

下一步不应直接内置 ERBM 自动合并，而应先实现“作者/整合包 profile 原样运行”：

- 选择一个外部 `.me3` 作为方案事实来源；
- 支持 `path` 与 `source`，并保留未知字段；
- 默认不重写作者 profile；
- 展开并校验相对路径、大小写、缺失文件和越界引用；
- 展示真实 packages、natives、`load_after` / `load_before`；
- 对每个方案显式选择 SeamlessCoop、Server Redirector 或无网络组件；
- 禁止隐式重复注入 `nrsc.dll`；
- 将包内目录标记为 `runtime`、`source`、`tool`、`patch-output`，只启动 runtime；
- 为最终运行文件生成哈希清单，供联机双方比较。

参数级自动合并应作为更晚的独立实验，必须具备：

- 基准游戏/regulation 版本检查；
- 参数行冲突报告和人工优先级；
- 原件、输出和回滚副本；
- 资源级冲突白名单；
- 单人、双人/多人、掉落、敌人、Boss、模型和存档回归；
- 上游更新后的可重复构建记录。

## 十、下一轮实验

建议按风险递增：

1. 用 5.30 包做离线静态解析测试，确保管理器能完整呈现其 profile，不启动游戏；
2. 实现外部 profile 的只读健康报告；
3. 获取当前 NightreignPLUS 版本的变更记录和文件清单，只比较结构与哈希；
4. 若权限允许，再在独立副本中复现 `More Map and Weapons + SkinOverhaul`
   的参数合并；
5. 最后才做当前游戏版本的真实启动和联机回归。

内容定位也应相应修正：可以做“拆解一个成功超级整合包是怎么合并的”，而不是
简单宣称“所有 Mod 天然兼容”或“管理器自动解决了冲突”。
