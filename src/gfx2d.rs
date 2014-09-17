
use gfx;
use gfx::DeviceHelper;
use graphics::BackEnd;
use graphics::BACK_END_MAX_VERTEX_COUNT as BUFFER_SIZE;

use Texture;

static VERTEX_SHADER: gfx::ShaderSource = shaders!{
    GLSL_120: b"
#version 120
attribute vec2 pos;
attribute vec4 color;
varying vec4 v_Color;
void main() {
    v_Color = color;
    gl_Position = vec4(pos, 0.0, 1.0);
}
"
    GLSL_150: b"
#version 150 core
in vec2 pos;
in vec4 color;
out vec4 v_Color;
void main() {
    v_Color = color;
    gl_Position = vec4(pos, 0.0, 1.0);
}
"
};

static FRAGMENT_SHADER: gfx::ShaderSource = shaders!{
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

static VERTEX_SHADER_UV: gfx::ShaderSource = shaders!{
    GLSL_120: b"
#version 120
attribute vec2 pos;
attribute vec4 color;
attribute vec2 uv;
uniform sampler2D s_texture;
varying vec4 v_Color;
varying vec2 v_UV;
void main() {
    v_UV = uv;
    v_Color = color;
    gl_Position = vec4(pos, 0.0, 1.0);
}
"
    GLSL_150: b"
#version 150 core
in vec2 pos;
in vec4 color;
in vec2 uv;
uniform sampler2D s_texture;
out vec4 v_Color;
out vec2 v_UV;
void main() {
    v_UV = uv;
    v_Color = color;
    gl_Position = vec4(pos, 0.0, 1.0);
}
"
};

static FRAGMENT_SHADER_UV: gfx::ShaderSource = shaders!{
    GLSL_120: b"
#version 120
uniform sampler2D s_texture;
varying vec2 v_UV;
varying vec4 v_Color;
void main()
{
    gl_FragColor = texture2D(s_texture, v_UV) * v_Color;
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

static POS_COMPONENTS: uint = 2;
static COLOR_COMPONENTS: uint = 4;
static UV_COMPONENTS: uint = 2;

// Boiler plate for automatic attribute construction.
// Needs to be improved on gfx-rs side.
// For some reason, using ``*_COMPONENT` triggers some macros errors.

#[vertex_format]
struct PositionFormat { v: [f32, ..2] }

#[vertex_format]
struct ColorFormat { v: [f32, ..4] }

#[vertex_format]
struct TexCoordsFormat { v: [f32, ..2] }

#[allow(unused_imports)]
#[shader_param(BatchUV, OwnedBatchUV)]
struct ParamsUV {
    s_texture: gfx::shade::TextureParam,
}

/// The graphics back-end.
pub struct Gfx2d {
    buffer_pos: gfx::BufferHandle<f32>,
    buffer_color: gfx::BufferHandle<f32>,
    buffer_uv: gfx::BufferHandle<f32>,
    batch: gfx::batch::OwnedBatch<(), ()>,
    batch_uv: OwnedBatchUV,
}

impl Gfx2d {
    /// Creates a new Gfx2d object.
    pub fn new<D: gfx::Device<C>,
               C: gfx::CommandBuffer>(device: &mut D) -> Gfx2d {
        let program = device.link_program(
                VERTEX_SHADER.clone(),
                FRAGMENT_SHADER.clone()
            ).unwrap();
        let program_uv = device.link_program(
                VERTEX_SHADER_UV.clone(),
                FRAGMENT_SHADER_UV.clone()
            ).unwrap();

        let buffer_pos = device.create_buffer(
            POS_COMPONENTS * BUFFER_SIZE,
            gfx::UsageDynamic);
        let buffer_color = device.create_buffer(
            COLOR_COMPONENTS * BUFFER_SIZE,
            gfx::UsageDynamic);
        let buffer_uv = device.create_buffer(
            UV_COMPONENTS * BUFFER_SIZE,
            gfx::UsageDynamic);

        let mut mesh = gfx::Mesh::new(BUFFER_SIZE as u32);
        mesh.attributes.push_all_move(gfx::VertexFormat::generate(
            None::<PositionFormat>,
            buffer_pos.raw()
        ));
        mesh.attributes.push_all_move(gfx::VertexFormat::generate(
            None::<ColorFormat>,
            buffer_color.raw()
        ));

        let mut mesh_uv = mesh.clone();
        mesh_uv.attributes.push_all_move(gfx::VertexFormat::generate(
            None::<TexCoordsFormat>,
            buffer_uv.raw()
        ));

        let batch = gfx::batch::OwnedBatch::new(mesh, program, ()).unwrap();

        let sampler = device.create_sampler(
                gfx::tex::SamplerInfo::new(gfx::tex::Trilinear,
                                           gfx::tex::Clamp)
            );

        // Create a dummy texture
        let texture_info = gfx::tex::TextureInfo {
            width: 1,
            height: 1,
            depth: 1,
            levels: 1,
            kind: gfx::tex::Texture2D,
            format: gfx::tex::RGBA8,
        };
        let image_info = texture_info.to_image_info();
        let texture = device.create_texture(texture_info).unwrap();
        device.update_texture(&texture, &image_info,
                [0x20u8, 0xA0u8, 0xC0u8, 0x00u8])
            .unwrap();
        let params_uv = ParamsUV {
            s_texture: (texture, Some(sampler))
        };

        let batch_uv = gfx::batch::OwnedBatch::new(
            mesh_uv, program_uv, params_uv).unwrap();

        Gfx2d {
            buffer_pos: buffer_pos,
            buffer_color: buffer_color,
            buffer_uv: buffer_uv,
            batch: batch,
            batch_uv: batch_uv,
        }
    }
}

/// Used for rendering 2D graphics.
pub struct RenderContext<'a, C: 'a + gfx::CommandBuffer> {
    renderer: &'a mut gfx::Renderer<C>,
    frame: &'a gfx::Frame,
    gfx2d: &'a mut Gfx2d,
}

impl<'a, C: gfx::CommandBuffer> RenderContext<'a, C> {
    /// Creates a new object for rendering 2D graphics.
    pub fn new(renderer: &'a mut gfx::Renderer<C>,
               frame: &'a gfx::Frame,
               gfx2d: &'a mut Gfx2d) -> RenderContext<'a, C> {
        RenderContext {
            renderer: renderer,
            frame: frame,
            gfx2d: gfx2d,
        }
    }
}

impl<'a, C: gfx::CommandBuffer> BackEnd<Texture>
for RenderContext<'a, C> {
    fn supports_clear_rgba(&self) -> bool { true }

    fn clear_rgba(&mut self, r: f32, g: f32, b: f32, a: f32) {
        let &RenderContext {
            ref mut renderer,
            frame,
            ..
        } = self;
        renderer.clear(
                gfx::ClearData {
                    color: [r, g, b, a],
                    depth: 0.0,
                    stencil: 0,
                },
                gfx::Color,
                frame
            );
    }

    fn enable_alpha_blend(&mut self) {
        use std::default::Default;
        use gfx::state::{Normal, Inverse, Factor};

        self.gfx2d.batch.state.blend = Some(gfx::state::Blend {
                value: [1.0, 1.0, 1.0, 1.0],
                color: gfx::state::BlendChannel {
                        equation: gfx::state::FuncAdd,
                        source: Factor(Normal, gfx::state::SourceAlpha),
                        destination: Factor(Inverse, gfx::state::SourceAlpha)
                    },
                alpha: Default::default()
            })
    }

    fn disable_alpha_blend(&mut self) {
        self.gfx2d.batch.state.blend = None;
    }

    fn supports_tri_list_xy_f32_rgba_f32(&self) -> bool { true }

    fn tri_list_xy_f32_rgba_f32(
        &mut self,
        vertices: &[f32],
        colors: &[f32]
    ) {
        let &RenderContext {
            ref mut renderer,
            ref frame,
            gfx2d: &Gfx2d {
                ref mut buffer_pos,
                ref mut buffer_color,
                ref mut batch,
                ..
            },
        } = self;

        assert_eq!(
            vertices.len() * COLOR_COMPONENTS,
            colors.len() * POS_COMPONENTS
        );
        renderer.update_buffer_vec(*buffer_pos, vertices, 0);
        renderer.update_buffer_vec(*buffer_color, colors, 0);

        let n = vertices.len() / POS_COMPONENTS;
        batch.slice = gfx::VertexSlice(gfx::TriangleList, 0, n as u32);
        renderer.draw(&*batch, *frame);
    }

    fn supports_single_texture(&self) -> bool { true }

    fn enable_single_texture(&mut self, texture: &Texture) {
        let ParamsUV {
            s_texture: (ref mut s_texture, _)
        } = self.gfx2d.batch_uv.param;
        *s_texture = texture.handle;
    }

    fn disable_single_texture(&mut self) {}

    // Assume all textures has alpha channel for now.
    fn has_texture_alpha(&self, _texture: &Texture) -> bool { true }

    fn supports_tri_list_xy_f32_rgba_f32_uv_f32(&self) -> bool { true }

    fn tri_list_xy_f32_rgba_f32_uv_f32(
        &mut self,
        vertices: &[f32],
        colors: &[f32],
        texture_coords: &[f32]
    ) {
        let &RenderContext {
            ref mut renderer,
            ref frame,
            gfx2d: &Gfx2d {
                ref mut buffer_pos,
                ref mut buffer_color,
                ref mut buffer_uv,
                ref mut batch_uv,
                ..
            },
        } = self;

        assert_eq!(
            vertices.len() * COLOR_COMPONENTS,
            colors.len() * POS_COMPONENTS
        );
        assert_eq!(
            vertices.len() * UV_COMPONENTS,
            texture_coords.len() * POS_COMPONENTS
        );
        renderer.update_buffer_vec(*buffer_pos, vertices, 0);
        renderer.update_buffer_vec(*buffer_color, colors, 0);
        renderer.update_buffer_vec(*buffer_uv, texture_coords, 0);

        let n = vertices.len() / POS_COMPONENTS;
        batch_uv.slice = gfx::VertexSlice(gfx::TriangleList, 0, n as u32);
        renderer.draw(&*batch_uv, *frame);
    }
}
