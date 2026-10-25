# 发布加固：日志、开机自启、签名与更新

版本：0.9.6
日期：2026-10-25

---

## 1. 已实现

### 1.1 本地日志
- 接入 `tauri-plugin-log`。
- 目标：stdout + 应用日志目录。
- 文件：`%LOCALAPPDATA%\com.muyu.studio\logs\muyu.log`。
- 单文件上限 2MB。
- Rust panic hook 写入日志。
- 前端 `window.error` / `unhandledrejection` 写入日志。
- 设置页「启动与日志」→「打开日志目录」。

验证：启动后日志出现 `MuYu started`；关闭进程后文件落盘 52 B。

### 1.2 开机自启
- 接入 `tauri-plugin-autostart`。
- 设置页开关：「开机自启」。
- 使用系统登录项；Windows 写注册表 Run，macOS 用 LaunchAgent。
- capability 增加 `autostart:default`。

### 1.3 托盘与单实例（0.9.2 已完成）
- 托盘菜单：显示主窗口 / 打开项目库 / 设置 / 退出 MuYu。
- 关闭窗口隐藏到托盘。
- 单实例只保留一个进程。

### 1.4 API Key 凭据管理（0.9.6）
- `keyring` + Windows Credential Manager。
- 服务：`com.muyu.studio`；账户：`llm-api-key`。
- `settings.json` 不再写入 `apiKey`。
- 旧版本明文 `apiKey` 在首次 `get_settings` 时自动迁移到凭据管理器，并从文件删除。
- 清空 Key 会删除凭据，不会回退到明文。

验证：
- 旧 `settings.json` 含 `sk-legacy-test` → 读取后文件无 Key，凭据存在。
- 保存新 Key → 文件无 Key，凭据存在。
- 清空 Key → 凭据删除。
- 已安装 0.9.6 上重复验证通过。

## 2. 代码签名

当前安装包未做 **Authenticode** 代码签名；Windows 上会触发 SmartScreen “未知发布者”。updater 的 minisign 签名只保证更新包完整，不等于代码签名。

正式签名需要：

1. 购买 Standard/EV Code Signing 证书（PFX 或硬件令牌）。
2. 在 `tauri.conf.json` 的 `bundle.windows` 增加：
   - `certificateThumbprint`
   - `digestAlgorithm: "sha256"`
   - `timestampUrl`
3. CI 中注入证书或使用云签名服务（Azure Trusted Signing / DigiCert）。
4. 对 `muyu.exe` 与 `MuYu_0.9.5_x64-setup.exe` 都签名。

没有证书无法完成可信签名；不能用自签名冒充正式发布。

## 3. 自动更新

已按 Clash Verge Rev 的方式接入：

1. 本机已生成 minisign 密钥对：`src-tauri/updater.key`（私钥，已 ignore）/ `updater.key.pub`。
2. 公钥写入 `tauri.conf.json` 的 `plugins.updater.pubkey`。
3. GitHub Actions 在 `v*` tag 构建，`createUpdaterArtifacts` + `includeUpdaterJson` 生成 `.sig` 与 `latest.json`。
4. 应用从 `https://github.com/MinimaxFlora/MuYu-AI/releases/latest/download/latest.json` 检查。
5. 设置页可手动检查/下载/安装；启动 8 秒后自动检查并在右下角提示。

详见 `docs/15-updater.md`。

注意：updater 的 minisign 签名只保证更新包完整，**不替代** Windows Authenticode 代码签名。

## 4. 崩溃报告

目前只有本地日志，没有远程上报。

后续可加：

- 崩溃转储（Windows WER / crashpad）
- 匿名错误上报（Sentry 自托管或 GlitchTip）
- 用户可选开关与隐私说明

## 5. 剩余发布级缺口

- 代码签名证书（Authenticode）。
- updater 私钥备份/轮换流程。
- 日志查看器（目前只打开目录）。
- 崩溃转储与远程上报。
- 安装包未做企业静默部署文档。

这些都需要外部账号/证书/服务器，不能在本地代码里“假完成”。
