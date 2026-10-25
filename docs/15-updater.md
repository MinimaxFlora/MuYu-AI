# GitHub 更新流程（对齐 Clash Verge Rev）

版本：0.9.5
日期：2026-10-25

---

## 1. 参考实现

Clash Verge Rev 的更新链路：

1. GitHub Actions 在 `v*` tag 上构建 Tauri 安装包。
2. 构建时用 `TAURI_SIGNING_PRIVATE_KEY` 给 updater 产物签名。
3. `tauri-action` 的 `includeUpdaterJson: true` 生成/上传 `latest.json`。
4. 应用内 `tauri-plugin-updater` 请求 GitHub Release 的 `latest.json`。
5. 用内置公钥校验安装包签名；Windows 用 NSIS `passive` 安装。
6. 安装完成重启应用。

参考文件：
- `clash-verge-rev/src-tauri/tauri.conf.json`：`plugins.updater.pubkey/endpoints`、`createUpdaterArtifacts`。
- `clash-verge-rev/.github/workflows/release.yml`：tauri-action + signing secrets。
- `clash-verge-rev/src-tauri/src/core/updater.rs`：Tauri updater、下载、签名校验、安装。

## 2. 幕语实现

### 2.1 依赖与配置
- `tauri-plugin-updater = "2"`。
- `bundle.createUpdaterArtifacts: true`。
- `plugins.updater.pubkey`：本机生成的 minisign 公钥。
- `plugins.updater.windows.installMode: "passive"`。
- 私钥：`src-tauri/updater.key`（已加入 `.gitignore`，不要提交）。

### 2.2 检查地址
用户在设置页填写 GitHub 仓库 `MinimaxFlora/MuYu-AI`，应用运行时拼接：

```
https://github.com/<owner>/<repo>/releases/latest/download/latest.json
```

这要求：
- 仓库公开；
- Release 已发布（不是 draft / prerelease）；
- Release 附带 `latest.json` 和安装包 `.sig`。

### 2.3 应用内流程
- 启动 8 秒后自动检查（可在设置里关闭）。
- 发现新版本：右下角提示「发现新版本 vX」，点击进入设置。
- 设置 → 更新：
  - 更新仓库
  - 启动自动检查
  - 检查更新
  - 版本/更新说明
  - 下载并安装 + 进度条
- Windows 下载完成后以 passive 模式启动 NSIS，应用退出并重启。

### 2.4 命令
- `check_update`：读取 `settings.json` 的 `updateRepo`，返回版本/说明/日期。
- `install_update`：重新 check → 下载 → 验签 → 安装；事件 `updater://progress`。

## 3. GitHub Actions

新增 `.github/workflows/release.yml`：

- tag `v*` 触发，校验 tag 与 `muyu/package.json` 版本一致。
- Windows runner 安装 NASM 2.16.03。
- `pnpm install --frozen-lockfile`。
- `tauri-apps/tauri-action`：
  - `projectPath: muyu`
  - `args: --bundles nsis`
  - `includeUpdaterJson: true`
  - `releaseDraft: false`
- Secrets：
  - `TAURI_SIGNING_PRIVATE_KEY`：`updater.key` 文件内容
  - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`：生成密钥时的密码（本机生成时为空）

发布后资产应包含：

```
MuYu_0.9.9_x64-setup.exe
MuYu_0.9.9_x64-setup.exe.sig
latest.json
```

### 3.1 已发布（2026-10-25）

- Release：`https://github.com/MinimaxFlora/MuYu-AI/releases/tag/v0.9.9`
- Actions run `40199655451`：成功。
- 资产：
  - `MuYu_0.9.9_x64-setup.exe` 4,710,274 B
  - `.sig` 432 B
  - `latest.json` 1,383 B
- `latest.json` 已验证包含 `windows-x86_64` 和 `windows-x86_64-nsis`，两项都有签名与 GitHub 下载地址。

## 4. 首次配置

1. 把仓库推到 GitHub，默认分支 `main`。
2. 在 GitHub Actions secrets 添加签名私钥。
3. 在应用设置里填更新仓库 `MinimaxFlora/MuYu-AI`（默认已填）。
4. 打 tag：

```bash
git tag v0.9.9
git push origin v0.9.9
```

5. Actions 完成后，在 Release 页面确认 `latest.json` 已上传。
6. 老版本应用点「检查更新」即可。

### 3.2 真实升级回归

- **0.9.3 → 0.9.4**：检查、下载、验签、NSIS 安装成功；发现 NSIS 语言选择框阻塞 passive 更新。
- **0.9.4 → 0.9.5**：已关闭 `displayLanguageSelector`；检查、下载、安装无弹窗，安装目录 `%LOCALAPPDATA%\MuYu`，版本变为 0.9.5。
- **0.9.5 → 0.9.6**：检查、下载、安装无弹窗；安装后版本 0.9.6；同时完成 API Key 凭据迁移验证。
- **0.9.6 → 0.9.8**：跨版本直接升级成功；安装后 0.9.8；注册表检测修复验证通过。
- **0.9.8 → 0.9.9**：自动检测真实剪映草稿根目录 D:/JianyingPro Drafts；升级成功。

## 5. 限制

- 已完成 0.9.3→0.9.4、0.9.4→0.9.5、0.9.5→0.9.6、0.9.6→0.9.8、0.9.8→0.9.9 五次真实升级；尚未测试失败回滚/断网重试。
- 私钥丢失后无法签发后续更新，必须备份。
- `latest.json` 依赖 GitHub `latest`，预发布版本不会被 `latest` 选中。
- 未接入 Clash 那样的镜像代理与静默后台缓存；当前是手动确认 + 自动检查提醒。
- 代码签名（Authenticode）仍是另一件事：updater 的 minisign 签名只保证更新包完整，不消除 Windows SmartScreen。
