//! Vulkan 设备、队列与显存分配器上下文单例管理器

use std::sync::{Arc, OnceLock};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::VulkanLibrary;

/// Vulkan 核心硬件运行上下文
pub struct VulkanContext {
    #[allow(dead_code)]
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

static VULKAN_CONTEXT: OnceLock<Option<Arc<VulkanContext>>> = OnceLock::new();

/// 获取或全局懒加载初始化 Vulkan 上下文单例
pub fn get_or_init_vulkan() -> Option<Arc<VulkanContext>> {
    VULKAN_CONTEXT
        .get_or_init(|| init_vulkan_context().map(Arc::new))
        .as_ref()
        .cloned()
}

/// 探测并初始化本地 Vulkan 驱动及优先独显设备
fn init_vulkan_context() -> Option<VulkanContext> {
    let library = match VulkanLibrary::new() {
        Ok(lib) => lib,
        Err(err) => {
            eprintln!("[Vulkan] 无法加载 Vulkan 运行时动态库: {err:?}");
            return None;
        }
    };

    let instance = match Instance::new(library, InstanceCreateInfo::default()) {
        Ok(inst) => inst,
        Err(err) => {
            eprintln!("[Vulkan] 无法创建 Vulkan 实例: {err:?}");
            return None;
        }
    };

    let physical_devices = match instance.enumerate_physical_devices() {
        Ok(devs) => devs,
        Err(err) => {
            eprintln!("[Vulkan] 枚举物理显卡设备失败: {err:?}");
            return None;
        }
    };

    // 优先选取独立显卡 (Discrete GPU)，若无则降级至集成显卡
    let physical_device = physical_devices.max_by_key(|p| match p.properties().device_type {
        PhysicalDeviceType::DiscreteGpu => 3,
        PhysicalDeviceType::IntegratedGpu => 2,
        PhysicalDeviceType::VirtualGpu => 1,
        _ => 0,
    })?;

    let gpu_name = &physical_device.properties().device_name;
    println!("[Vulkan] 成功定位渲染物理显卡设备: {gpu_name}");

    // 查找支持图形渲染与传输指令的队列族 (Graphics Queue Family)
    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_idx, q)| q.queue_flags.intersects(QueueFlags::GRAPHICS))? as u32;

    let (device, mut queues) = match Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    ) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("[Vulkan] 创建逻辑设备 Device 失败: {err:?}");
            return None;
        }
    };

    let queue = queues.next()?;
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        Default::default(),
    ));
    let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
        device.clone(),
        Default::default(),
    ));

    Some(VulkanContext {
        instance,
        device,
        queue,
        memory_allocator,
        command_buffer_allocator,
        descriptor_set_allocator,
    })
}
