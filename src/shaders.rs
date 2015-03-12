pub static VERTEX_SHADER: [&'static [u8]; 2] = [
b"#version 120
uniform vec4 color;

attribute vec2 pos;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
}
",
b"#version 150 core
uniform vec4 color;

in vec2 pos;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
}
"
];

pub static FRAGMENT_SHADER: [&'static [u8]; 2] = [
b"#version 120
uniform vec4 color;

void main() {
    gl_FragColor = color;
}
",
b"#version 150 core
uniform vec4 color;

out vec4 o_Color;

void main() {
    o_Color = color;
}
"
];

pub static VERTEX_SHADER_UV: [&'static [u8]; 2] = [
b"#version 120
uniform sampler2D s_texture;
uniform vec4 color;

attribute vec2 pos;
attribute vec2 uv;

varying vec2 v_UV;

void main() {
    v_UV = uv;
    gl_Position = vec4(pos, 0.0, 1.0);
}
",
b"#version 150 core
uniform sampler2D s_texture;
uniform vec4 color;

in vec2 pos;
in vec2 uv;
out vec2 v_UV;
void main() {
    v_UV = uv;
    gl_Position = vec4(pos, 0.0, 1.0);
}
"
];

pub static FRAGMENT_SHADER_UV: [&'static [u8]; 2] = [
b"#version 120
uniform sampler2D s_texture;
uniform vec4 color;

varying vec2 v_UV;

void main()
{
    gl_FragColor = texture2D(s_texture, v_UV) * color;
}
",
b"#version 150 core
uniform sampler2D s_texture;
uniform vec4 color;

out vec4 o_Color;

in vec2 v_UV;

void main()
{
    o_Color = texture(s_texture, v_UV) * color;
}
"
];
