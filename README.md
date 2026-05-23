# Elden Ring Nightreign Mod Manager

面向《艾尔登法环：黑夜君临 / Elden Ring: Nightreign》的 Windows 桌面 Mod 管理器。项目使用 Tauri v2、React、TypeScript 和 Rust 构建，目标是把 Nightreign + ME3(Mod Engine 3) + SeamlessCoop/Spacewar 环境下的 Mod 管理流程做成可视化工具。

> 本项目为非官方工具，与 FromSoftware、Bandai Namco、Steam、Mod Engine 或 SeamlessCoop 团队无隶属关系。使用 Mod 前请自行备份存档并确认 Mod 来源可信。

## 下载

第一版发布包见 GitHub Releases：

- <https://github.com/JWLuo0719/EldenRing-Nightreign-ModManager/releases/tag/v0.1.0>

推荐下载：

- `nightreign-mod-manager_0.1.0_x64-setup.exe`

也可以使用 MSI：

- `nightreign-mod-manager_0.1.0_x64_en-US.msi`

## 主要功能

- 配置游戏目录、ME3 目录和启动程序。
- 扫描 `Game\mods` 下的本地 Mod 文件夹和根目录 DLL。
- 安装 ZIP Mod。
- 添加外部 Mod 目录和外部 DLL。
- 编辑已识别的 JSON/INI Mod 配置文件，例如 `nighter.json`。
- 管理配置方案、启用状态和加载顺序基础数据。
- 生成 ME3 profile 并通过 ME3 启动 Nightreign。
- 诊断启动脚本、ME3 profile、启动日志和文件冲突。
- 删除本地 Mod 时移动到系统回收站；外部 Mod 只移除注册记录。

## 已验证链路

当前已验证可用的启动链路为 SeamlessCoop/Spacewar 环境：

```text
管理器启动
-> 生成 active-nightreign.me3
-> 生成 launch-nightreign.bat
-> me3.exe launch --exe nightreign.exe --skip-steam-init --online --game nightreign -p active-nightreign.me3
-> ME3 注入并加载 packages/natives
```

关键约定：

- ME3 入口使用 `me3.exe`，通常位于 `{ME3目录}\bin\me3.exe`。
- 不把 `nrsc_launcher.exe` 作为 ME3 的 `--exe` 传入；如果用户配置了它，管理器会在 ME3 链路中改用同目录的 `nightreign.exe`。
- `SeamlessCoop\nrsc.dll` 会作为 early native 加载。
- `Game\mods\nighter.dll` 会作为普通 native 加载。

## 首次使用

1. 安装并启动管理器。
2. 在设置向导中选择游戏目录，目录内应包含 `nightreign.exe`。
3. 选择 ME3 目录，目录内应包含 `me3.exe` 或 `bin\me3.exe`。
4. 将普通资源包 Mod 放入游戏目录的 `Game\mods` 下，或在 Mod 仓库页面添加外部 Mod/DLL。
5. 按需启用 Mod，打开诊断页检查生成的 ME3 profile。
6. 回到启动台启动游戏。

## 注意事项

- 启动前请确认没有残留的 `nightreign.exe` 或 `me3-launcher.exe` 进程。
- 联机补丁不会被自动查找或自动覆盖；只有用户手动选择补丁 `Game` 文件夹并确认后，才会复制补丁文件。
- 某些 Mod 自带 `.me3` 可能引用不存在的文件。管理器会跳过不存在的 package/native，但如果游戏仍卡住，应优先排查 Mod 与当前游戏版本、深夜解锁或联机环境的兼容性。
- 当前版本仍是第一版，建议每次更换 Mod 组合后先查看诊断页输出。

## 本地开发

环境：

- Node.js
- Rust
- Tauri v2 依赖

常用命令：

```powershell
cd app
npm install
npm run dev
npx tauri dev
npm run build
npm run lint
```

Rust 检查：

```powershell
cd app/src-tauri
cargo check
cargo test
```

打包：

```powershell
cd app
npx tauri build
```

## 仓库约定

- `me3/`、`mods/`、`reference/` 为本地工具、测试 Mod 和参考文件目录，不应提交。
- 不要把个人游戏路径、ME3 路径或补丁路径写入源码默认配置。
- 配置文件存放在系统配置目录的 `nightreign-mod-manager` 下。

## 许可证

当前尚未声明开源许可证。发布前如需开放给他人复用代码，请先补充明确许可证文件。
