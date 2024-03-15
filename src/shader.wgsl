struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct Vertex {
    position: vec3<f32>,
    tex_coords: vec2<f32>,
}

@vertex 
fn vs_main(@builtin(vertex_index) vid: u32) -> VertexOutput {
    var out: VertexOutput;

    var verts = array<Vertex, 6>(
        Vertex(vec3<f32>(-1.0, 1.0, 0.0), vec2<f32>(0.0, 0.0)),
        Vertex(vec3<f32>(-1.0, -1.0, 0.0), vec2<f32>(0.0, 1.0)),
        Vertex(vec3<f32>(1.0, 1.0, 0.0), vec2<f32>(1.0, 0.0)),
        //
        Vertex(vec3<f32>(-1.0, -1.0, 0.0), vec2<f32>(0.0, 1.0)),
        Vertex(vec3<f32>(1.0, -1.0, 0.0), vec2<f32>(1.0, 1.0)),
        Vertex(vec3<f32>(1.0, 1.0, 0.0), vec2<f32>(1.0, 0.0)),
    );

    let vert = verts[vid];
    out.clip_pos = vec4<f32>(vert.position, 1.0); // Quad: 
    out.tex_coords = vert.tex_coords;
    return out;
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var s: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, s, in.tex_coords);
}
