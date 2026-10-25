# 品牌图标

- `logo-source.png`：用户提供的原始 logo（1254×1254，白底圆角）。
- `app-icon.png`：裁掉黑边、四角透明后的 1024 图标，供 Tauri 生成各平台图标。

重新生成：

```powershell
python scripts/make_brand_assets.py
pnpm tauri icon brand/app-icon.png
```

同源产物：

- `public/logo.png`：512 应用内 logo / favicon
- `public/apple-touch-icon.png`：180
- `src-tauri/icons/*`：Windows .ico、macOS .icns、Linux png、Android/iOS
