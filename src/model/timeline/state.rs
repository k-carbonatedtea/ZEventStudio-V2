use revfl::timeline::Timeline;

/// 时间线选中项枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimelineSelection {
    #[default]
    None,
    /// 选中的剪辑索引
    Clip(usize),
    /// 选中的角色索引
    Actor(usize),
    /// 选中的分镜镜头索引
    Cut(usize),
    /// 选中的单帧触发器索引
    #[allow(dead_code)]
    Oneshot(usize),
}

/// 剪辑拖拽调整模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragMode {
    /// 整体平移时间位置
    Move,
    /// 左侧手柄调整起始帧与时长 (Trim Start)
    ResizeStart,
    /// 右侧手柄调整时长 (Trim End)
    ResizeEnd,
}

/// 正在进行的剪辑拖拽状态
#[derive(Debug, Clone, PartialEq)]
pub struct DraggingClipState {
    pub clip_idx: usize,
    pub drag_mode: DragMode,
    pub initial_start_time: f32,
    pub initial_duration: f32,
    pub initial_mouse_x: f32,
    pub initial_actor_idx: usize,
}

/// BFEVTM 时间线文档的运行时编辑状态
#[derive(Debug, Clone)]
pub struct TimelineDocumentState {
    /// 当前时间线核心数据
    pub timeline: Timeline,
    /// 原始加载时的快照 (用于精确比对是否被实质修改)
    pub original_timeline: Timeline,
    /// 当前播放指针游标帧数 (Playhead Frame, 0.0 .. duration)
    pub current_frame: f32,
    /// 是否正在播放动画时间线
    pub is_playing: bool,
    /// 播放帧率 (通常 30.0 或 60.0 FPS)
    pub fps: f32,
    /// 视口缩放比例 (每帧占据的像素宽度，默认 1.5 px/frame)
    pub zoom: f32,
    /// 轨道水平滚动偏移量
    pub scroll_x: f32,
    /// 轨道垂直滚动偏移量
    #[allow(dead_code)]
    pub scroll_y: f32,
    /// 当前选中的时间线元素
    pub selected: TimelineSelection,
    /// 剪辑拖拽交互状态
    pub dragging_clip: Option<DraggingClipState>,
    /// 是否正在标尺上拖动播放游标 (Scrubbing)
    pub is_scrubbing: bool,
    /// 左侧角色轨道列表面板宽度 (默认 240.0 px)
    pub header_width: f32,
    /// 轨道搜索与过滤关键字
    #[allow(dead_code)]
    pub filter_query: String,
    /// 是否弹出「新建剪辑」对话框
    pub modal_add_clip_open: bool,
    /// 是否弹出「新建角色」对话框
    pub modal_add_actor_open: bool,
    /// 是否弹出「新建镜头」对话框
    pub modal_add_cut_open: bool,
    /// 是否折叠右侧属性检查器
    pub inspector_collapsed: bool,
}

impl TimelineDocumentState {
    /// 基于加载的 Timeline 数据创建编辑状态
    pub fn new(timeline: Timeline) -> Self {
        let original_timeline = timeline.clone();
        Self {
            timeline,
            original_timeline,
            current_frame: 0.0,
            is_playing: false,
            fps: 30.0,
            zoom: 1.5,
            scroll_x: 0.0,
            scroll_y: 0.0,
            selected: TimelineSelection::None,
            dragging_clip: None,
            is_scrubbing: false,
            header_width: 240.0,
            filter_query: String::new(),
            modal_add_clip_open: false,
            modal_add_actor_open: false,
            modal_add_cut_open: false,
            inspector_collapsed: false,
        }
    }

    /// 在帧率循环中推进游标播放
    pub fn advance_frame(&mut self, elapsed_secs: f32) {
        if !self.is_playing {
            return;
        }
        let delta_frames = elapsed_secs * self.fps;
        let mut next = self.current_frame + delta_frames;
        let max_duration = self.timeline.duration.max(1.0);
        if next >= max_duration {
            next = 0.0; // 循环播放
        }
        self.current_frame = next;
    }

    /// 切换播放 / 暂停
    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
    }

    /// 停止播放并返回第 0 帧
    pub fn stop_and_rewind(&mut self) {
        self.is_playing = false;
        self.current_frame = 0.0;
    }

    /// 精确步进指定帧数
    pub fn step_frame(&mut self, delta: f32) {
        self.is_playing = false;
        let next = (self.current_frame + delta).clamp(0.0, self.timeline.duration.max(1.0));
        self.current_frame = next;
    }

    /// 检查当前时间线是否发生了实质性变动
    pub fn is_modified(&self) -> bool {
        crate::model::timeline::diff::is_timeline_modified(&self.timeline, &self.original_timeline)
    }
}

impl PartialEq for TimelineDocumentState {
    fn eq(&self, other: &Self) -> bool {
        (self.current_frame - other.current_frame).abs() < 0.001
            && self.is_playing == other.is_playing
            && (self.fps - other.fps).abs() < 0.001
            && (self.zoom - other.zoom).abs() < 0.001
            && self.selected == other.selected
            && !crate::model::timeline::diff::is_timeline_modified(&self.timeline, &other.timeline)
    }
}
