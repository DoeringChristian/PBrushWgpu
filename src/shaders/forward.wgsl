// Vertex Shader

struct VertexInput{
    [[location(0)]] pos: vec2<f32>;
    [[location(1)]] uv: vec2<f32>;
};

struct VertexOutput{
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(model: VertexInput) -> VertexOutput{
    var out: VertexOutput;
    out.uv = model.uv;
    out.clip_position = vec4<f32>(model.pos, 0.0, 1.0);
    return out;
}

// Fragment Shader
[[group(0), binding(0)]]
var t_src: texture_2d<f32>;
[[group(0), binding(1)]]
var s_src: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32>{
    return textureSample(t_src, s_src, in.uv);
}
