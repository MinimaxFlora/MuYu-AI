# 发布加固：日志、开机自启、签名与更新

版本：0.9.3
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

## 2. 代码签名

当前安装包**未签名**。Windows 上会触发 SmartScreen “未知发布者”。

正式签名需要：

1. 购买 Standard/EV Code Signing 证书（PFX 或硬件令牌）。
2. 在 `tauri.conf.json` 的 `bundle.windows` 增加：
   - `certificateThumbprint`
   - `digestAlgorithm: "sha256"`
   - `timestampUrl`
3. CI 中注入证书或使用云签名服务（Azure Trusted Signing / DigiCert）。
4. 对 `MuYu.exe` 与 `MuYu_0.9.3_x64-setup.exe` 都签名。

没有证书无法完成可信签名；不能用自签名冒充正式发布。

## 3. 自动更新

Tauri updater 需要三样东西：

1. 更新签名密钥对（`tauri signer generate`）。
2. 公钥写入 `tauri.conf.json` 的 updater pubkey。
3. 可访问的更新 endpoint（静态 JSON + 安装包）。

当前没有更新服务器与签名密钥，**未开启 updater**，避免出现“检查更新必失败”的假功能。

后续建议：

- endpoint：`https://update.muyu.studio/latest.json`
- 产物：NSIS 安装包 + `.sig`
- 渠道：stable / beta
- 应用内“检查更新”与托盘提醒

## 4. 崩溃报告

目前只有本地日志，没有远程上报。

后续可加：

- 崩溃转储（Windows WER / crashpad）
- 匿名错误上报（Sentry 自托管或 GlitchTip）
- 用户可选开关与隐私说明

## 5. 剩余发布级缺口

- 代码签名证书。
- 更新服务器与签名密钥。
- 日志查看器（目前只打开目录）。
- 崩溃转储与远程上报。
- 安装包未做企业静默部署文档。

这些都需要外部账号/证书/服务器，不能在本地代码里“假完成”。
