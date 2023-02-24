struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,  
}

@vertex
fn vertex_shader_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var pos = array<vec2<f32>,6>(
        vec2<f32>(-0.5, 0.7),
        vec2<f32>(0.3, 0.6),
        vec2<f32>(0.5, 0.3),
        vec2<f32>(0.4, -0.5),
        vec2<f32>(-0.4, -0.4),
        vec2<f32>(-0.3, 0.2),
    )
    return vec4<f32>(pos[in_vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 0.0, 1.0);
}