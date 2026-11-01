//! WGPU 着色器模块定义与 WGSL 加载

pub const SHADER_WGSL: &str = r#"
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
    @location(3) model0: vec4<f32>,
    @location(4) model1: vec4<f32>,
    @location(5) model2: vec4<f32>,
    @location(6) model3: vec4<f32>,
    @location(7) normal0: vec4<f32>,
    @location(8) normal1: vec4<f32>,
    @location(9) normal2: vec4<f32>,
    @location(10) instance_flags: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) selected: f32,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world = mat4x4<f32>(model.model0, model.model1, model.model2, model.model3);
    let normal = mat3x3<f32>(model.normal0.xyz, model.normal1.xyz, model.normal2.xyz);
    out.clip_position = uniforms.mvp * world * vec4<f32>(model.position, 1.0);
    out.world_normal = normal * model.normal;
    out.uv = model.uv;
    out.selected = model.instance_flags.x;
    return out;
}

@fragment
fn fs_main(in: VertexOutput, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    if (uniforms.flags.w == 3u || (uniforms.flags.w == 1u && !front) || (uniforms.flags.w == 2u && front)) { discard; }
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
    if (uniforms.flags.z == 1u || in.selected > 0.5) {
        rgb = mix(rgb, vec3<f32>(0.2, 0.9, 0.9), 0.35);
    }

    return vec4<f32>(rgb, col.a);
}
"#;

/// 创建 WGPU 原生 WGSL 着色器模块
pub fn create_shader_module(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ZEvent Model Shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER_WGSL.into()),
    })
}

