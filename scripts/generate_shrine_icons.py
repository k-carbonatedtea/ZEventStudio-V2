from PIL import Image, ImageFilter, ImageEnhance
import os

src_path = r"C:\Users\Administrator\Desktop\Zevent\ice-spear\assets\img\map\shrine.png"
out_dir = r"C:\Users\Administrator\Desktop\Zevent\ZEventStudio\assets\map"
os.makedirs(out_dir, exist_ok=True)

im = Image.open(src_path).convert("RGBA")
w, h = im.size

# 复制原版
im.save(os.path.join(out_dir, "shrine_raw.png"))

# 1. 希卡神庙原版青蓝色已激活状态 (Sheikah Cyan / Blue)
# 官方游戏中为亮青蓝色 (0, 210, 255) / (56, 189, 248)
blue_im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
blue_pixels = blue_im.load()
src_pixels = im.load()

for y in range(h):
    for x in range(w):
        r, g, b, a = src_pixels[x, y]
        if a == 0:
            continue
        # 计算灰度明度
        luma = (r * 0.299 + g * 0.587 + b * 0.114) / 255.0
        if luma > 0.05:
            # 白色与亮部着亮青蓝 (0, 215, 255)
            # 根据明度保持渐变与抗锯齿
            nr = int(20 + 20 * luma)
            ng = int(180 * luma + 30)
            nb = int(250 * luma)
            blue_pixels[x, y] = (min(255, nr), min(255, ng), min(255, nb), a)
        else:
            # 黑色线条部分变为深海拉鲁暗蓝 (10, 25, 45)
            blue_pixels[x, y] = (10, 25, 45, int(a * 0.85))

blue_im.save(os.path.join(out_dir, "shrine_blue.png"))

# 2. 悬浮/选中高亮状态 (亮白炽微蓝 + 外发光)
glow_im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
glow_pixels = glow_im.load()
for y in range(h):
    for x in range(w):
        r, g, b, a = src_pixels[x, y]
        if a == 0:
            continue
        luma = (r * 0.299 + g * 0.587 + b * 0.114) / 255.0
        if luma > 0.05:
            nr = int(120 + 135 * luma)
            ng = int(220 + 35 * luma)
            nb = 255
            glow_pixels[x, y] = (min(255, nr), min(255, ng), nb, a)
        else:
            glow_pixels[x, y] = (20, 50, 80, int(a * 0.9))

glow_im.save(os.path.join(out_dir, "shrine_active.png"))
print("Shrine icons generated successfully!")
