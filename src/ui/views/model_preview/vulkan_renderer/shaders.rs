//! Vulkan 着色器模块与 WGSL/SPIR-V 动态编译

use std::sync::Arc;
use vulkano::device::Device;
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};

const SHADER_WGSL: &str = r#"
struct Uniforms {
    mvp: mat4x4<f32>,
    sun_dir: vec4<f32>,
    base_color: vec4<f32>,
    flags: vec4<u32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(2)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.mvp * vec4<f32>(model.position, 1.0);
    out.world_normal = model.normal;
    out.uv = model.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var col: vec4<f32>;
    if (uniforms.flags.x == 1u) {
        // 绑定了漫反射贴图时，使用贴图真实采样色彩
        col = textureSample(t_diffuse, s_diffuse, in.uv);
    } else {
        // 无贴图几何网格使用材质/部件区分底色
        col = uniforms.base_color;
    }

    // Alpha Cutout 镂空测试 (如睫毛、花纹边缘、镂空披风)
    if (uniforms.flags.y == 1u && col.a < 0.15) {
        discard;
    }

    // 双面半兰伯特平滑光照 (Double-sided Half-Lambert)
    let n_len = length(in.world_normal);
    let n = select(vec3<f32>(0.0, 1.0, 0.0), in.world_normal / n_len, n_len > 0.001);
    let l = normalize(uniforms.sun_dir.xyz);
    let dot_nl = dot(n, l);
    let half_lambert = max(dot_nl * 0.5 + 0.5, (-dot_nl) * 0.4 + 0.4);
    let lit = clamp(0.25 + 0.75 * half_lambert, 0.20, 1.0);

    var rgb = col.rgb * lit;

    // 选中网格高亮渲染态
    if (uniforms.flags.z == 1u) {
        rgb = mix(rgb, vec3<f32>(0.2, 0.9, 0.9), 0.35);
    }

    return vec4<f32>(rgb, col.a);
}
"#;

/// 编译 WGSL 并构建 Vulkan ShaderModule
pub fn create_shader_module(device: Arc<Device>) -> Option<Arc<ShaderModule>> {
    let module = match naga::front::wgsl::parse_str(SHADER_WGSL) {
        Ok(m) => m,
        Err(err) => {
            eprintln!("[Vulkan] WGSL 解析失败: {err:?}");
            return None;
        }
    };

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    );
    let info = match validator.validate(&module) {
        Ok(i) => i,
        Err(err) => {
            eprintln!("[Vulkan] Naga 校验着色器失败: {err:?}");
            return None;
        }
    };

    let spv = match naga::back::spv::write_vec(
        &module,
        &info,
        &naga::back::spv::Options {
            lang_version: (1, 0),
            flags: naga::back::spv::WriterFlags::empty(),
            ..Default::default()
        },
        None,
    ) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("[Vulkan] Naga 生成 SPIR-V 字节码失败: {err:?}");
            return None;
        }
    };

    // SAFETY: Naga 已对 SPIR-V 实施全面语义与格式校验
    unsafe {
        match ShaderModule::new(device, ShaderModuleCreateInfo::new(&spv)) {
            Ok(shader) => Some(shader),
            Err(err) => {
                eprintln!("[Vulkan] 创建 ShaderModule 失败: {err:?}");
                None
            }
        }
    }
}
