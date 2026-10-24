from PIL import Image

map_img = Image.open(r"c:\Users\Administrator\Desktop\Zevent\ZEventStudio\.cache\map_tiles\world_map_full.png")
# 裁剪 (4704, 1400) 附近 600x600 区域
box = (4704 - 300, 1400 - 300, 4704 + 300, 1400 + 300)
crop = map_img.crop(box)
crop.save(r"c:\Users\Administrator\Desktop\Zevent\ZEventStudio\scratch\volcano_check.png")
print("Saved volcano_check.png")
