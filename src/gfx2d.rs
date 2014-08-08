
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

static FRAGMENT_SHADER_TRI_LIST_XY_RGBA: gfx::ShaderSource = shaders! {
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

/*
static VERTEX_SHADER_TRI_LIST_XY_RGBA_UV: &'static str = "
#version 330
in vec4 a_v4Position;
in vec4 a_v4FillColor;
in vec2 a_v2TexCoord;
uniform sampler2D s_texture;
out vec2 v_v2TexCoord;
out vec4 v_v4FillColor;
void main()
{
v_v2TexCoord = a_v2TexCoord;
v_v4FillColor = a_v4FillColor;
gl_Position = a_v4Position;
}
";
static FRAGMENT_SHADER_TRI_LIST_XY_RGBA_UV: &'static str = "
#version 330
out vec4 out_color;
uniform sampler2D s_texture;
in vec2 v_v2TexCoord;
in vec4 v_v4FillColor;
void main()
{
out_color = texture(s_texture, v_v2TexCoord) * v_v4FillColor;
}
";
*/

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
