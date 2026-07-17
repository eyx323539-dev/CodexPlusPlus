# Contributing

This is the BRO API customized Codex++ repository.

## Local Development

```powershell
git clone https://github.com/eyx323539-dev/CodexPlusPlus.git
cd CodexPlusPlus
```

Build the manager frontend:

```powershell
cd apps\codex-plus-manager
npm install --package-lock=false
npm run vite:build
```

Build release binaries:

```powershell
cd C:\Users\32529\Desktop\codexplusplus
cargo build --release
```

For publishing and updater details, follow `BRO_RELEASE_GUIDE.md`.

