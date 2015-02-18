
use gfx;
use graphics::{ Context, BackEnd };
use graphics::BACK_END_MAX_VERTEX_COUNT as BUFFER_SIZE;

use Texture;

static VERTEX_SHADER: &'static [u8] = b"
#version 120
uniform vec4 color;

attribute vec2 pos;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
}
";

static FRAGMENT_SHADER: &'static [u8] = b"
#version 120
uniform vec4 color;

void main() {
    gl_FragColor = color;
}
";

static VERTEX_SHADER_UV: &'static [u8] = b"
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
";

static FRAGMENT_SHADER_UV: &'static [u8] = b"
#version 120
uniform sampler2D s_texture;
uniform vec4 color;

varying vec2 v_UV;

void main()
{
    gl_FragColor = texture2D(s_texture, v_UV) * color;
}
";

static POS_COMPONENTS: usize = 2;
static UV_COMPONENTS: usize = 2;

// Boiler plate for automatic attribute construction.
// Needs to be improved on gfx-rs side.
// For some reason, using ``*_COMPONENT` triggers some macros errors.

#[vertex_format]
struct PositionFormat { pos: [f32; 2] }

#[vertex_format]
struct ColorFormat { color: [f32; 4] }

#[vertex_format]
struct TexCoordsFormat { uv: [f32; 2] }

#[shader_param]
struct Params {
    color: [f32; 4],
}

#[shader_param]
struct ParamsUV {
    color: [f32; 4],
    s_texture: gfx::shade::TextureParam,
}

/// The graphics back-end.
pub struct G2D {
    buffer_pos: gfx::BufferHandle<f32>,
    buffer_uv: gfx::BufferHandle<f32>,
    batch: gfx::batch::OwnedBatch<Params>,
    batch_uv: gfx::batch::OwnedBatch<ParamsUV>,
}

impl G2D {
    /// Creates a new G2D object.
    pub fn new<D: gfx::Device>(device: &mut D) -> G2D {
        use gfx::DeviceExt;
        let program = device.link_program(
            VERTEX_SHADER, FRAGMENT_SHADER)
            .unwrap();
        let program_uv = device.link_program(
            VERTEX_SHADER_UV, FRAGMENT_SHADER_UV)
            .unwrap();

        let buffer_pos = device.create_buffer(
            POS_COMPONENTS * BUFFER_SIZE,
            gfx::BufferUsage::Dynamic);
        let buffer_uv = device.create_buffer(
            UV_COMPONENTS * BUFFER_SIZE,
            gfx::BufferUsage::Dynamic);

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
            color: [1.0; 4],
        };
        let mut batch = gfx::batch::OwnedBatch::new(mesh, program, params)
            .unwrap();

        let sampler = device.create_sampler(
            gfx::tex::SamplerInfo::new(
                gfx::tex::FilterMethod::Trilinear,
                gfx::tex::WrapMode::Clamp)
        );

        // Create a dummy texture
        let texture_info = gfx::tex::TextureInfo {
            width: 1,
            height: 1,
            depth: 1,
            levels: 1,
            kind: gfx::tex::TextureKind::Texture2D,
            format: gfx::tex::RGBA8,
        };
        let image_info = texture_info.to_image_info();
        let texture = device.create_texture(texture_info)
            .unwrap();
        device.update_texture(&texture, &image_info,
                &[0x20u8, 0xA0u8, 0xC0u8, 0x00u8])
            .unwrap();
        let params_uv = ParamsUV {
            color: [1.0; 4],
            s_texture: (texture, Some(sampler))
        };
        let mut batch_uv = gfx::batch::OwnedBatch::new(
            mesh_uv, program_uv, params_uv)
            .unwrap();

        // Disable culling.
        batch.state.primitive.method =
            gfx::state::RasterMethod::Fill(gfx::state::CullMode::Nothing);
        batch_uv.state.primitive.method =
            gfx::state::RasterMethod::Fill(gfx::state::CullMode::Nothing);

        G2D {
            buffer_pos: buffer_pos,
            buffer_uv: buffer_uv,
            batch: batch,
            batch_uv: batch_uv,
        }
    }

    /// Renders graphics to a Gfx renderer.
    pub fn draw<C, F>(
        &mut self,
        renderer: &mut gfx::Renderer<C>,
        frame: &gfx::Frame,
        mut f: F
    )
        where
            C: gfx::CommandBuffer,
            F: FnMut(Context, &mut GraphicsBackEnd<C>)
    {
        let ref mut g = GraphicsBackEnd::new(
            renderer,
            frame,
            self
        );
        let c = Context::abs(
            frame.width as f64,
            frame.height as f64
        );
        g.enable_alpha_blend();
        f(c, g);
        g.disable_alpha_blend();
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
        use gfx::tex::Components::RGBA;

        texture.handle.get_info().format.get_components() == Some(RGBA)
    }

    /// Enabled alpha blending.
    pub fn enable_alpha_blend(&mut self) {
        use std::default::Default;
        use gfx::state::InverseFlag::{Normal, Inverse};
        use gfx::state::Factor;
        use gfx::state::BlendValue::SourceAlpha;

        let blend = gfx::state::Blend {
            value: [1.0, 1.0, 1.0, 1.0],
            color: gfx::state::BlendChannel {
                    equation: gfx::state::Equation::Add,
                    source: Factor(Normal, SourceAlpha),
                    destination: Factor(Inverse, SourceAlpha)
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

impl<'a, C: gfx::CommandBuffer> BackEnd
for GraphicsBackEnd<'a, C> {
    type Texture = Texture;

    fn clear(&mut self, color: [f32; 4]) {
        let &mut GraphicsBackEnd {
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

    fn tri_list<F>(&mut self, color: &[f32; 4], mut f: F)
        where F: FnMut(&mut FnMut(&[f32]))
    {
        let &mut GraphicsBackEnd {
            ref mut renderer,
            ref frame,
            g2d: &mut G2D {
                ref mut buffer_pos,
                ref mut batch,
                ..
            },
        } = self;

        batch.param.color = *color;

        f(&mut |vertices: &[f32]| {
            renderer.update_buffer_vec(*buffer_pos, vertices, 0);

            let n = vertices.len() / POS_COMPONENTS;
            batch.slice = gfx::Slice {
                    prim_type: gfx::PrimitiveType::TriangleList,
                    start: 0,
                    end: n as u32,
                    kind: gfx::SliceKind::Vertex
                };
            let _ = renderer.draw(batch, *frame);
        })
    }

    fn tri_list_uv<F>(&mut self, color: &[f32; 4], texture: &Texture, mut f: F)
        where F: FnMut(&mut FnMut(&[f32], &[f32]))
    {
        let &mut GraphicsBackEnd {
            ref mut renderer,
            ref frame,
            g2d: &mut G2D {
                ref mut buffer_pos,
                ref mut buffer_uv,
                ref mut batch_uv,
                ..
            },
        } = self;

        batch_uv.param.s_texture.0 = texture.handle;
        batch_uv.param.color = *color;

        f(&mut |vertices: &[f32], texture_coords: &[f32]| {
            assert_eq!(
                vertices.len() * UV_COMPONENTS,
                texture_coords.len() * POS_COMPONENTS
            );
            renderer.update_buffer_vec(*buffer_pos, vertices, 0);
            renderer.update_buffer_vec(*buffer_uv, texture_coords, 0);

            let n = vertices.len() / POS_COMPONENTS;
            batch_uv.slice = gfx::Slice {
                    prim_type: gfx::PrimitiveType::TriangleList,
                    start: 0,
                    end: n as u32,
                    kind: gfx::SliceKind::Vertex
                };
            let _ = renderer.draw(batch_uv, *frame);
        })
    }
}
