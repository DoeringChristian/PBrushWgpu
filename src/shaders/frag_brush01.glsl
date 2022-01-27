#version 460

layout(location = 0) out vec4 o_color;

struct Transforms{
    mat4 model;
    mat4 view;
    mat4 proj;
};

layout(location = 0) in vec2 f_pos;
layout(location = 1) in vec2 f_uv;

layout(set = 0, binding = 0) uniform transforms{
    mat4 model;
    mat4 view;
    mat4 proj;
};

layout(set = 1, binding = 0) uniform texture2D t_self;
layout(set = 1, binding = 1) uniform sampler s_self;

layout(set = 2, binding = 0) uniform stroke{
    vec2 pos0;
    vec2 pos1;
};


layout(set = 3, binding = 0) uniform texture2D t_background;
layout(set = 3, binding = 1) uniform sampler s_background;

float fallofn(float x){
    return exp(-(x * x));
}

float falloft(float t){
    if(t > 0.0 && t < 1.0)
        return 1.0;
    return 0.0;
}

void main(){

    vec2 uv = f_uv;

    vec2 n = normalize(pos1 - pos0);
    float t = dot(n, uv - pos0);
    vec2 p = t * n + pos0;
    float d = length(p - uv);

    vec4 brush_val = vec4(fallofn(d * 50.0), 0.0, 0.0, 1.0) * falloft(t);

    o_color = texture(sampler2D(t_self, s_self), f_uv) * brush_val;
}
