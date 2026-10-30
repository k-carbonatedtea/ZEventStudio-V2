//! WGPU 核心硬件设备、指令队列与适配器上下文单例管理器

use std::sync::{Arc, OnceLock};

/// WGPU 核心硬件运行上下文
pub struct WgpuContext {
    #[allow(dead_code)]
    pub instance: Arc<wgpu::Instance>,
    #[allow(dead_code)]
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

static WGPU_CONTEXT: OnceLock<Option<Arc<WgpuContext>>> = OnceLock::new();

/// 获取或全局懒加载初始化 WGPU 上下文单例
pub fn get_or_init_wgpu() -> Option<Arc<WgpuContext>> {
    WGPU_CONTEXT
        .get_or_init(|| init_wgpu_context().map(Arc::new))
        .as_ref()
        .cloned()
}

/// 探测并初始化本地 WGPU 驱动及高性能显卡设备
fn init_wgpu_context() -> Option<WgpuContext> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        flags: wgpu::InstanceFlags::default(),
        backend_options: wgpu::BackendOptions::default(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        display: None,
    });

    let adapter = smol::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    }))
    .or_else(|_| {
        smol::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
    })
    .ok()?;

    let adapter_info = adapter.get_info();
    println!(
        "[WGPU] 成功定位渲染物理显卡设备: {} ({:?})",
        adapter_info.name, adapter_info.backend
    );

    let (device, queue) = smol::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ZEvent WGPU Device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        default_queue: wgpu::QueueDescriptor::default(),
    }))
    .map_err(|err| {
        eprintln!("[WGPU] 请求逻辑设备 Device 失败: {err:?}");
        err
    })
    .ok()?;

    Some(WgpuContext {
        instance: Arc::new(instance),
        adapter: Arc::new(adapter),
        device: Arc::new(device),
        queue: Arc::new(queue),
    })
}
