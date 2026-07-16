# Nightreign Mod Manager App

本目录是夜临 Mod 管理器的 Tauri v2 应用：React/TypeScript 前端位于 `src/`，Rust 后端位于 `src-tauri/`。

常用命令：

```powershell
npm install
npm run dev
npx tauri dev
npm run lint
npm run build
```

Rust 验证：

```powershell
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check
cargo test
```

完整产品说明、启动链路和安全约定见仓库根目录的 `README.md` 与 `AGENTS.md`。
