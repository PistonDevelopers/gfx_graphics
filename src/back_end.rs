use std::marker::PhantomData;
use graphics::{ Context, DrawState, Graphics, Viewport };
use graphics::BACK_END_MAX_VERTEX_COUNT as BUFFER_SIZE;
use { gfx, Texture };

const POS_COMPONENTS: usize = 2;
const UV_COMPONENTS: usize = 2;

// Boiler plate for automatic attribute construction.
// Needs to be improved on gfx-rs side.
// For some reason, using ``*_COMPONENT` triggers some macros errors.

gfx_vertex!( PositionFormat {
    pos@ pos: [f32; 2],
});

gfx_vertex!( ColorFormat {
    color@ color: [f32; 4],
});

gfx_vertex!( TexCoordsFormat {
    uv@ uv: [f32; 2],
});

gfx_parameters!( Params {
    color@ color: [f32; 4],
});

gfx_parameters!( ParamsUV {
    color@ color: [f32; 4],
    s_texture@ texture: gfx::shade::TextureParam<R>,
});


/// The data used for drawing 2D graphics.
pub struct Gfx2d<R: gfx::Resources> {
    buffer_pos: gfx::handle::Buffer<R, f32>,
    buffer_uv: gfx::handle::Buffer<R, f32>,
    batch: gfx::batch::Full<Params<R>>,
    batch_uv: gfx::batch::Full<ParamsUV<R>>,
}

impl<R: gfx::Resources> Gfx2d<R> {
    /// Creates a new Gfx2d object.
    pub fn new<F>(factory: &mut F) -> Self
        where F: gfx::Factory<R>
    {
        use gfx::traits::*;
        use gfx::VertexFormat;
        use shaders::{ colored, textured };

        let program = {
            let vertex = gfx::ShaderSource {
                glsl_120: Some(colored::VERTEX_GLSL_120),
                glsl_150: Some(colored::VERTEX_GLSL_150_CORE),
                .. gfx::ShaderSource::empty()
            };
            let fragment = gfx::ShaderSource {
                glsl_120: Some(colored::FRAGMENT_GLSL_120),
                glsl_150: Some(colored::FRAGMENT_GLSL_150_CORE),
                .. gfx::ShaderSource::empty()
            };
            factory.link_program_source(
                vertex,
                fragment
            ).unwrap()
        };

        let program_uv = {
            let vertex = gfx::ShaderSource {
                glsl_120: Some(textured::VERTEX_GLSL_120),
                glsl_150: Some(textured::VERTEX_GLSL_150_CORE),
                .. gfx::ShaderSource::empty()
            };
            let fragment = gfx::ShaderSource {
                glsl_120: Some(textured::FRAGMENT_GLSL_120),
                glsl_150: Some(textured::FRAGMENT_GLSL_150_CORE),
                .. gfx::ShaderSource::empty()
            };
            factory.link_program_source(
                vertex,
                fragment
            ).unwrap()
        };

        let buffer_pos = factory.create_buffer_dynamic(
            POS_COMPONENTS * BUFFER_SIZE,
            gfx::BufferRole::Vertex
        );
        let buffer_uv = factory.create_buffer_dynamic(
            UV_COMPONENTS * BUFFER_SIZE,
            gfx::BufferRole::Vertex
        );

        let mut mesh = gfx::Mesh::new(BUFFER_SIZE as u32);
        mesh.attributes.extend(
            PositionFormat::generate(&buffer_pos)
        );

        // Reuse parameters from `mesh`.
        let mut mesh_uv = mesh.clone();
        mesh_uv.attributes.extend(
            TexCoordsFormat::generate(&buffer_uv)
        );

        let params = Params {
            color: [1.0; 4],
            _r: PhantomData,
        };
        let mut batch = gfx::batch::Full::new(
            mesh,
            program,
            params
        ).unwrap();

        let sampler = factory.create_sampler(
            gfx::tex::SamplerInfo::new(
                gfx::tex::FilterMethod::Trilinear,
                gfx::tex::WrapMode::Clamp
            )
        );

        let tex_handle = Texture::empty(factory).unwrap().handle();

        let params_uv = ParamsUV {
            color: [1.0; 4],
            texture: (tex_handle, Some(sampler)),
            _r: PhantomData,
        };
        let mut batch_uv = gfx::batch::Full::new(
            mesh_uv,
            program_uv,
            params_uv
        ).unwrap();

        // Disable culling.
        batch.state.primitive.method =
            gfx::state::RasterMethod::Fill(gfx::state::CullFace::Nothing);
        batch_uv.state.primitive.method =
            gfx::state::RasterMethod::Fill(gfx::state::CullFace::Nothing);

        Gfx2d {
            buffer_pos: buffer_pos.cast(),
            buffer_uv: buffer_uv.cast(),
            batch: batch,
            batch_uv: batch_uv,
        }
    }

    /// Renders graphics to a Gfx renderer.
    pub fn draw<C, O, F>(
        &mut self,
        renderer: &mut gfx::Renderer<R, C>,
        output: &O,
        viewport: Viewport,
        mut f: F
    )
        where C: gfx::CommandBuffer<R>,
              O: gfx::Output<R>,
              F: FnMut(Context, &mut GfxGraphics<R, C, O>)
    {
        let ref mut g = GfxGraphics::new(
            renderer,
            output,
            self
        );
        let c = Context::new_viewport(viewport);
        f(c, g);
    }
}

/// Used for rendering 2D graphics.
pub struct GfxGraphics<'a, R, C, O>
    where R: gfx::Resources + 'a,
          C: gfx::CommandBuffer<R> + 'a,
          O: gfx::Output<R> + 'a,
          R::Buffer: 'a,
          R::ArrayBuffer: 'a,
          R::Shader: 'a,
          R::Program: 'a,
          R::FrameBuffer: 'a,
          R::Surface: 'a,
          R::Texture: 'a,
          R::Sampler: 'a
{
    renderer: &'a mut gfx::Renderer<R, C>,
    output: &'a O,
    g2d: &'a mut Gfx2d<R>,
}

impl<'a, R, C, O> GfxGraphics<'a, R, C, O>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
          O: gfx::Output<R>
{
    /// Creates a new object for rendering 2D graphics.
    pub fn new(renderer: &'a mut gfx::Renderer<R, C>,
               output: &'a O,
               g2d: &'a mut Gfx2d<R>) -> Self {
        GfxGraphics {
            renderer: renderer,
            output: output,
            g2d: g2d,
        }
    }

    /// Returns true if texture has alpha channel.
    pub fn has_texture_alpha(&self, texture: &Texture<R>) -> bool {
        use gfx::tex::Components::RGBA;

        texture.handle().get_info().format.get_components() == Some(RGBA)
    }
}

impl<'a, R, C, O> Graphics for GfxGraphics<'a, R, C, O>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
          O: gfx::Output<R>,
          R::Buffer: 'a,
          R::ArrayBuffer: 'a,
          R::Shader: 'a,
          R::Program: 'a,
          R::FrameBuffer: 'a,
          R::Surface: 'a,
          R::Texture: 'a,
          R::Sampler: 'a
{
    type Texture = Texture<R>;

    fn clear_color(&mut self, color: [f32; 4]) {
        let &mut GfxGraphics {
            ref mut renderer,
            output,
            ..
        } = self;
        renderer.clear(
            gfx::ClearData {
                color: color,
                depth: 0.0,
                stencil: 0,
            },
            gfx::COLOR,
            output
        );
    }

    fn clear_stencil(&mut self, value: u8) {
        let &mut GfxGraphics {
            ref mut renderer,
            output,
            ..
        } = self;
        renderer.clear(
            gfx::ClearData {
                color: [0.0; 4],
                depth: 0.0,
                stencil: value,
            },
            gfx::STENCIL,
            output
        );
    }

    fn tri_list<F>(
        &mut self,
        draw_state: &DrawState,
        color: &[f32; 4],
        mut f: F
    )
        where F: FnMut(&mut FnMut(&[f32]))
    {
        let &mut GfxGraphics {
            ref mut renderer,
            ref output,
            g2d: &mut Gfx2d {
                ref mut buffer_pos,
                ref mut batch,
                ..
            },
        } = self;

        batch.state = *draw_state;
        batch.params.color = *color;

        f(&mut |vertices: &[f32]| {
            renderer.update_buffer(&buffer_pos.raw(), vertices, 0).unwrap();

            let n = vertices.len() / POS_COMPONENTS;
            batch.slice = gfx::Slice {
                    prim_type: gfx::PrimitiveType::TriangleList,
                    start: 0,
                    end: n as u32,
                    kind: gfx::SliceKind::Vertex
            };
            let _ = renderer.draw(batch, None, *output);
        })
    }

    fn tri_list_uv<F>(
        &mut self,
        draw_state: &DrawState,
        color: &[f32; 4],
        texture: &<Self as Graphics>::Texture,
        mut f: F
    )
        where F: FnMut(&mut FnMut(&[f32], &[f32]))
    {
        let &mut GfxGraphics {
            ref mut renderer,
            ref output,
            g2d: &mut Gfx2d {
                ref mut buffer_pos,
                ref mut buffer_uv,
                ref mut batch_uv,
                ..
            },
        } = self;

        batch_uv.state = *draw_state;
        batch_uv.params.texture.0 = texture.handle();
        batch_uv.params.color = *color;

        f(&mut |vertices: &[f32], texture_coords: &[f32]| {
            assert_eq!(
                vertices.len() * UV_COMPONENTS,
                texture_coords.len() * POS_COMPONENTS
            );
            renderer.update_buffer(&buffer_pos.raw(), vertices, 0).unwrap();
            renderer.update_buffer(&buffer_uv.raw(), texture_coords, 0).unwrap();

            let n = vertices.len() / POS_COMPONENTS;
            batch_uv.slice = gfx::Slice {
                    prim_type: gfx::PrimitiveType::TriangleList,
                    start: 0,
                    end: n as u32,
                    kind: gfx::SliceKind::Vertex
            };
            let _ = renderer.draw(batch_uv, None, *output);
        })
    }
}
