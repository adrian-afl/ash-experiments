#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 Normal;
layout(location = 1) in vec2 UV;

layout(set = 0, binding = 0) uniform sampler2D tex;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(
        texture(tex, UV).rgb * (dot(normalize(Normal), vec3(0.0, 1.0, 0.0)) * 0.5 + 0.5), 
        1.0
    );
}