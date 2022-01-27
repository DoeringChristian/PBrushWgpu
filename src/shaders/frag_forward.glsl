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

layout(set = 1, binding = 0) uniform texture2D t_src;
layout(set = 1, binding = 1) uniform sampler s_src;

void main(){
    o_color = texture(sampler2D(t_src, s_src), f_uv);
    //o_color = vec4(f_uv, 0.0, 1.0);
}
