# 602 × Skin Overhaul 服装中文兼容补丁

这是为本项目当前 Skin Overhaul 服装包制作的本地兼容中文层。

- 以 602 2.1.7.1（2026-07-22）的 `item_dlc01.msgbnd.dcx` 为底稿；
- 保留 602 的完整 `menu_dlc01.msgbnd.dcx`，不覆盖其 MMV/Weapons 文本；
- 仅在 `GoodsName.fmg` 合入 Skin Overhaul 新增的 76 个服装名称；
- 原始 602 和 Skin Overhaul 文件均不修改。

## 使用规则

把本补丁当作完整的“中文资源层”使用：启用它时，不要同时启用原始 602、559 或任何其它
包含完整 `msg\zhocn\item_dlc01.msgbnd.dcx` / `menu_dlc01.msgbnd.dcx` 的中文包。它应与
Skin Overhaul 一起启用。

本补丁只解决中文名称，不会合并 Skin Overhaul 与 MMV/Weapons 的
`regulation.bin`。这两份玩法数据仍需要单独的参数级合并，不能依赖加载顺序。

## 构建可追溯性

- 名称映射：`tools/skin-overhaul-602-zhocn-names.json`
- 构建脚本：`tools/build-skin-overhaul-602-zhocn-item.ps1`
- 产物 `item_dlc01.msgbnd.dcx`：177,920 bytes，SHA-256
  `E19B438301CCECBB91C6EB2A02F66FDE6518285A08B1DB03F17A8D850127112A`
- 回读验证：76 / 76 个 Skin Overhaul `GoodsName` ID 已写入预期名称。

其中已有官方中文名称的角色和装备沿用现有官方译名；联动或作者自定义外观使用本项目
补译。若 Skin Overhaul 后续新增、删除或调整 ID，必须重新从其英文 `GoodsName.fmg`
审计后再更新映射，不能直接沿用本补丁。
