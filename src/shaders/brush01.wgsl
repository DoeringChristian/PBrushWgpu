// Vertex Shader

struct Stroke{
    pos0: vec2<f32>;
    pos1: vec2<f32>;
};

struct Transforms{
    model: mat4x4<f32>;
    view: mat4x4<f32>;
    proj: mat4x4<f32>;
};

struct VertexInput{
    [[location(0)]] pos: vec2<f32>;
    [[location(1)]] uv: vec2<f32>;
};

struct VertexOutput{
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

[[group(3), binding(0)]]
var<uniform> transforms: Transforms;

[[stage(vertex)]]
fn vs_main(model: VertexInput) -> VertexOutput{
    var out: VertexOutput;
    out.uv = model.uv;
    out.clip_position = vec4<f32>(model.pos, 0.0, 1.0);
    return out;
}

// Fragment Shader
[[group(0), binding(0)]]
var t_background: texture_2d<f32>;
[[group(0), binding(1)]]
var s_background: sampler;

[[group(1), binding(0)]]
var t_self: texture_2d<f32>;
[[group(1), binding(1)]]
var s_self: sampler;

[[group(2), binding(0)]]
var<uniform> stroke: Stroke;

fn fallof(x: f32) -> f32{
    return exp(-(x * x));
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32>{

    let uv = in.uv;

    var n:vec2<f32> = normalize((stroke.pos1 - stroke.pos0));
    var t: f32 = dot(n, uv - stroke.pos0);
    var p: vec2<f32> = t * n + stroke.pos0;
    var d: f32 = length(p - uv);

    var brush_val: vec4<f32> = vec4<f32>(fallof(d * 50.0), 0.0, 0.0, 0.0);

    if(t < 0.0 || t > 1.0){
        brush_val = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    return textureSample(t_self, s_self, in.uv) * brush_val;
}
