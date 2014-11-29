
use gfx;
use gfx::DeviceHelper;
use graphics::{ Context, BackEnd };
use graphics::BACK_END_MAX_VERTEX_COUNT as BUFFER_SIZE;

use Texture;

static VERTEX_SHADER: gfx::ShaderSource<'static> = shaders!{
    GLSL_120: b"
#version 120
uniform vec4 color;

attribute vec2 pos;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
}
"
    GLSL_150: b"
#version 150 core
uniform vec4 color;

in vec2 pos;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
}
"
};

static FRAGMENT_SHADER: gfx::ShaderSource<'static> = shaders!{
    GLSL_120: b"
#version 120
uniform vec4 color;

void main() {
    gl_FragColor = color;
}
"
    GLSL_150: b"
#version 150 core
uniform vec4 color;

out vec4 o_Color;

void main() {
    o_Color = color;
}
"
};

static VERTEX_SHADER_UV: gfx::ShaderSource<'static> = shaders!{
    GLSL_120: b"
#version 120
uniform sampler2D s_texture;
uniform vec4 color;

attribute vec2 pos;
attribute vec2 uv;

varying vec2 v_UV;

void main() {
    v_UV = uv;
    gl_Position = vec4(pos, 0.0, 1.0);
}
"
    GLSL_150: b"
#version 150 core
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
};

static FRAGMENT_SHADER_UV: gfx::ShaderSource<'static> = shaders!{
    GLSL_120: b"
#version 120
uniform sampler2D s_texture;
uniform vec4 color;

varying vec2 v_UV;

void main()
{
    gl_FragColor = texture2D(s_texture, v_UV) * color;
}
"
    GLSL_150: b"
#version 150 core
uniform sampler2D s_texture;
uniform vec4 color;

out vec4 o_Color;

in vec2 v_UV;

void main()
{
    o_Color = texture(s_texture, v_UV) * color;
}
"
};

static POS_COMPONENTS: uint = 2;
static UV_COMPONENTS: uint = 2;

// Boiler plate for automatic attribute construction.
// Needs to be improved on gfx-rs side.
// For some reason, using ``*_COMPONENT` triggers some macros errors.

#[vertex_format]
struct PositionFormat { pos: [f32, ..2] }

#[vertex_format]
struct ColorFormat { color: [f32, ..4] }

#[vertex_format]
struct TexCoordsFormat { uv: [f32, ..2] }

#[allow(unused_imports)]
#[shader_param(Batch, OwnedBatch)]
struct Params {
    color: [f32, ..4],
}

#[allow(unused_imports)]
#[shader_param(BatchUV, OwnedBatchUV)]
struct ParamsUV {
    color: [f32, ..4],
    s_texture: gfx::shade::TextureParam,
}

/// The graphics back-end.
pub struct G2D {
    buffer_pos: gfx::BufferHandle<f32>,
    buffer_uv: gfx::BufferHandle<f32>,
    batch: OwnedBatch,
    batch_uv: OwnedBatchUV,
}

impl G2D {
    /// Creates a new G2D object.
    pub fn new<D: gfx::Device<C>,
               C: gfx::CommandBuffer>(device: &mut D) -> G2D {
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
        let buffer_uv = device.create_buffer(
            UV_COMPONENTS * BUFFER_SIZE,
            gfx::UsageDynamic);

        let mut mesh = gfx::Mesh::new(BUFFER_SIZE as u32);
        mesh.attributes.extend(gfx::VertexFormat::generate(
            None::<PositionFormat>,
            buffer_pos.raw()
        ).into_iter());

        // Reuse parameters from `mesh`.
        let mut mesh_uv = mesh.clone();
        mesh_uv.attributes.extend(gfx::VertexFormat::generate(
            None::<TexCoordsFormat>,
            buffer_uv.raw()
        ).into_iter());

        let params = Params {
                color: [1.0, ..4],
            };
        let mut batch = gfx::batch::OwnedBatch::new(mesh, program, params).unwrap();

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
                &[0x20u8, 0xA0u8, 0xC0u8, 0x00u8])
            .unwrap();
        let params_uv = ParamsUV {
            color: [1.0, ..4],
            s_texture: (texture, Some(sampler))
        };
        let mut batch_uv = gfx::batch::OwnedBatch::new(
            mesh_uv, program_uv, params_uv).unwrap();
        
        // Disable culling.
        batch.state.primitive.method = 
            gfx::state::Fill(gfx::state::CullMode::Nothing);
        batch_uv.state.primitive.method =
            gfx::state::Fill(gfx::state::CullMode::Nothing);

        G2D {
            buffer_pos: buffer_pos,
            buffer_uv: buffer_uv,
            batch: batch,
            batch_uv: batch_uv,
        }
    }

    /// Renders graphics to a Gfx renderer.
    pub fn draw<C: gfx::CommandBuffer>(
        &mut self,
        renderer: &mut gfx::Renderer<C>,
        frame: &gfx::Frame,
        f: |c: Context, g: &mut GraphicsBackEnd<C>|
    ) {
        let ref mut g = GraphicsBackEnd::new(
            renderer, 
            frame, 
            self
        );
        let c = Context::abs(
            frame.width as f64,
            frame.height as f64
        );
        f(c, g);
    }
}

/// Used for rendering 2D graphics.
pub struct GraphicsBackEnd<'a, C: 'a + gfx::CommandBuffer> {
    renderer: &'a mut gfx::Renderer<C>,
    frame: &'a gfx::Frame,
    g2d: &'a mut G2D,
}

impl<'a, C: gfx::CommandBuffer> GraphicsBackEnd<'a, C> {
    /// Creates a new object for rendering 2D graphics.
    pub fn new(renderer: &'a mut gfx::Renderer<C>,
               frame: &'a gfx::Frame,
               g2d: &'a mut G2D) -> GraphicsBackEnd<'a, C> {
        GraphicsBackEnd {
            renderer: renderer,
            frame: frame,
            g2d: g2d,
        }
    }
    
    /// Returns true if texture has alpha channel.
    pub fn has_texture_alpha(&self, texture: &Texture) -> bool {
        texture.handle.get_info().format.get_components() == Some(gfx::tex::RGBA)
    }
    
    /// Enabled alpha blending.
    pub fn enable_alpha_blend(&mut self) {
        use std::default::Default;
        use gfx::state::{Normal, Inverse, Factor};

        let blend = gfx::state::Blend {
            value: [1.0, 1.0, 1.0, 1.0],
            color: gfx::state::BlendChannel {
                    equation: gfx::state::FuncAdd,
                    source: Factor(Normal, gfx::state::SourceAlpha),
                    destination: Factor(Inverse, gfx::state::SourceAlpha)
                },
            alpha: Default::default()
        };

        self.g2d.batch.state.blend = Some(blend);
        self.g2d.batch_uv.state.blend = Some(blend);
    }

    /// Disables alpha blending.
    pub fn disable_alpha_blend(&mut self) {
        self.g2d.batch.state.blend = None;
        self.g2d.batch_uv.state.blend = None;
    }
}

impl<'a, C: gfx::CommandBuffer> BackEnd<Texture>
for GraphicsBackEnd<'a, C> {
    fn clear(&mut self, color: [f32, ..4]) {
        let &GraphicsBackEnd {
            ref mut renderer,
            frame,
            ..
        } = self;
        renderer.clear(
                gfx::ClearData {
                    color: color,
                    depth: 0.0,
                    stencil: 0,
                },
                gfx::COLOR,
                frame
            );
    }

    fn color(&mut self, color: [f32, ..4]) {
        self.g2d.batch.param.color = color;
        self.g2d.batch_uv.param.color = color;
    }

    fn tri_list(&mut self, vertices: &[f32]) {
        let &GraphicsBackEnd {
            ref mut renderer,
            ref frame,
            g2d: &G2D {
                ref mut buffer_pos,
                ref mut batch,
                ..
            },
        } = self;

        renderer.update_buffer_vec(*buffer_pos, vertices, 0);

        let n = vertices.len() / POS_COMPONENTS;
        batch.slice = gfx::Slice {
                prim_type: gfx::TriangleList,
                start: 0,
                end: n as u32,
                kind: gfx::SliceKind::Vertex
            };
        renderer.draw(batch, *frame);
    }

    fn enable_texture(&mut self, texture: &Texture) {
        let ParamsUV { 
            s_texture: (ref mut s_texture, _), .. 
        } = self.g2d.batch_uv.param;
        *s_texture = texture.handle;
    }

    fn disable_texture(&mut self) {}

    fn tri_list_uv(&mut self, vertices: &[f32], texture_coords: &[f32]) {
        let &GraphicsBackEnd {
            ref mut renderer,
            ref frame,
            g2d: &G2D {
                ref mut buffer_pos,
                ref mut buffer_uv,
                ref mut batch_uv,
                ..
            },
        } = self;

        assert_eq!(
            vertices.len() * UV_COMPONENTS,
            texture_coords.len() * POS_COMPONENTS
        );
        renderer.update_buffer_vec(*buffer_pos, vertices, 0);
        renderer.update_buffer_vec(*buffer_uv, texture_coords, 0);

        let n = vertices.len() / POS_COMPONENTS;
        batch_uv.slice = gfx::Slice {
                prim_type: gfx::TriangleList,
                start: 0,
                end: n as u32,
                kind: gfx::SliceKind::Vertex
            };
        renderer.draw(batch_uv, *frame);
    }
}
