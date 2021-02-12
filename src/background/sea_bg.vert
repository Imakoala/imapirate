#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 0) out vec4 world_position;
layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
    world_position = Model * vec4(Vertex_Position, 1.0);
}