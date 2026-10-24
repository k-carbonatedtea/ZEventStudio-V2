from PIL import Image
import numpy as np

im1 = Image.open(r"C:\Users\Administrator\.gemini\antigravity-ide\brain\367438a3-844b-41c8-8ce0-d1a938a7c90d\.user_uploaded\media_1792865902847.jpg").convert("RGB")
im2 = Image.open(r"C:\Users\Administrator\.gemini\antigravity-ide\brain\367438a3-844b-41c8-8ce0-d1a938a7c90d\.user_uploaded\media_1792865911361.jpg").convert("RGB")

# 寻找风神兽 (在左侧 x: 150..300, y: 200..400)
# 寻找水神兽 (在右侧 x: 750..950, y: 400..600)
def find_beast_card(im, x_range, y_range):
    arr = np.array(im)[y_range[0]:y_range[1], x_range[0]:x_range[1]]
    # 深蓝卡片背景 R < 20, G < 40, B > 60 或者青色边框
    # 让我们找青色文字或边框: R < 100, G > 180, B > 230
    mask = (arr[:, :, 0] < 100) & (arr[:, :, 1] > 180) & (arr[:, :, 2] > 230)
    y, x = np.where(mask)
    if len(x) == 0:
        return None
    return (x.mean() + x_range[0], y.mean() + y_range[0])

m1 = find_beast_card(im1, (180, 320), (250, 400)) # 梅德
r1 = find_beast_card(im1, (780, 920), (450, 600)) # 露塔

m2 = find_beast_card(im2, (150, 320), (250, 420))
r2 = find_beast_card(im2, (780, 950), (450, 650))

print("im1 Medoh:", m1, "Ruta:", r1)
print("im2 Medoh:", m2, "Ruta:", r2)

if m1 and r1 and m2 and r2:
    dist1 = np.sqrt((m1[0] - r1[0])**2 + (m1[1] - r1[1])**2)
    dist2 = np.sqrt((m2[0] - r2[0])**2 + (m2[1] - r2[1])**2)
    print(f"im1 distance: {dist1:.2f}")
    print(f"im2 distance: {dist2:.2f}")
    print(f"Marker scale ratio: {dist2 / dist1:.4f}")
