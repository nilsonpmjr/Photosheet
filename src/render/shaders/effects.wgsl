// Shader de Efeitos de Camada em GPU (Vulkan / WGSL)
// Traduzido dos shaders Metal em Compositor/Rendering/MetalLayerEffects.swift.

struct SpreadUniforms {
    width: u32,
    height: u32,
    reach: u32,
    smallest: u32,
};

struct ShiftUniforms {
    width: u32,
    height: u32,
    dx: f32,
    dy: f32,
};

struct BlurUniforms {
    width: u32,
    height: u32,
    sigma: f32,
    radius: u32,
};

struct ComposeUniforms {
    width: u32,
    height: u32,
    stroke_color: vec4<f32>,
    shadow_color: vec4<f32>,
    overlay_color: vec4<f32>,
    inner_color: vec4<f32>,
    glow_color: vec4<f32>,
    inner_glow_color: vec4<f32>,
    flags: vec4<u32>, // [has_stroke, stroke_inside, has_shadow, has_inner_shadow]
    more: vec4<u32>,  // [has_color_overlay, has_outer_glow, has_inner_glow, unused]
};

@group(0) @binding(0) var<storage, read> in_pixels: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> out_coverage: array<f32>;
@group(0) @binding(2) var<uniform> spread_params: SpreadUniforms;

@compute @workgroup_size(16, 16)
fn effects_alpha(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= spread_params.width || gid.y >= spread_params.height) { return; }
    let index = gid.y * spread_params.width + gid.x;
    out_coverage[index] = in_pixels[index].a;
}

@compute @workgroup_size(16, 16)
fn effects_spread_rows(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= spread_params.width || gid.y >= spread_params.height) { return; }
    let reach = i32(spread_params.reach);
    let x = i32(gid.x);
    var best = select(0.0, 1.0, spread_params.smallest == 1u);

    for (var offset = -reach; offset <= reach; offset++) {
        let sample_x = x + offset;
        var value = 0.0;
        if (sample_x >= 0 && sample_x < i32(spread_params.width)) {
            value = in_pixels[gid.y * spread_params.width + u32(sample_x)].r;
        }
        if (spread_params.smallest == 1u) {
            best = min(best, value);
        } else {
            best = max(best, value);
        }
    }
    out_coverage[gid.y * spread_params.width + gid.x] = best;
}

@compute @workgroup_size(16, 16)
fn effects_spread_columns(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= spread_params.width || gid.y >= spread_params.height) { return; }
    let reach = i32(spread_params.reach);
    let y = i32(gid.y);
    var best = select(0.0, 1.0, spread_params.smallest == 1u);

    for (var offset = -reach; offset <= reach; offset++) {
        let sample_y = y + offset;
        var value = 0.0;
        if (sample_y >= 0 && sample_y < i32(spread_params.height)) {
            value = in_pixels[u32(sample_y) * spread_params.width + gid.x].r;
        }
        if (spread_params.smallest == 1u) {
            best = min(best, value);
        } else {
            best = max(best, value);
        }
    }
    out_coverage[gid.y * spread_params.width + gid.x] = best;
}
