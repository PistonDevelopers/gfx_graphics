
use gfx;
use graphics::{
    BackEnd,
};

use Texture;

static VERTEX_SHADER_TRI_LIST_XY_RGBA: gfx::ShaderSource = shaders!{
    GLSL_120: b"
#version 120
attribute vec2 pos;
attribute vec3 color;
varying vec4 v_Color;
void main() {
    v_Color = vec4(color, 1.0);
    gl_Position = vec4(pos, 0.0, 1.0);
}
"
    GLSL_150: b"
#version 150 core
in vec2 pos;
in vec3 color;
out vec4 v_Color;
void main() {
    v_Color = vec4(color, 1.0);
    gl_Position = vec4(pos, 0.0, 1.0);
}
"
};

static FRAGMENT_SHADER_TRI_LIST_XY_RGBA: gfx::ShaderSource = shaders!{
    GLSL_120: b"
#version 120
varying vec4 v_Color;
void main() {
    gl_FragColor = v_Color;
}
"
    GLSL_150: b"
#version 150 core
in vec4 v_Color;
out vec4 o_Color;
void main() {
    o_Color = v_Color;
}
"
};

static VERTEX_SHADER_TRI_LIST_XY_RGBA_UV: gfx::ShaderSource = shaders!{
    GLSL_120: b"
#version 120
attribute vec2 pos;
attribute vec3 color;
attribute vec2 uv;
uniform sampler2D s_texture;
varying vec4 v_Color;
varying vec2 v_UV;
void main() {
    v_UV = uv;
    v_Color = color;
    gl_Position = a_v4Position;
}
"
    GLSL_150: b"
#version 150 core
in vec2 pos;
in vec3 color;
in vec2 uv;
uniform sampler2D s_texture;
out vec4 v_Color;
out vec2 v_UV;
void main() {
    v_UV = uv;
    v_Color = color;
    gl_Position = a_v4Position;
}
"
};

static FRAGMENT_SHADER_TRI_LIST_XY_RGBA_UV: gfx::ShaderSource = shaders!{
    GLSL_120: b"
#version 120
uniform sampler2D s_texture;
varying vec2 v_UV;
varying vec4 v_Color;
void main()
{
    gl_FragColor = texture(s_texture, v_UV) * v_Color;
}
"
    GLSL_150: b"
#version 150 core
out vec4 o_Color;
uniform sampler2D s_texture;
in vec2 v_UV;
in vec4 v_Color;
void main()
{
    o_Color = texture(s_texture, v_UV) * v_Color;
}
"
};

#[vertex_format]
struct Vertex {
    pos: [f32, ..2],
    color: [f32, ..4],
}

impl Vertex {
    fn new(pos: [f32, ..2], color: [f32, ..4], uv: [f32, ..2]) -> Vertex {
        Vertex {
            pos: pos,
            color: color,
        }
    }
}

#[vertex_format]
struct VertexUV {
    pos: [f32, ..2],
    color: [f32, ..4],
    uv: [f32, ..2],
}

impl VertexUV {
    fn new(pos: [f32, ..2], color: [f32, ..4], uv: [f32, ..2]) -> VertexUV {
        VertexUV {
            pos: pos,
            color: color,
            uv: uv,
        }
    }
}

/// The graphics back-end.
pub struct Gfx2d;

impl Gfx2d {
    /// Creates a new Gfx2d object.
    pub fn new() -> Gfx2d {
        Gfx2d
    }
}

impl BackEnd<Texture> for Gfx2d {

}
