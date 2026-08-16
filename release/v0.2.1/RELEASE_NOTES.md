# Nightreign Mod Manager v0.2.1

补丁版构建，日期：2026-08-16。

## 下载文件

- `nightreign-mod-manager_0.2.1_x64-setup.exe`：推荐普通玩家使用的 NSIS 安装包。
- `nightreign-mod-manager_0.2.1_x64_zh-CN.msi`：简体中文 MSI 安装包。
- `SHA256SUMS.txt`：发布文件校验值。

## 本版更新

- MMV 社区兼容模式改为设置页的全局开关，可在安装 MMV 前预先选择；原作者文件继续保持只读。
- 深夜解锁检查改为以本次生成的启动计划为准，正确识别嵌套目录、外部 DLL 与 SeamlessCoop 中的 `nighter.dll`。
- 修复训练场（Boss Arena）目录中 Sandbox/Progression 两份互斥作者 Profile 被叠加的问题。管理器会使用唯一满足单一玩法数据文件门槛的 Sandbox Profile，并在启动前检查中显示实际采用的文件名。

## 验证与边界

- 已通过前端 lint/生产构建、Rust 格式、Clippy、check 与测试。
- NSIS 与 MSI 均按中文安装资源配置构建并完成回读：产品版本均为 `0.2.1`，两者均包含
  `【双击】--更多游戏内容.url`，其内容指向 `https://link3.cc/voyagekit`。
- 训练场作者 README 仅声明 Steam 正版支持；本版仅验证了管理器的启动计划生成，未将 Spacewar + Seamless 表述为作者支持或实际游戏回归。
- 本发布不包含游戏、ME3、SeamlessCoop 或第三方 Mod 文件。
