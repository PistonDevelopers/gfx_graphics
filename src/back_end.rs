use graphics::{ Context, DrawState, Graphics, Viewport };
use graphics::BACK_END_MAX_VERTEX_COUNT as BUFFER_SIZE;
use { gfx, Texture };
use gfx::format::{DepthStencil, Rgba8};
use shader_version::{ OpenGL, Shaders };
use shader_version::glsl::GLSL;

const POS_COMPONENTS: usize = 2;
const UV_COMPONENTS: usize = 2;

// Boiler plate for automatic attribute construction.
// Needs to be improved on gfx-rs side.
// For some reason, using ``*_COMPONENT` triggers some macros errors.

gfx_vertex_struct!( PositionFormat {
    pos: [f32; 2] = "pos",
});

gfx_vertex_struct!( ColorFormat {
    color: [f32; 4] = "color",
});

gfx_vertex_struct!( TexCoordsFormat {
    uv: [f32; 2] = "uv",
});

gfx_pipeline!( pipe_colored {
    pos: gfx::VertexBuffer<PositionFormat> = (),
    color: gfx::Global<[f32; 4]> = "color",
    out_color: gfx::RenderTarget<gfx::format::Rgba8> = "o_Color",
});

gfx_pipeline!( pipe_textured {
    pos: gfx::VertexBuffer<PositionFormat> = (),
    uv: gfx::VertexBuffer<TexCoordsFormat> = (),
    color: gfx::Global<[f32; 4]> = "color",
    texture: gfx::TextureSampler<[f32; 4]> = "s_texture",
    out_color: gfx::RenderTarget<gfx::format::Rgba8> = "o_Color",
});

/// The data used for drawing 2D graphics.
pub struct Gfx2d<R: gfx::Resources> {
    buffer_pos: gfx::handle::Buffer<R, PositionFormat>,
    buffer_uv: gfx::handle::Buffer<R, TexCoordsFormat>,
    pso_colored: gfx::pso::PipelineState<R, pipe_colored::Meta>,
    pso_textured: gfx::pso::PipelineState<R, pipe_textured::Meta>,
    sampler: gfx::handle::Sampler<R>,
}

impl<R: gfx::Resources> Gfx2d<R> {
    /// Creates a new Gfx2d object.
    pub fn new<F>(opengl: OpenGL, factory: &mut F) -> Self
        where F: gfx::Factory<R>
    {
        use gfx::traits::*;
        use shaders::{ colored, textured };

        let glsl = opengl.to_glsl();

        let pso_colored = factory.create_pipeline_simple(
            Shaders::new()
                .set(GLSL::V1_20, colored::VERTEX_GLSL_120)
                .set(GLSL::V1_50, colored::VERTEX_GLSL_150_CORE)
                .get(glsl).unwrap(),
            Shaders::new()
                .set(GLSL::V1_20, colored::FRAGMENT_GLSL_120)
                .set(GLSL::V1_50, colored::FRAGMENT_GLSL_150_CORE)
                .get(glsl).unwrap(),
            gfx::state::CullFace::Nothing,
            pipe_colored::new()
        ).unwrap();

        let pso_textured = factory.create_pipeline_simple(
            Shaders::new()
                .set(GLSL::V1_20, textured::VERTEX_GLSL_120)
                .set(GLSL::V1_50, textured::VERTEX_GLSL_150_CORE)
                .get(glsl).unwrap(),
            Shaders::new()
                .set(GLSL::V1_20, textured::FRAGMENT_GLSL_120)
                .set(GLSL::V1_50, textured::FRAGMENT_GLSL_150_CORE)
                .get(glsl).unwrap(),
            gfx::state::CullFace::Nothing,
            pipe_textured::new()
        ).unwrap();

        let buffer_pos = factory.create_buffer_dynamic(
            POS_COMPONENTS * BUFFER_SIZE,
            gfx::BufferRole::Vertex
        );
        let buffer_uv = factory.create_buffer_dynamic(
            UV_COMPONENTS * BUFFER_SIZE,
            gfx::BufferRole::Vertex
        );

        let sampler_info = gfx::tex::SamplerInfo::new(
            gfx::tex::FilterMethod::Bilinear,
            gfx::tex::WrapMode::Clamp
        );
        let sampler = factory.create_sampler(sampler_info);

        Gfx2d {
            buffer_pos: buffer_pos,
            buffer_uv: buffer_uv,
            pso_colored: pso_colored,
            pso_textured: pso_textured,
            sampler: sampler
        }
    }

    /// Renders graphics to a Gfx renderer.
    pub fn draw<C, F>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        output_color: &gfx::handle::RenderTargetView<R, Rgba8>,
        output_stencil: &gfx::handle::DepthStencilView<R, DepthStencil>,
        viewport: Viewport,
        f: F
    )
        where C: gfx::CommandBuffer<R>,
              F: FnOnce(Context, &mut GfxGraphics<R, C>)
    {
        let ref mut g = GfxGraphics::new(
            encoder,
            output_color,
            output_stencil,
            self
        );
        let c = Context::new_viewport(viewport);
        f(c, g);
    }
}

/// Used for rendering 2D graphics.
pub struct GfxGraphics<'a, R, C>
    where R: gfx::Resources + 'a,
          C: gfx::CommandBuffer<R> + 'a,
          R::Buffer: 'a,
          R::Shader: 'a,
          R::Program: 'a,
          R::Texture: 'a,
          R::Sampler: 'a
{
    encoder: &'a mut gfx::Encoder<R, C>,
    output_color: &'a gfx::handle::RenderTargetView<R, Rgba8>,
    output_stencil: &'a gfx::handle::DepthStencilView<R, DepthStencil>,
    g2d: &'a mut Gfx2d<R>,
}

impl<'a, R, C> GfxGraphics<'a, R, C>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
{
    /// Creates a new object for rendering 2D graphics.
    pub fn new(encoder: &'a mut gfx::Encoder<R, C>,
               output_color: &'a gfx::handle::RenderTargetView<R, Rgba8>,
               output_stencil: &'a gfx::handle::DepthStencilView<R, DepthStencil>,
               g2d: &'a mut Gfx2d<R>) -> Self {
        GfxGraphics {
            encoder: encoder,
            output_color: output_color,
            output_stencil: output_stencil,
            g2d: g2d,
        }
    }

    /// Returns true if texture has alpha channel.
    pub fn has_texture_alpha(&self, texture: &Texture<R>) -> bool {
        use gfx::format::SurfaceType::*;

        match texture.surface.get_info().format {
            R4_G4_B4_A4
            | R5_G5_B5_A1
            | R8_G8_B8_A8
            | R10_G10_B10_A2
            | R16_G16_B16_A16
            | R32_G32_B32_A32 => true,
            R3_G3_B2
            | R4_G4
            | R5_G6_B5
            | R8 | R8_G8 | R8_G8_B8
            | R11_G11_B10
            | R16 | R16_G16 | R16_G16_B16
            | R32 | R32_G32 | R32_G32_B32
            | D16 | D24 | D24_S8 | D32 => false,
        }
    }
}

impl<'a, R, C> Graphics for GfxGraphics<'a, R, C>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
          R::Buffer: 'a,
          R::Shader: 'a,
          R::Program: 'a,
          R::Texture: 'a,
          R::Sampler: 'a
{
    type Texture = Texture<R>;

    fn clear_color(&mut self, color: [f32; 4]) {
        let &mut GfxGraphics {
            ref mut encoder,
            output_color,
            ..
        } = self;
        encoder.clear(output_color, color);
    }

    fn clear_stencil(&mut self, value: u8) {
        let &mut GfxGraphics {
            ref mut encoder,
            output_stencil,
            ..
        } = self;
        encoder.clear_stencil(output_stencil, value);
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
            ref mut encoder,
            output_color,
            g2d: &mut Gfx2d {
                ref mut buffer_pos,
                ref mut pso_colored,
                ..
            },
            ..
        } = self;

        // TODO: Update draw state.

        f(&mut |vertices: &[f32]| {
            use std::mem::transmute;

            unsafe {
                encoder.update_buffer(&buffer_pos, transmute(vertices), 0)
                    .unwrap();
            }

            let data = pipe_colored::Data {
                pos: buffer_pos.clone(),
                color: *color,
                out_color: output_color.clone(),
            };

            let n = vertices.len() / POS_COMPONENTS;
            let slice = gfx::Slice {
                    instances: None,
                    start: 0,
                    end: n as u32,
                    kind: gfx::SliceKind::Vertex
            };
            encoder.draw(&slice, pso_colored, &data);
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
            ref mut encoder,
            output_color,
            g2d: &mut Gfx2d {
                ref mut buffer_pos,
                ref mut buffer_uv,
                ref mut pso_textured,
                ref sampler,
                ..
            },
            ..
        } = self;

        // TODO: Update draw state.

        f(&mut |vertices: &[f32], texture_coords: &[f32]| {
            use std::mem::transmute;

            assert_eq!(
                vertices.len() * UV_COMPONENTS,
                texture_coords.len() * POS_COMPONENTS
            );
            unsafe {
                encoder.update_buffer(&buffer_pos, transmute(vertices), 0)
                    .unwrap();
                encoder.update_buffer(&buffer_uv, transmute(texture_coords), 0)
                    .unwrap();
            }

            let data = pipe_textured::Data {
                pos: buffer_pos.clone(),
                uv: buffer_uv.clone(),
                color: *color,
                texture: (texture.view.clone(), sampler.clone()),
                out_color: output_color.clone(),
            };

            let n = vertices.len() / POS_COMPONENTS;
            let slice = gfx::Slice {
                    instances: None,
                    start: 0,
                    end: n as u32,
                    kind: gfx::SliceKind::Vertex
            };
            encoder.draw(&slice, pso_textured, &data);
        })
    }
}
