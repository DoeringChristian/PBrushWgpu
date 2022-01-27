#version 460

layout(location = 0) in vec2 i_pos;
layout(location = 1) in vec2 i_uv;

struct Transforms{
    mat4 model;
    mat4 view;
    mat4 proj;
};

layout(location = 0) out vec2 f_pos;
layout(location = 1) out vec2 f_uv;

layout(set = 0, binding = 0) uniform transforms{
    mat4 model;
    mat4 view;
    mat4 proj;
};

void main(){
    f_pos = i_pos;
    f_uv = i_uv;
    gl_Position = model * proj * vec4(i_pos, 0.0, 1.0);
}
