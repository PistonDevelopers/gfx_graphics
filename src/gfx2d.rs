
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

#[vertex_format]
struct Vertex {
    pos: [f32, ..2],
    color: [f32, ..4],
}

impl Vertex {
    fn new(pos: [f32, ..2], color: [f32, ..4]) -> Vertex {
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

#[allow(unused_imports)]
#[shader_param(BatchUV, OwnedBatchUV)]
struct ParamsUV {
    s_texture: gfx::shade::TextureParam,
}

/// The graphics back-end.
pub struct Gfx2d<C: gfx::CommandBuffer> {
    buffer: gfx::BufferHandle<Vertex>,
    buffer_uv: gfx::BufferHandle<VertexUV>,
    batch: gfx::batch::OwnedBatch<(), ()>,
    batch_uv: OwnedBatchUV,
}

impl<C: gfx::CommandBuffer> Gfx2d<C> {
    /// Creates a new Gfx2d object.
    pub fn new<D: gfx::Device<C>>(device: &mut D) -> Gfx2d<C> {
        let program = device.link_program(
                VERTEX_SHADER.clone(),
                FRAGMENT_SHADER.clone()
            ).unwrap();
        let program_uv = device.link_program(
                VERTEX_SHADER_UV.clone(),
                FRAGMENT_SHADER_UV.clone()
            ).unwrap();
        let buffer = device.create_buffer(BUFFER_SIZE, gfx::UsageDynamic);
        let buffer_uv = device.create_buffer(BUFFER_SIZE, gfx::UsageDynamic);
        let mesh = gfx::Mesh::from_format(buffer, BUFFER_SIZE as u32);
        let mesh_uv = gfx::Mesh::from_format(buffer_uv, BUFFER_SIZE as u32);
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
                &vec![0x20u8, 0xA0u8, 0xC0u8, 0x00u8].as_slice())
            .unwrap();
        let params_uv = ParamsUV {
            s_texture: (texture, Some(sampler))
        };

        let batch_uv = gfx::batch::OwnedBatch::new(
            mesh_uv, program_uv, params_uv).unwrap();

        Gfx2d {
            buffer: buffer,
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
    gfx2d: &'a mut Gfx2d<C>,
}

impl<'a, C: gfx::CommandBuffer> RenderContext<'a, C> {
    /// Creates a new object for rendering 2D graphics.
    pub fn new(renderer: &'a mut gfx::Renderer<C>,
               frame: &'a gfx::Frame,
               gfx2d: &'a mut Gfx2d<C>) -> RenderContext<'a, C> {
        RenderContext {
            renderer: renderer,
            frame: frame,
            gfx2d: gfx2d
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
                ref mut buffer,
                ref mut batch,
                ..
            }
        } = self;
        let n = vertices.len() / 2;
        let mut vertex_data = Vec::with_capacity(n);
        for i in range(0, n) {
            vertex_data.push(
                Vertex::new(
                    [vertices[2 * i], vertices[2 * i + 1]],
                    [
                        colors[4 * i],
                        colors[4 * i + 1],
                        colors[4 * i + 2],
                        colors[4 * i + 3]
                    ]
                )
            );
        }

        let n = vertex_data.len();
        renderer.update_buffer_vec(*buffer, vertex_data, 0);
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
                ref mut buffer_uv,
                ref mut batch_uv,
                ..
            }
        } = self;
        let n = vertices.len() / 2;
        let mut vertex_data = Vec::with_capacity(n);
        for i in range(0, n) {
            vertex_data.push(
                VertexUV::new(
                    [vertices[2 * i], vertices[2 * i + 1]],
                    [
                        colors[4 * i],
                        colors[4 * i + 1],
                        colors[4 * i + 2],
                        colors[4 * i + 3]
                    ],
                    [
                        texture_coords[2 * i],
                        texture_coords[2 * i + 1]
                    ]
                )
            );
        }

        let n = vertex_data.len();
        renderer.update_buffer_vec(*buffer_uv, vertex_data, 0);
        batch_uv.slice = gfx::VertexSlice(gfx::TriangleList, 0, n as u32);
        renderer.draw(&*batch_uv, *frame);
    }
}
