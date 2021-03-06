#version 460

#define M_PI 3.1415926535897932384626433832795

layout(location = 0) out vec4 o_color;

layout(location = 0) in vec2 f_pos;
layout(location = 1) in vec2 f_uv;
layout(location = 2) in vec2 f_bguv;

layout(set = 0, binding = 0) uniform transforms{
    mat4 model;
    mat4 view;
    mat4 proj;
};

layout(set = 1, binding = 0) uniform texture2D t_self;
layout(set = 1, binding = 1) uniform sampler s_self;

layout(set = 2, binding = 0) uniform Stroke{
    vec2 pos0;
    vec2 pos1;
    float p0;
    float p1;
}stroke;


layout(set = 3, binding = 0) uniform texture2D t_background;
layout(set = 3, binding = 1) uniform sampler s_background;

float fallofn(float x){
    return exp(-(x * x));
}

float falloft(float t){

    return fallofn((t - 0.5) / 3.0);
    /*
    if(t > 0.0 && t < 1.0)
        return 1.0;
    return 0.0;
    */
}

void main(){

    vec2 uv = f_bguv;

    vec2 n = normalize(stroke.pos1 - stroke.pos0);
    float t = dot(n, uv - stroke.pos0);
    vec2 p = t * n + stroke.pos0;
    float d = length(p - uv);

    float brush_strength = fallofn(d * 50.0) * falloft(t / length(stroke.pos1 - stroke.pos0));

    o_color = texture(sampler2D(t_self, s_self), f_uv) * (1.0 - brush_strength) + vec4(1.0, 0.0, 0.0, 1.0) * brush_strength;
    //o_color = texture(sampler2D(t_self, s_self), f_uv) + vec4(1.0, 0.0, 0.0, 0.0) * brush_strength;
}
