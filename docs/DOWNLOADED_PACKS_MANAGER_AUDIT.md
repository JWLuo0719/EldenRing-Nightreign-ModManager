# 两个已下载整合包的管理器适配审计

审计日期：2026-07-30

> 状态更新：本文件记录的是实施前审计。MMV 的作者 Profile 字段丢失和
> Seamless/Server Redirector 冲突已在同日完成管理器适配；最新行为见
> `docs/MMV_MANAGER_COMPATIBILITY.md`。SkinOverhaul 污染与两包内容冲突结论
> 仍然有效。

审计对象：

```text
D:\Game\ELDEN RING NIGHTREIGN\mods\More Map Variations 2.1.7-hotfix1 & Weapons Mod
D:\Game\ELDEN RING NIGHTREIGN\mods\SkinOverhaul
```

本轮只读取目录、配置、脚本、文件元数据和 SHA-256，没有运行任何 EXE、DLL、
BAT 或 ME3 profile，也没有修改游戏目录。

## 一、结论

| 对象 | 管理器扫描 | 当前管理器启动 | 当前版本状态 | 作为公开附件 |
| --- | --- | --- | --- | --- |
| MMV 2.1.7 Hotfix 1 + Weapons | 能识别核心 package/native | 不安全，阻止推荐 | 文件版本正确，但启动语义会丢失 | 不允许 |
| 当前 `SkinOverhaul` 目录 | 可推断为 package | 不应启动 | 不是干净独立版，含旧合并 regulation | 不允许 |
| 两者同时启用 | 都能显示 | 不兼容 | 三个关键文件真实冲突 | 不允许 |

“管理器能显示 Mod”不等于“能按作者要求安全启动”。这两个目录目前都不适合
随管理器作为附件分享。

## 二、MMV + Weapons 目录

### 文件概况

```text
文件：1,440
目录：38
大小：约 2.552 GiB
regulation.bin：1
.me3：1
DLL：1
EXE：1
```

profile：

```toml
profileVersion = "v1"
savefile = "NR0000.co2"
start_online = true

[[supports]]
game = "nightrein"

[[packages]]
path = "mod"

[[natives]]
path = "mod/Server Redirector/cl_server_redirector.dll"
load_early = true
```

关键 SHA-256：

```text
mod\regulation.bin
d36b9960e19c748112f2a8d0d4c00d33a2bec8ae9bb1707975516c3dbb64f579

mod\Server Redirector\cl_server_redirector.dll
9413c4e6c4cc6d958e4b3dd4756548168749622ad2e0d5a4bd9ad3295ff2350c
```

该目录符合 2.1.7 Hotfix 1 + Weapons 的规模和结构，包含 Server Redirector。
当前只含 `engus` 和 `jpnjp` 文本，没有 `msg\zhocn`，所以它还不是中文方案。

### 当前管理器能处理的部分

- 作为外部目录注册时，可以找到顶层 `.me3`；
- package 使用 `path = "mod"`，当前解析器可以读取；
- Server Redirector DLL 使用 `path`，当前解析器可以读取；
- `load_early = true` 可以保留；
- 管理器重新生成的 profile 会写回正确的 `game = "nightreign"`。

### 阻断项

1. 当前 profile 生成器不会保留 `savefile = "NR0000.co2"`；
2. 不会保留 `start_online = true`；
3. 会丢弃作者 profile 中未显式支持的其他字段和未来扩展；
4. 当前游戏根目录存在 `SeamlessCoop\nrsc.dll`，管理器会自动注入它；
5. 结果会同时加载 SeamlessCoop 与 Server Redirector，而后者 ReadMe 明确写着
   不可与 Seamless 一起使用；
6. 失去独立存档声明后，存在误用普通存档的风险；
7. 作者要求文件位于实际 Steam 游戏目录之外，不建议通过当前 ZIP 安装器解压
   到 `Game\mods`；
8. profile 的 `game = "nightrein"` 与项目已验证值 `nightreign` 拼写不同，
   直接运行作者 profile 前需要实际验证，不能静默修正后假设等价。

因此，当前状态属于：

```text
可解析 ≠ 可安全启动
```

### 适配所需最小改动

- 增加“作者 profile 原样运行”模式；
- 方案级网络组件选择：`server-redirector` 时禁止注入 `nrsc.dll`；
- 保留 `savefile`、`start_online` 和未知字段；
- 外部目录只读注册，不移动、不重命名、不删除；
- 启动前显示实际 native 列表和存档扩展名；
- 对 `game` 拼写、相对路径和缺失文件做健康检查；
- 将中文 602 作为单独覆盖层，并记录版本和哈希。

在这些能力完成前，MMV 只能由作者 `.me3` 独立启动，不能宣传为当前管理器
支持的一键方案。

## 三、当前 SkinOverhaul 目录

### 文件概况

```text
文件：470
目录：6
大小：约 3.038 GiB
regulation.bin：1
.me3：0
DLL：0
BAT：1
```

缺少作者安装说明中应与 Skin 目录同级使用的：

```text
Skin Overhaul Nightreign.me3
Backup Launcher.bat
SeamlessCoop\
```

存在 `parts\01_Online.bat`，但当前未生成 `_l` 在线镜像文件。该脚本会复制并
重命名大量皮肤文件，使在线环境能识别远端外观；管理器当前不会运行或验证它。

### 关键问题：不是干净独立版

当前文件：

```text
regulation.bin
SHA-256 b06275a641caa50e08e235e7f5d2c8ba545fde7be5d70afdb693d2920d6b51fd
```

该哈希与此前 NightreignPLUS 5.30 审计中，ERBM 在 2026-05-30 合并
`More Map and Weapons + SkinOverhaul` 后的输出完全一致。

这说明当前目录的 `regulation.bin` 不是可独立判断的原始 Skin Overhaul
regulation，而是旧 MMV + Skin 兼容分支的合并产物。单独加载时可能引用并未
随目录提供的旧 MMV 参数/资源；与当前 MMV 2.1.7.1 一起加载时又会覆盖其新版
参数。

此外，Skin Overhaul 官方 1.9.0 主文件标注：

```text
Game 1.03.2
Calibration 1.03.4
```

当前官方 Regulation 为 1.03.5，原版也需要重新验证；这个被二次合并的目录
风险更高。

### 管理器行为

当前管理器会因为存在 `parts`、`material` 和 `regulation.bin`，将它推断为
package。这一步能成功，但会掩盖三个问题：

- regulation 来源不干净；
- 在线准备脚本尚未执行；
- 作者 profile 和存档/网络环境缺失。

因此必须在健康检查中区分：

```text
目录可识别
目录可加载
版本可兼容
方案已验证
```

当前目录只能达到第一项。

## 四、两个目录同时启用

相同相对路径共有 3 个，内容全部不同：

```text
material/allmaterial.matbinbnd.dcx
msg/engus/item_dlc01.msgbnd.dcx
regulation.bin
```

加载 Skin 在后：

- 旧合并 `regulation.bin` 覆盖 MMV 2.1.7 Hotfix 1；
- Skin 材质和英文文本覆盖 MMV；
- 可能丢失 2.1.7.1 修复和新增参数。

加载 MMV 在后：

- MMV regulation 覆盖 Skin 新皮肤/武器参数；
- Skin 内容可能出现缺模型、缺条目或不可选；
- MMV 材质和英文文本覆盖 Skin。

因此加载顺序无法修复该组合。若未来要合并，必须基于当前 MMV
`d36b9960...` 和一份干净、当前版本的 Skin regulation 重新做参数合并，并
人工处理材质/文本冲突。

## 五、再分发结论

公开页面权限：

- More Map Variations：禁止上传到其他站点；
- MoreWeapons：禁止上传到其他站点；
- Skin Overhaul：禁止上传到其他站点；
- MMV 简中 602：允许署名转载。

所以即使这两个目录在本机能够运行，也不能直接作为管理器、视频或网盘的附件
重新发布，除非取得所有相关作者的明确书面授权。下载自“其他整合作者”不能替代
上游作者授权。

可发布的替代形态：

```text
管理器安装包
整合方案清单/README
原作者页面链接
要求版本、哈希和安装位置
管理器生成的 profile/诊断规则
经许可的 602 中文翻译（必须署名）
```

不要包含 MMV、MoreWeapons、Skin Overhaul 的实际资产文件。

## 六、建议决策

### 当前可以做

1. 将 MMV 目录保留为外部只读测试样本；
2. 不启用当前 SkinOverhaul；
3. 不把两者同时交给当前管理器启动；
4. 先实现外部作者 profile 原样运行和方案级网络组件；
5. 公开分享时只发配方、来源链接与健康检查；
6. 首个真正可随管理器附件分发的示范仍优先使用已核对允许转载的
   Duchess Unmasked，或自制/明确授权资源。

### 完成适配后的 MMV 验证顺序

1. 不加载游戏根目录 Seamless；
2. 保留 `NR0000.co2` 独立存档；
3. 只加载 MMV package + Server Redirector；
4. 验证单人、匹配、掉落、退出和重进；
5. 再覆盖 602 中文；
6. 双方比较版本、profile、DLL、regulation 和翻译哈希。

### Skin 后续

1. 重新取得干净的官方 Skin Overhaul 1.9.0；
2. 校验原档、profile 和 regulation；
3. 在副本上运行并核对 `01_Online.bat` 输出；
4. 先单独 Seamless 回归；
5. 不与 MMV 合并，除非重新制作当前版本兼容补丁并取得分发授权。
