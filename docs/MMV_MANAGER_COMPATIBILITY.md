# MMV 管理器兼容说明

更新日期：2026-07-31

适用样本：

```text
D:\Game\ELDEN RING NIGHTREIGN\mods\More Map Variations 2.1.7-hotfix1 & Weapons Mod
```

## 已实现行为

- 外部 Mod 顶层 `.me3` 不再只提取 package/native 路径；管理器会保留
  `savefile`、`start_online`、未知根字段，以及 package/native/support 条目中的
  扩展字段。
- 作者 Profile 中的相对路径会在生成副本中解析为绝对路径，原始目录和原始
  `.me3` 保持只读。
- `supports.game = "nightrein"` 会在生成副本中规范为已验证的
  `nightreign`，不会修改作者文件。
- 检测到 `cl_server_redirector.dll` 时，当前方案自动使用
  `server_redirector` 联机后端。
- Server Redirector 方案不再自动注入游戏目录中的
  `SeamlessCoop\nrsc.dll` 或 `nighter.dll`。
- 如果用户另外启用了 `nrsc.dll` 或 `nighter.dll`，Profile 生成会因联机后端
  冲突而失败，阻止启动。
- Server Redirector 作者 Profile 必须位于实际 `Game` 目录之外。
- Server Redirector 启动时不再追加 `--skip-steam-init`，保留它所依赖的正版
  Steam 身份初始化；SeamlessCoop/Spacewar 路线继续使用该参数。
- Server Redirector 与 `OnlineFix / Spacewar` 不能共用同一 `Game` 目录。
  启动前检查和实际启动命令都会阻止这种组合，但不会删除或覆盖现有补丁文件。
- Server Redirector 方案未检测到 Steam 进程时会阻止启动。
- 启动前检查会显示联机后端、作者 Profile、存档文件、在线启动字段，以及
  Seamless/OnlineFix/nighter 在当前 MMV 方案中的实际处理方式。
- 作者 Profile 指定的 `NR0000.co2` 与 Seamless 默认存档同名，并非独立存档。
  管理器会显示警告并在实际启动前备份已有 `.co2/.co2.bak`，但用户仍需避免
  把不同用途的进度混为一体。

## 当前样本验证结果

管理器已对用户下载的真实目录执行只读集成测试，并生成：

```toml
profileVersion = "v1"
savefile = "NR0000.co2"
start_online = true

[[natives]]
load_early = true
path = 'D:\Game\ELDEN RING NIGHTREIGN\mods\More Map Variations 2.1.7-hotfix1 & Weapons Mod\mod\Server Redirector\cl_server_redirector.dll'

[[packages]]
path = 'D:\Game\ELDEN RING NIGHTREIGN\mods\More Map Variations 2.1.7-hotfix1 & Weapons Mod\mod'

[[supports]]
game = "nightreign"
```

验证确认生成结果没有 `nrsc.dll` 或 `nighter.dll`。

## 使用方式

1. 在管理器设置中选择干净的 Steam 正版 `Game` 目录；该目录不能包含
   `OnlineFix64.dll`、`OnlineFix.ini`、`winmm.dll` 等 Spacewar 补丁文件。
2. 保持整合包位于实际 `Game` 目录之外。
3. 在 Mod 仓库选择“添加外部 Mod”，选择整合包最外层目录，不要选择内层
   `mod`。
4. 停用其他单独注册的 `nrsc.dll`、`nighter.dll` 或 Server Redirector。
5. 启动 Steam，并确认正版游戏及 Forsaken Hollows DLC 可用。
6. 在启动台执行“启动前检查”。
7. 确认显示：
   - 方案联机后端：Server Redirector；
   - 作者 Profile：`savefile=NR0000.co2`、`start_online=true`；
   - MMV 外部位置：通过；
   - SeamlessCoop：本方案不注入；
   - OnlineFix / Spacewar：未安装；
   - Steam 状态：已检测。
8. 再通过管理器启动。

## 边界

- 这解决的是 MMV + Weapons 作者整合包的加载、存档和联机启动语义，不会把
  Skin Overhaul 自动合并进 MMV。
- OnlineFix 文件如果仍在 `Game` 目录，启动前检查会报错并阻止启动。MMV
  作者说明和本地 Redirector ReadMe 都要求通过 Steam 定位、身份和邀请，
  因此当前 Spacewar 游戏目录不能作为 MMV 的受支持运行环境。
- 管理器不会运行整合包内的自带 EXE、BAT，也不会修改或重新分发 MMV 资源。
- 2026-07-31 在 Spacewar/OnlineFix 目录中的真实回归已经确认 MMV package、
  `cl_server_redirector.dll` 和 `NR0000.co2` 都成功加载，但游戏服务器登录失败；
  该结果不算 MMV 游戏内验收通过。
- 最终游戏内验收必须在干净的 Steam 正版目录中重新验证地图变化、敌人生成、
  武器获取、人物显示、存档行为和 Server Redirector 联机。
