# BOTW 地图编辑器

在设置中配置对应平台的游戏资源目录，然后从文件树或“文件 → 打开”打开 `.smubin` / `.mubin`。支持磁盘文件和 SARC/Pack 内的地图条目，也可以把文件路径作为程序启动参数。资源解析在后台执行，首次打开较大的地图需要等待。

## 场景与资源

- 根据 ActorInfo 的 `bfres`、`mainModel` 选择对象子模型，自动搜索 `-00`、`-01` 等分包；缺少 ActorInfo 模型配置时读取 Actor Pack 中的 ModelList。
- 读取材质的着色器采样器映射、纹理数组层、UV 变换和正反面设置，避免把法线或遮罩显示成颜色。岩壁共享地形纹理数组。
- 按对象的位置、XYZ 欧拉旋转、非均匀缩放绘制实例。共享同一模型的对象复用几何与纹理。UMii NPC 使用现有 UMii 部件合成功能。
- 自动显示同一区域的 Static/Dynamic 配套地图。配套对象作为只读背景；点击顶部“配套地图”可切换到对应文件编辑。保存只写回当前标签对应的文件。
- 神庙额外加载 Pack 中的 `DgnMrgPrt_Dungeon*.sbfres` 合并建筑及配套纹理，作为只读背景。
- 地形读取 TSCB、256×256 高度与材质数据，水面单独读取 64×64 高度。当前区域使用最小 250 米的地形瓦片；裁掉被更细瓦片覆盖的粗瓦片部分。
- Switch 的 `Terrain.Tex.sbfres` / `Terrain.Tex1.sbfres` 可直接解码，包含 88 个材质编号到 83 个纹理层的映射。也支持 `MaterialAlb_*.png` 缓存作为解码失败时的回退：环境变量 `ZEVENT_TERRAIN_CACHE`、项目的 `Terrain/Materials`，或程序工作目录的 `.cache/map_terrain`。
- 地形保留原生 1024×1024 材质数组，由 GPU 按 256×256 材质控制图混合并重复采样；自动生成 mipmap，并使用 8 倍各向异性过滤。近景不再把整块 250 米地形压进一张 512×512 烘焙图。软件渲染回退仍使用低分辨率缩略纹理。
- 区域与逻辑标记默认隐藏，选中时显示。支持 Box、Sphere、Cylinder 标记；资源缺失会在对象列表和属性栏显示诊断。

## 操作

| 操作 | 方法 |
| --- | --- |
| 选中对象 | 左键点击实际模型，或从左侧列表选择 |
| 搜索 | 输入 Actor 名称或十六进制 HashId |
| 旋转视角 | 右键拖动，或 Alt+左键拖动 |
| 平移视角 | 中键拖动 |
| 缩放视角 | 滚轮 |
| 聚焦对象 | `F` 或双击列表对象 |
| 移动对象 | 选中后 Shift+左键拖动，在当前高度的 XZ 平面移动 |
| 精确变换 | 右侧输入位置、旋转角度或缩放，按 Enter 应用；Esc 取消输入 |
| 吸附 | “吸附到地形”将对象原点放到高度场表面 |
| 复制 / 删除 | `Ctrl+D` / `Delete`，或属性栏按钮 |
| 撤销 / 重做 | `Ctrl+Z` / `Ctrl+Y` / `Ctrl+Shift+Z`，最多 100 步 |
| 保存 | `Ctrl+S` 或顶部保存按钮 |

保存保留 BYML 版本、端序、Yaz0 压缩状态、未知字段和 Rails 数据。变换后清零 SRTHash；复制生成唯一 HashId；删除清理当前文件内指向被删除对象的连线。SARC 内的文件通过现有归档写入器回写。

## 加载与视口性能

资源查找、SARC 索引与 BFRES 解析在一次加载内复用；仅并行解码当前显示材质使用的颜色纹理，跳过本预览着色器未使用的法线和遮罩。BNTX 优先读取原生纹理指针表。地图标签共享最近使用的已解码地形数组，重新加载时检查源文件的长度和修改时间。

视口在独立线程渲染，快速拖动时保留最新待处理请求，避免输入积压。相机变化复用 GPU 绘制命令，只更新相机矩阵；编辑、撤销、选择、可见性或资源变化会使相应缓存失效。场景快照共享几何和纹理，静止地图停止持续重绘。点击拾取与选择框使用当前已显示帧的相机。

2026-11-02，本机 RTX 5080 / Vulkan、开发构建、F-5 Dynamic（含配套地图，782 网格、5,153,629 三角形），相同 1766×1172 渲染尺寸的实测：

| 指标 | 优化前 | 优化后 |
| --- | ---: | ---: |
| 地图资源加载 | 50.2 秒 | 5.9 秒 |
| 解码并保留的场景纹理 | 1725 | 673 |
| 旋转时渲染及图像读回耗时，中位数 | 23.59 毫秒 | 3.72 毫秒 |
| 旋转时渲染及图像读回耗时，12 帧最大值 | 31.37 毫秒 | 5.29 毫秒 |
| 首帧 GPU 准备、渲染及读回 | 0.74 秒 | 0.94 秒 |

原生地形提高了首帧纹理上传量。以上单帧耗时包含离屏 GPU 渲染与 CPU 读回，不等同于软件界面 FPS；未清空操作系统磁盘缓存，也不代表所有地图和显卡的性能。GPUI 仍通过图像接收离屏结果，尚未实现直接共享 GPU 纹理。

## 当前范围

这是对象放置与静态场景预览功能。地形高度编辑、Rails 控制点编辑、游戏运行时动画、完整材质着色器、UMii 面部与体型动态变形尚未实现。水面使用简化材质。神庙中缺少独立模型的建筑由合并背景显示；修改这些对象的放置数据不会重建合并模型、碰撞或导航数据。Wii U 地图的读写保留大端格式，但其 BFRES/FTEX 显示能力受现有解析器限制，未达到 Switch 的验证范围。

## 验证

```powershell
# 独立格式与编辑操作测试
cd H:\Zevent\UKingLibrary
cargo test --offline

# 编辑器回归测试（不需要游戏资源）
cd H:\Zevent\ZEventStudio
cargo test --offline --bin nodeflow-editor map_editor

# 本地真实地图、模型、地形、水面、GPU 渲染与保存往返测试
$env:ZEVENT_TEST_MAP = '你的 romfs\Map\MainField\F-5\F-5_Dynamic.smubin'
$env:ZEVENT_TEST_RENDER = 'H:\Zevent\ZEventStudio\.cache\map-validation.png'
cargo test --offline --bin nodeflow-editor real_map_models_terrain_render_and_save -- --ignored --nocapture

# 同一场景的加载、首帧及旋转性能（沿用上面的环境变量）
$env:ZEVENT_PROFILE_LOAD = '1'
cargo test --offline --bin nodeflow-editor map_performance_benchmark -- --ignored --nocapture

# GPU 地形细节、编辑后缓存失效、连续输入最终帧回归
cargo test --offline --bin nodeflow-editor native_terrain_repeats_detail -- --ignored
cargo test --offline --bin nodeflow-editor gpu_render_cache -- --ignored
cargo test --offline --bin nodeflow-editor latest_camera_request -- --ignored
```

真实资源测试不包含游戏数据，需使用本机配置的资源目录。地形参考图片对比测试还需要本机的 Rito PNG 缓存。

已在 Switch 的 F-5、D-7 和 Dungeon085 验证模型、地形、水面及神庙合并背景的加载与 GPU 渲染。F-5、D-7 的可显示模型类型均成功解析。另用 F-5 副本在真实界面执行数值修改、撤销、重做和保存，并比较整个 BYML，确认只有选定对象的位置与 SRTHash 改变，原始地图未改动。
