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
layout(location = 2) out vec2 f_bguv;

layout(set = 0, binding = 0) uniform transforms{
    mat4 model;
    mat4 view;
    mat4 proj;
};
layout(set = 2, binding = 0) uniform Stroke{
    vec2 pos0;
    vec2 pos1;
}stroke;

void main(){
    f_pos = i_pos;
    // have to invert y axis of uv
    f_uv = vec2(i_uv.x, 1-i_uv.y);

    // have to use position because the proj matrix is made for the position
    f_bguv = (((model * proj) * vec4(i_pos, 0.0, 1.0)).xy + vec2(1.0, 1.0)) / 2.0;
    //f_bguv = (model * proj * vec4(i_uv, 0.0, 1.0)).xy;
    gl_Position = vec4(i_pos, 0.0, 1.0);
}
