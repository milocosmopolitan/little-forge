import bevy_ui_render::ui_vertex_output::UiVertexOutput;
@group(1) @binding(0) var<uniform> strength: vec4<f32>;
@group(1) @binding(1) var plate: texture_2d<f32>;
@group(1) @binding(2) var plate_sampler: sampler;
@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let source = textureSample(plate, plate_sampler, uv).rgb;
    // Registration follows the blade from (0.17,0.54) to (0.75,0.79).
    // Highlight only hot metal; the hand and hammer never change.
    let line_y = 0.54 + (uv.x - 0.17) * 0.41;
    let distance = abs(uv.y - line_y);
    let ends = smoothstep(0.12, 0.25, uv.x) * (1.0-smoothstep(0.72,0.77,uv.x));
    let hot = smoothstep(0.25,0.85,source.r) * (1.0-smoothstep(0.3,0.6,source.b));
    let core = exp(-distance*distance*4200.0) * hot;
    let halo = exp(-distance*distance*1300.0) * 0.25;
    return vec4<f32>(1.0, 0.3 + core*0.3, 0.025, (core+halo)*ends*strength.x);
}
