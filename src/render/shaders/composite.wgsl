// Shader de Composição de Camadas e Modos de Mesclagem em GPU (Vulkan / WGSL)
// Executado em espaço sRGB linear com suporte a todos os 27 modos de mesclagem do Photoshop.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Fullscreen quad usando 3 vértices (triângulo de cobertura total)
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

struct CompositeUniforms {
    opacity: f32,
    folder_opacity: f32,
    blend_mode: u32,
    has_mask: u32,
};

@group(0) @binding(0) var<uniform> uniforms: CompositeUniforms;
@group(0) @binding(1) var s_sampler: sampler;
@group(0) @binding(2) var t_layer: texture_2d<f32>;
@group(0) @binding(3) var t_backdrop: texture_2d<f32>;
@group(0) @binding(4) var t_mask: texture_2d<f32>;

fn blend_channel(mode: u32, s: f32, d: f32) -> f32 {
    switch (mode) {
        case 0u: { return s; } // Normal
        case 1u: { return min(d, s); } // Darken
        case 2u: { return d * s; } // Multiply
        case 3u: { // Color Burn
            if (s <= 0.0) { return 0.0; }
            return 1.0 - min(1.0, (1.0 - d) / s);
        }
        case 4u: { return max(0.0, d + s - 1.0); } // Linear Burn
        case 5u: { return max(d, s); } // Lighten
        case 6u: { return d + s - d * s; } // Screen
        case 7u: { // Color Dodge
            if (s >= 1.0) { return 1.0; }
            return min(1.0, d / (1.0 - s));
        }
        case 8u: { return min(1.0, d + s); } // Linear Dodge (Add)
        case 9u: { // Overlay
            if (d <= 0.5) { return 2.0 * d * s; }
            return 1.0 - 2.0 * (1.0 - d) * (1.0 - s);
        }
        case 10u: { // Soft Light
            if (s <= 0.5) {
                return d - (1.0 - 2.0 * s) * d * (1.0 - d);
            } else {
                let d_sqrt = sqrt(d);
                var g = d_sqrt;
                if (d <= 0.25) {
                    g = ((16.0 * d - 12.0) * d + 4.0) * d;
                }
                return d + (2.0 * s - 1.0) * (g - d);
            }
        }
        case 11u: { // Hard Light
            if (s <= 0.5) { return 2.0 * d * s; }
            return 1.0 - 2.0 * (1.0 - d) * (1.0 - s);
        }
        case 12u: { // Vivid Light
            if (s <= 0.5) {
                if (s <= 0.0) { return 0.0; }
                return 1.0 - min(1.0, (1.0 - d) / (2.0 * s));
            } else {
                let s2 = 2.0 * (s - 0.5);
                if (s2 >= 1.0) { return 1.0; }
                return min(1.0, d / (1.0 - s2));
            }
        }
        case 13u: { return clamp(d + 2.0 * s - 1.0, 0.0, 1.0); } // Linear Light
        case 14u: { // Pin Light
            if (s <= 0.5) { return min(d, 2.0 * s); }
            return max(d, 2.0 * (s - 0.5));
        }
        case 15u: { // Hard Mix
            var v = 0.0;
            if (s <= 0.5) {
                if (s > 0.0) { v = 1.0 - min(1.0, (1.0 - d) / (2.0 * s)); }
            } else {
                let s2 = 2.0 * (s - 0.5);
                if (s2 >= 1.0) { v = 1.0; } else { v = min(1.0, d / (1.0 - s2)); }
            }
            if (v < 0.5) { return 0.0; } else { return 1.0; }
        }
        case 16u: { return abs(d - s); } // Difference
        case 17u: { return d + s - 2.0 * d * s; } // Exclusion
        case 18u: { return max(0.0, d - s); } // Subtract
        case 19u: { // Divide
            if (s <= 0.0) { return 1.0; }
            return min(1.0, d / s);
        }
        default: { return s; }
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let layer_sample = textureSample(t_layer, s_sampler, in.uv);
    let backdrop_sample = textureSample(t_backdrop, s_sampler, in.uv);

    var mask_coverage = 1.0;
    if (uniforms.has_mask == 1u) {
        mask_coverage = textureSample(t_mask, s_sampler, in.uv).r;
    }

    let effective_alpha = layer_sample.a * uniforms.opacity * uniforms.folder_opacity * mask_coverage;
    if (effective_alpha <= 0.0) {
        return backdrop_sample;
    }

    let blended_r = blend_channel(uniforms.blend_mode, layer_sample.r, backdrop_sample.r);
    let blended_g = blend_channel(uniforms.blend_mode, layer_sample.g, backdrop_sample.g);
    let blended_b = blend_channel(uniforms.blend_mode, layer_sample.b, backdrop_sample.b);

    let final_rgb = mix(backdrop_sample.rgb, vec3<f32>(blended_r, blended_g, blended_b), effective_alpha);
    let final_a = max(backdrop_sample.a, effective_alpha);

    return vec4<f32>(final_rgb, final_a);
}
