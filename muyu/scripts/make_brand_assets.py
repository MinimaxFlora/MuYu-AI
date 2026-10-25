from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "brand" / "logo-source.png"
PUBLIC = ROOT / "public"
PUBLIC.mkdir(exist_ok=True)

src = Image.open(SOURCE).convert("RGBA")
# 原图四角/边缘有一圈黑底，先裁到白色圆角方块，再把圆角切透明。
bbox = src.convert("L").point(lambda value: 255 if value > 24 else 0).getbbox()
if bbox:
    src = src.crop(bbox)
# 再向内收 1.2%，去掉白底外沿的黑边。
inset = max(1, int(src.width * 0.025))
src = src.crop((inset, inset, src.width - inset, src.height - inset))
size = max(src.size)
if src.size != (size, size):
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    canvas.alpha_composite(src, ((size - src.width) // 2, (size - src.height) // 2))
    src = canvas

# 新 logo 是白底圆角方块；把四角切透明，任务栏/桌面图标更干净。
corner = src.getpixel((0, 0))
if corner[3] > 8:
    mask = Image.new("L", (size, size), 0)
    radius = int(size * 0.199)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, size - 1, size - 1], radius=radius, fill=255)
    icon = src.copy()
    icon.putalpha(mask)
else:
    icon = src

icon = icon.resize((1024, 1024), Image.LANCZOS)
icon.save(ROOT / "brand" / "app-icon.png")

full = icon.resize((512, 512), Image.LANCZOS)
full.save(PUBLIC / "logo.png")
full.resize((180, 180), Image.LANCZOS).save(PUBLIC / "apple-touch-icon.png")

print("saved", icon.size, full.size)
