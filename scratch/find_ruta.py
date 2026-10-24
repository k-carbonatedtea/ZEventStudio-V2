from PIL import Image
import numpy as np

im1 = Image.open(r"C:\Users\Administrator\.gemini\antigravity-ide\brain\367438a3-844b-41c8-8ce0-d1a938a7c90d\.user_uploaded\media_1792865902847.jpg").convert("RGB")
im2 = Image.open(r"C:\Users\Administrator\.gemini\antigravity-ide\brain\367438a3-844b-41c8-8ce0-d1a938a7c90d\.user_uploaded\media_1792865911361.jpg").convert("RGB")

def find_blue_box(im, x_range, y_range):
    arr = np.array(im)[y_range[0]:y_range[1], x_range[0]:x_range[1]]
    # 蓝色背景: B > 60, R < 40
    mask = (arr[:, :, 2] > 70) & (arr[:, :, 0] < 40)
    y, x = np.where(mask)
    if len(x) == 0:
        return None
    return (x.mean() + x_range[0], y.mean() + y_range[0])

# 露塔在右边
r1 = find_blue_box(im1, (800, 950), (500, 650))
r2 = find_blue_box(im2, (800, 950), (500, 650))
print("Ruta 1:", r1)
print("Ruta 2:", r2)
