import os
import sys
import glob
import struct
import subprocess
from concurrent.futures import ThreadPoolExecutor
from PIL import Image
import oead

# 查找可用的 deswizzle 工具
POSSIBLE_EXES = [
    os.path.join(os.path.dirname(__file__), "test_deswizzle.exe"),
    r"C:\Users\Administrator\.gemini\antigravity-ide\brain\47976b52-74e0-432a-9e58-9737d5d9b4ef\scratch\test_deswizzle\target\release\test_deswizzle.exe",
    r"C:\Users\Administrator\.gemini\antigravity-ide\brain\47976b52-74e0-432a-9e58-9737d5d9b4ef\scratch\test_deswizzle\target\debug\test_deswizzle.exe",
]

def get_exe_path():
    for p in POSSIBLE_EXES:
        if os.path.exists(p):
            return p
    return None

def process_tile(src_file, out_dir, exe_path):
    filename = os.path.basename(src_file)
    if not filename.startswith("MapTex1_") or not filename.endswith(".sbmaptex"):
        return
    sec = filename[len("MapTex1_"):-len(".sbmaptex")]
    # 只提取基准瓦片 (排除带有_0, _16384 等状态后缀的文件)
    if "_" in sec:
        return
    
    out_png = os.path.join(out_dir, f"{sec}.png")
    if os.path.exists(out_png) and os.path.getsize(out_png) > 1000:
        return
    
    try:
        data = open(src_file, 'rb').read()
        dec = oead.yaz0.decompress(data)
        
        idx = dec.find(b'FTEX')
        if idx == -1:
            return
        ftex = dec[idx:]
        w, h = 560, 560
        fields = struct.unpack('>IIIIIIIIIIIIIIII', ftex[4:68])
        swizzle = fields[13]
        pitch = fields[15]
        
        swizzled = dec[0x1000 : 0x1000 + 184320]
        tmp_swz = os.path.join(out_dir, f"{sec}.tmp.swz")
        tmp_bc1 = os.path.join(out_dir, f"{sec}.tmp.bc1")
        with open(tmp_swz, 'wb') as f:
            f.write(swizzled)
            
        cmd = [exe_path, tmp_swz, tmp_bc1, str(w), str(h), str(swizzle), str(pitch)]
        subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        
        raw_bc1 = open(tmp_bc1, 'rb').read()
        img = Image.frombytes('RGBA', (w, h), raw_bc1, 'bcn', 1)
        img.save(out_png, 'PNG')
        
        if os.path.exists(tmp_swz): os.remove(tmp_swz)
        if os.path.exists(tmp_bc1): os.remove(tmp_bc1)
    except Exception as e:
        pass

def stitch_full_world_map(out_dir):
    cols = ['Z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K']
    full_path = os.path.join(out_dir, "world_map_full.png")
    full_img = Image.new('RGB', (12 * 560, 10 * 560), (90, 77, 35))
    valid = False
    for col_idx, col in enumerate(cols):
        for r_num in range(10):
            fn = os.path.join(out_dir, f"{col}-{r_num}.png")
            if os.path.exists(fn):
                try:
                    tile = Image.open(fn).convert('RGB')
                    full_img.paste(tile, (col_idx * 560, r_num * 560))
                    valid = True
                except Exception:
                    pass
    if valid:
        full_img.save(full_path, "PNG")
        print(f"Stitched unified world map: {full_path}")

def extract_from_dir(src_dir, out_dir):
    exe_path = get_exe_path()
    if not exe_path:
        print("Error: test_deswizzle executable not found")
        return 0
    os.makedirs(out_dir, exist_ok=True)
    files = glob.glob(os.path.join(src_dir, "MapTex1_*.sbmaptex"))
    with ThreadPoolExecutor(max_workers=8) as ex:
        for f in files:
            ex.submit(process_tile, f, out_dir, exe_path)
    count = len(glob.glob(os.path.join(out_dir, "*.png")))
    if count >= 10:
        stitch_full_world_map(out_dir)
    return count

if __name__ == '__main__':
    src = sys.argv[1] if len(sys.argv) > 1 else r"E:\Wiiu emu\Botw_Dump\Games\The Legend of Zelda-Breath of the Wild\content\UI\MapTex\MainField"
    out = sys.argv[2] if len(sys.argv) > 2 else r"C:\Users\Administrator\Desktop\Zevent\ZEventStudio\.cache\map_tiles"
    cnt = extract_from_dir(src, out)
    print(f"Extraction complete: {cnt} tiles in {out}")

