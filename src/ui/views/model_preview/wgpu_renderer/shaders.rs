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

struct Camera { mvp: mat4x4<f32>, };
@group(2) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var terrain_layers: texture_2d_array<f32>;
@group(1) @binding(1)
var terrain_sampler: sampler;
@group(1) @binding(2)
var terrain_control: texture_2d<u32>;
struct TerrainParams { values: array<vec4<f32>, 256>, };
@group(1) @binding(3)
var<uniform> terrain_params: TerrainParams;

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
    out.clip_position = camera.mvp * world * vec4<f32>(model.position, 1.0);
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

    return shade_color(col, in);
}

fn shade_color(col: vec4<f32>, in: VertexOutput) -> vec4<f32> {
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

fn terrain_pair(ids: vec2<u32>, weight: f32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec4<f32> {
    let a = terrain_params.values[min(ids.x, 255u)];
    let b = terrain_params.values[min(ids.y, 255u)];
    if (weight <= 0.0) {
        return textureSampleGrad(terrain_layers, terrain_sampler, uv * a.xy, i32(a.z), dx * a.xy, dy * a.xy);
    }
    if (weight >= 1.0) {
        return textureSampleGrad(terrain_layers, terrain_sampler, uv * b.xy, i32(b.z), dx * b.xy, dy * b.xy);
    }
    let color_a = textureSampleGrad(terrain_layers, terrain_sampler, uv * a.xy, i32(a.z), dx * a.xy, dy * a.xy);
    let color_b = textureSampleGrad(terrain_layers, terrain_sampler, uv * b.xy, i32(b.z), dx * b.xy, dy * b.xy);
    return mix(color_a, color_b, weight);
}

@fragment
fn fs_terrain(in: VertexOutput) -> @location(0) vec4<f32> {
    let dx = dpdx(in.uv);
    let dy = dpdy(in.uv);
    let last = vec2<i32>(textureDimensions(terrain_control)) - vec2<i32>(1);
    let p = clamp(in.uv, vec2<f32>(0.0), vec2<f32>(1.0)) * vec2<f32>(last);
    let lo = vec2<i32>(floor(p));
    let hi = min(lo + vec2<i32>(1), last);
    let fraction = fract(p);
    let m00 = textureLoad(terrain_control, lo, 0);
    let m10 = textureLoad(terrain_control, vec2<i32>(hi.x, lo.y), 0);
    let m01 = textureLoad(terrain_control, vec2<i32>(lo.x, hi.y), 0);
    let m11 = textureLoad(terrain_control, hi, 0);
    var color: vec4<f32>;
    if (all(m00.rg == m10.rg) && all(m00.rg == m01.rg) && all(m00.rg == m11.rg)) {
        let weight = mix(mix(f32(m00.b), f32(m10.b), fraction.x), mix(f32(m01.b), f32(m11.b), fraction.x), fraction.y) / 255.0;
        color = terrain_pair(m00.rg, weight, in.uv, dx, dy);
    } else {
        // Interpolate colors at material boundaries, never integer layer IDs.
        let c00 = terrain_pair(m00.rg, f32(m00.b) / 255.0, in.uv, dx, dy);
        let c10 = terrain_pair(m10.rg, f32(m10.b) / 255.0, in.uv, dx, dy);
        let c01 = terrain_pair(m01.rg, f32(m01.b) / 255.0, in.uv, dx, dy);
        let c11 = terrain_pair(m11.rg, f32(m11.b) / 255.0, in.uv, dx, dy);
        color = mix(mix(c00, c10, fraction.x), mix(c01, c11, fraction.x), fraction.y);
    }
    return shade_color(vec4<f32>(color.rgb, 1.0), in);
}
"#;

/// 创建 WGPU 原生 WGSL 着色器模块
pub fn create_shader_module(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ZEvent Model Shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER_WGSL.into()),
    })
}
