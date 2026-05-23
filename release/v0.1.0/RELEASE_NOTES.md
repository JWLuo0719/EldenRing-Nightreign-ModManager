# Nightreign Mod Manager v0.1.0

第一版可分发构建。

## 安装包

- `nightreign-mod-manager_0.1.0_x64-setup.exe`：推荐给普通用户使用。
- `nightreign-mod-manager_0.1.0_x64_en-US.msi`：适合需要 MSI 部署的环境。
- `SHA256SUMS.txt`：安装包校验值。

## 主要功能

- 配置游戏目录、ME3 目录和启动程序。
- 扫描 `Game\mods` 下的本地 Mod 和 DLL。
- 安装 ZIP Mod。
- 添加外部 Mod 目录和外部 DLL。
- 编辑已识别的 JSON/INI Mod 配置文件。
- 管理配置方案和启用状态。
- 通过 ME3 启动 Nightreign，并支持已验证的 SeamlessCoop/Spacewar 链路。
- 诊断 ME3 profile、启动脚本、日志和文件冲突。

## 注意事项

- 首次使用必须在设置页选择游戏目录和 ME3 目录。
- 删除本地 Mod 会移动到系统回收站；外部 Mod 只会移除注册记录。
- 启动前请确认没有残留的 `nightreign.exe` 或 `me3-launcher.exe` 进程。
- More Map Variations 等 Mod 如自带 `.me3` 存在坏引用，管理器会跳过不存在的路径；若仍卡住，优先按 Mod 兼容性问题排查。
