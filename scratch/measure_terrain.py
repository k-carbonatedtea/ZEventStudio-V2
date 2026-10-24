from PIL import Image
import numpy as np

im1 = Image.open(r"C:\Users\Administrator\.gemini\antigravity-ide\brain\367438a3-844b-41c8-8ce0-d1a938a7c90d\.user_uploaded\media_1792865902847.jpg").convert("RGB")
im2 = Image.open(r"C:\Users\Administrator\.gemini\antigravity-ide\brain\367438a3-844b-41c8-8ce0-d1a938a7c90d\.user_uploaded\media_1792865911361.jpg").convert("RGB")

# 我们来看利特村那个圆形的岛屿在 im1 和 im2 的位置
# 利特村在一个独特的湖中岛上 (x: 200..300, y: 300..450)
# 还有火山口 (x: 650..800, y: 150..300)
# 让我们量一下火山口岩浆环与利特村湖岛之间的距离！
