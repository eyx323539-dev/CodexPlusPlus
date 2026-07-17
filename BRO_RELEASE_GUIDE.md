# BRO Codex++ 发布与更新手册

这份文档记录当前 BRO API 定制版改动、首次发布状态，以及以后更新版本时的标准流程。

## 当前发布信息

- GitHub 仓库：https://github.com/eyx323539-dev/CodexPlusPlus
- 当前 Release：`v1.2.38-bro.1`
- 当前安装包：`CodexPlusPlus-1.2.38-bro.1-windows-x64-setup.exe`
- 更新源：
  `https://github.com/eyx323539-dev/CodexPlusPlus/releases/latest/download/latest.json`

用户只有安装 `v1.2.38-bro.1` 或之后版本，软件内“检查更新”才会走 BRO 自己的 GitHub Release。旧版安装包仍可能指向原作者更新源。

## 已完成的 BRO 定制内容

- 首页广告卡片改为 `BRO API 中转站`
- 首页广告按钮链接改为 `https://api.skuzi.cn/`
- 首页广告小图标改为 BRO logo
- 供应商预设里的 JOJO Code 改为 `BRO API 中转站`
- core 内置广告改为 BRO API
- 更新检查源改为 BRO 自己的 GitHub 仓库
- 安装包 Publisher 改为 `BRO API`
- GitHub 仓库已创建并推送源码
- Release 已上传安装包和 `latest.json`

## 每次更新版本的标准流程

以下命令默认在项目根目录执行：

```powershell
cd C:\Users\32529\Desktop\codexplusplus
```

### 1. 修改版本号

建议版本格式继续用：

```text
v1.2.38-bro.2
v1.2.38-bro.3
v1.2.39-bro.1
```

如果只改广告、文案、小修复，可以递增 `bro.2`、`bro.3`。

如果上游 CodexPlusPlus 大版本升级，再改成 `1.2.39-bro.1` 这种。

### 2. 修改代码或内容

常见修改位置：

- 首页广告：`apps/codex-plus-manager/src/App.tsx`
- 首页广告样式：`apps/codex-plus-manager/src/styles.css`
- BRO logo：`apps/codex-plus-manager/src/bro-api-logo.png`
- 供应商预设：`apps/codex-plus-manager/src/presets.ts`
- 内置广告：`crates/codex-plus-core/src/ads.rs`
- 更新源：`crates/codex-plus-core/src/update.rs`
- Windows 安装包配置：`scripts/installer/windows/CodexPlusPlus.nsi`

注意：不要用 PowerShell 直接整文件重写中文源码，容易把 UTF-8 中文弄坏。小改动优先用补丁或编辑器。

### 3. 构建前端

```powershell
cd C:\Users\32529\Desktop\codexplusplus\apps\codex-plus-manager
npm run vite:build
```

成功时会看到类似：

```text
✓ built in ...
```

### 4. 编译 release

```powershell
cd C:\Users\32529\Desktop\codexplusplus
cargo build --release
```

成功时会看到：

```text
Finished `release` profile [optimized]
```

如果只有 `CUBENCE_IMAGE`、`ERGOU_API_IMAGE` 未使用警告，可以忽略。

### 5. 复制 exe 到安装包目录

```powershell
New-Item -ItemType Directory -Force dist\windows\app | Out-Null
Copy-Item target\release\codex-plus-plus.exe dist\windows\app\ -Force
Copy-Item target\release\codex-plus-plus-manager.exe dist\windows\app\ -Force
```

### 6. 生成 Windows 安装包

把 `$version` 改成这次要发布的版本，不带开头的 `v`：

```powershell
$version = "1.2.38-bro.2"
& "C:\Program Files (x86)\NSIS\makensis.exe" "/DVERSION=$version" scripts\installer\windows\CodexPlusPlus.nsi
```

输出文件会在：

```text
dist\windows\CodexPlusPlus-1.2.38-bro.2-windows-x64-setup.exe
```

如果提示 `Can't open output file`，通常是旧安装包正在被占用。关闭资源管理器预览、安装程序窗口，或换一个新版本号再打包。

### 7. 本地提交并推送 GitHub

```powershell
git status
git add .
git commit -m "Release v1.2.38-bro.2"
git push
```

不要提交这些目录：

- `target/`
- `dist/`
- `node_modules/`
- `apps/codex-plus-manager/dist/`

它们已在 `.gitignore` 里，正常不会被提交。

### 8. 创建 GitHub Release

把版本号和安装包文件名改成这次发布的：

```powershell
$tag = "v1.2.38-bro.2"
$asset = "dist\windows\CodexPlusPlus-1.2.38-bro.2-windows-x64-setup.exe"

& "C:\Program Files\GitHub CLI\gh.exe" release create $tag $asset `
  --repo eyx323539-dev/CodexPlusPlus `
  --title $tag `
  --notes "BRO API Codex++ update."
```

### 9. 生成并上传 latest.json

这是最关键的一步。没有 `latest.json`，软件里的“检查更新”就不能顺利发现新版本。

```powershell
$tag = "v1.2.38-bro.2"
$repo = "eyx323539-dev/CodexPlusPlus"
$assetName = "CodexPlusPlus-1.2.38-bro.2-windows-x64-setup.exe"

$payload = [ordered]@{
  version = $tag
  tag_name = $tag
  url = "https://github.com/$repo/releases/tag/$tag"
  body = "BRO API Codex++ update."
  assets = @(
    [ordered]@{
      name = $assetName
      browser_download_url = "https://github.com/$repo/releases/download/$tag/$assetName"
    }
  )
}

$payload | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath latest.json -Encoding UTF8

& "C:\Program Files\GitHub CLI\gh.exe" release upload $tag latest.json --clobber --repo $repo
```

### 10. 验证更新源

```powershell
curl.exe -L https://github.com/eyx323539-dev/CodexPlusPlus/releases/latest/download/latest.json
```

确认输出里有：

- 新版本号
- 新安装包文件名
- 正确的 `browser_download_url`

也可以查看 Release：

```powershell
& "C:\Program Files\GitHub CLI\gh.exe" release view v1.2.38-bro.2 --repo eyx323539-dev/CodexPlusPlus --json tagName,url,assets
```

## 用户侧更新逻辑

软件内“检查更新”会读取：

```text
https://github.com/eyx323539-dev/CodexPlusPlus/releases/latest/download/latest.json
```

然后比较：

- 当前安装的软件版本
- `latest.json` 里的 `version`

如果 `latest.json` 的版本更新，就会下载 `assets[0].browser_download_url` 指向的安装包，并启动安装。

## 重要注意事项

- 新版必须创建 GitHub Release，单纯 `git push` 不会让用户更新。
- Release 里必须上传 Windows 安装包。
- Release 里必须上传 `latest.json`。
- `latest.json` 的安装包文件名必须和 Release asset 完全一致。
- 用户第一次必须安装已经切换更新源的新包，例如 `v1.2.38-bro.1` 或之后版本。
- 如果改了更新源代码，必须重新打安装包，用户安装后才生效。
- 不建议删除旧 Release，否则老用户可能无法下载历史版本。
- 如果 GitHub Release 页面显示安装包上传成功，但软件检查不到，优先检查 `latest.json`。

## 快速发布命令模板

把版本号替换成新版本即可：

```powershell
cd C:\Users\32529\Desktop\codexplusplus

$version = "1.2.38-bro.2"
$tag = "v$version"
$repo = "eyx323539-dev/CodexPlusPlus"
$assetName = "CodexPlusPlus-$version-windows-x64-setup.exe"
$asset = "dist\windows\$assetName"

cd apps\codex-plus-manager
npm run vite:build

cd C:\Users\32529\Desktop\codexplusplus
cargo build --release

New-Item -ItemType Directory -Force dist\windows\app | Out-Null
Copy-Item target\release\codex-plus-plus.exe dist\windows\app\ -Force
Copy-Item target\release\codex-plus-plus-manager.exe dist\windows\app\ -Force

& "C:\Program Files (x86)\NSIS\makensis.exe" "/DVERSION=$version" scripts\installer\windows\CodexPlusPlus.nsi

git add .
git commit -m "Release $tag"
git push

& "C:\Program Files\GitHub CLI\gh.exe" release create $tag $asset --repo $repo --title $tag --notes "BRO API Codex++ update."

$payload = [ordered]@{
  version = $tag
  tag_name = $tag
  url = "https://github.com/$repo/releases/tag/$tag"
  body = "BRO API Codex++ update."
  assets = @(
    [ordered]@{
      name = $assetName
      browser_download_url = "https://github.com/$repo/releases/download/$tag/$assetName"
    }
  )
}

$payload | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath latest.json -Encoding UTF8
& "C:\Program Files\GitHub CLI\gh.exe" release upload $tag latest.json --clobber --repo $repo

curl.exe -L "https://github.com/$repo/releases/latest/download/latest.json"
```

## 如果以后要同步上游

现在本仓库是从源码 zip 初始化的，不是标准 fork。以后如果要同步原作者新版本，推荐流程：

1. 单独下载原作者最新版源码
2. 对比当前 BRO 改动
3. 重新套用 BRO 广告、logo、更新源、供应商预设
4. 重新测试和发布

最容易漏的 BRO 改动是：

- `crates/codex-plus-core/src/update.rs`
- `apps/codex-plus-manager/src/App.tsx`
- `apps/codex-plus-manager/src/presets.ts`
- `crates/codex-plus-core/src/ads.rs`
- `docs/images/sponsor-bro-api.png`
- `apps/codex-plus-manager/src/bro-api-logo.png`

