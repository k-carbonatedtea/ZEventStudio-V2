from PIL import Image
import numpy as np

im1 = Image.open(r"C:\Users\Administrator\.gemini\antigravity-ide\brain\367438a3-844b-41c8-8ce0-d1a938a7c90d\.user_uploaded\media_1792865902847.jpg").convert("RGB")
im2 = Image.open(r"C:\Users\Administrator\.gemini\antigravity-ide\brain\367438a3-844b-41c8-8ce0-d1a938a7c90d\.user_uploaded\media_1792865911361.jpg").convert("RGB")

# 寻找蜥蜴图标 (黑色像素几乎为 0, 在火山口周围 x: 650..800, y: 150..300)
# 火蜥蜴图标是一个纯黑色的剪影: R < 20, G < 20, B < 20
arr1 = np.array(im1)[150:300, 650:800]
arr2 = np.array(im2)[150:300, 650:800]

black1 = (arr1[:, :, 0] < 25) & (arr1[:, :, 1] < 25) & (arr1[:, :, 2] < 25)
black2 = (arr2[:, :, 0] < 25) & (arr2[:, :, 1] < 25) & (arr2[:, :, 2] < 25)

y1, x1 = np.where(black1)
y2, x2 = np.where(black2)

print("im1 lizard pos:", x1.mean() + 650, y1.mean() + 150)
print("im2 lizard pos:", x2.mean() + 650, y2.mean() + 150)
