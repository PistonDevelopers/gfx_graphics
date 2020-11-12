extern crate gfx;

use graphics::{ Context, DrawState, Graphics, Viewport };
use graphics::BACK_END_MAX_VERTEX_COUNT as BUFFER_SIZE;
use graphics::draw_state;
use graphics::color::gamma_srgb_to_linear;
use Texture;
use gfx::format::{DepthStencil, Srgba8};
use gfx::pso::PipelineState;
use shader_version::{ OpenGL, Shaders };
use shader_version::glsl::GLSL;

// The number of chunks to fill up before rendering.
// Amount of memory used: `BUFFER_SIZE * CHUNKS * 4 * (2 + 4)`
// `4` for bytes per f32, and `2 + 4` for position and color.
const CHUNKS: usize = 100;

gfx_defines! {
    vertex PositionFormat {
        pos: [f32; 2] = "pos",
    }

    vertex ColorFormat {
        color: [f32; 4] = "color",
    }

    vertex TexCoordsFormat {
        uv: [f32; 2] = "uv",
    }
}

gfx_pipeline_base!( pipe_colored {
    pos: gfx::VertexBuffer<PositionFormat>,
    color: gfx::VertexBuffer<ColorFormat>,
    blend_target: gfx::BlendTarget<gfx::format::Srgba8>,
    stencil_target: gfx::StencilTarget<gfx::format::DepthStencil>,
    blend_ref: gfx::BlendRef,
    scissor: gfx::Scissor,
});

gfx_pipeline_base!( pipe_textured {
    pos: gfx::VertexBuffer<PositionFormat>,
    uv: gfx::VertexBuffer<TexCoordsFormat>,
    color: gfx::Global<[f32; 4]>,
    texture: gfx::TextureSampler<[f32; 4]>,
    blend_target: gfx::BlendTarget<gfx::format::Srgba8>,
    stencil_target: gfx::StencilTarget<gfx::format::DepthStencil>,
    blend_ref: gfx::BlendRef,
    scissor: gfx::Scissor,
});

gfx_pipeline_base!( pipe_textured_color {
    pos: gfx::VertexBuffer<PositionFormat>,
    uv: gfx::VertexBuffer<TexCoordsFormat>,
    color: gfx::VertexBuffer<ColorFormat>,
    texture: gfx::TextureSampler<[f32; 4]>,
    blend_target: gfx::BlendTarget<gfx::format::Srgba8>,
    stencil_target: gfx::StencilTarget<gfx::format::DepthStencil>,
    blend_ref: gfx::BlendRef,
    scissor: gfx::Scissor,
});

// Stores one PSO per blend setting.
struct PsoBlend<T> {
    alpha: T,
    add: T,
    multiply: T,
    invert: T,
    none: T,
    lighter: T,
}

impl<T> PsoBlend<T> {
    fn blend(&mut self, blend: Option<draw_state::Blend>) -> &mut T {
        use graphics::draw_state::Blend;

        match blend {
            Some(Blend::Alpha) => &mut self.alpha,
            Some(Blend::Add) => &mut self.add,
            Some(Blend::Multiply) => &mut self.multiply,
            Some(Blend::Invert) => &mut self.invert,
            Some(Blend::Lighter) => &mut self.lighter,
            None => &mut self.none,
        }
    }
}

// Stores one `PsoBlend` per clip setting.
struct PsoStencil<T> {
    none: PsoBlend<T>,
    clip: PsoBlend<T>,
    inside: PsoBlend<T>,
    outside: PsoBlend<T>,
    increment: PsoBlend<T>,
}

impl<T> PsoStencil<T> {
    fn new<Fact, F>(factory: &mut Fact, f: F) -> PsoStencil<T>
        where F: Fn(
            &mut Fact,
            gfx::state::Blend,
            gfx::state::Stencil,
            gfx::state::ColorMask
        ) -> T
    {
        use gfx::state::{Blend, BlendChannel, Comparison, Equation, Factor,
            Stencil, StencilOp};
        use gfx::preset::blend;

        let stencil = Stencil::new(Comparison::Always, 0,
            (StencilOp::Keep, StencilOp::Keep, StencilOp::Keep));
        let stencil_clip = Stencil::new(Comparison::Never, 255,
            (StencilOp::Replace, StencilOp::Keep, StencilOp::Keep));
        let stencil_inside = Stencil::new(Comparison::Equal, 255,
            (StencilOp::Keep, StencilOp::Keep, StencilOp::Keep));
        let stencil_outside = Stencil::new(Comparison::NotEqual, 255,
            (StencilOp::Keep, StencilOp::Keep, StencilOp::Keep));
        let stencil_increment = Stencil::new(Comparison::Never, 255,
            (StencilOp::IncrementClamp, StencilOp::Keep, StencilOp::Keep));

        // Channel color masks.
        let mask_all = gfx::state::ColorMask::all();
        let mask_none = gfx::state::ColorMask::empty();

        // Fake disabled blending using the same pipeline.
        let no_blend = Blend {
            color: BlendChannel {
                equation: Equation::Add,
                source: Factor::One,
                destination: Factor::Zero,
            },
            alpha: BlendChannel {
                equation: Equation::Add,
                source: Factor::One,
                destination: Factor::Zero,
            },
        };

        use gfx::state::BlendValue;

        const BLEND_LIGHTER: Blend = Blend {
                    color: BlendChannel {
                        equation: Equation::Add,
                        source: Factor::ZeroPlus(BlendValue::SourceAlpha),
                        destination: Factor::One,
                    },
                    alpha: BlendChannel {
                        equation: Equation::Add,
                        source: Factor::Zero,
                        destination: Factor::One,
                    },
                };

        PsoStencil {
            none: PsoBlend {
                alpha: f(factory, blend::ALPHA, stencil, mask_all),
                add: f(factory, blend::ADD, stencil, mask_all),
                multiply: f(factory, blend::MULTIPLY, stencil, mask_all),
                invert: f(factory, blend::INVERT, stencil, mask_all),
                lighter: f(factory, BLEND_LIGHTER, stencil, mask_all),
                none: f(factory, no_blend, stencil, mask_all),
            },
            clip: PsoBlend {
                alpha: f(factory, blend::ALPHA, stencil_clip, mask_none),
                add: f(factory, blend::ADD, stencil_clip, mask_none),
                multiply: f(factory, blend::MULTIPLY, stencil_clip, mask_none),
                invert: f(factory, blend::INVERT, stencil_clip, mask_none),
                lighter: f(factory, BLEND_LIGHTER, stencil_clip, mask_none),
                none: f(factory, no_blend, stencil_clip, mask_none),
            },
            inside: PsoBlend {
                alpha: f(factory, blend::ALPHA, stencil_inside, mask_all),
                add: f(factory, blend::ADD, stencil_inside, mask_all),
                multiply: f(factory, blend::MULTIPLY, stencil_inside, mask_all),
                invert: f(factory, blend::INVERT, stencil_inside, mask_all),
                lighter: f(factory, BLEND_LIGHTER, stencil_inside, mask_all),
                none: f(factory, no_blend, stencil_inside, mask_all),
            },
            outside: PsoBlend {
                alpha: f(factory, blend::ALPHA, stencil_outside, mask_all),
                add: f(factory, blend::ADD, stencil_outside, mask_all),
                multiply: f(factory, blend::MULTIPLY, stencil_outside, mask_all),
                invert: f(factory, blend::INVERT, stencil_outside, mask_all),
                lighter: f(factory, BLEND_LIGHTER, stencil_outside, mask_all),
                none: f(factory, no_blend, stencil_outside, mask_all),
            },
            increment: PsoBlend {
                alpha: f(factory, blend::ALPHA, stencil_increment, mask_all),
                add: f(factory, blend::ADD, stencil_increment, mask_all),
                multiply: f(factory, blend::MULTIPLY, stencil_increment, mask_all),
                invert: f(factory, blend::INVERT, stencil_increment, mask_all),
                lighter: f(factory, BLEND_LIGHTER, stencil_increment, mask_all),
                none: f(factory, no_blend, stencil_increment, mask_all),
            },
        }
    }

    // Returns a PSO and stencil reference given a stencil and blend setting.
    fn stencil_blend(
        &mut self,
        stencil: Option<draw_state::Stencil>,
        blend: Option<draw_state::Blend>
    ) -> (&mut T, u8) {
        use graphics::draw_state::Stencil;

        match stencil {
            None => (self.none.blend(blend), 0),
            Some(Stencil::Clip(val)) => (self.clip.blend(blend), val),
            Some(Stencil::Inside(val)) => (self.inside.blend(blend), val),
            Some(Stencil::Outside(val)) => (self.outside.blend(blend), val),
            Some(Stencil::Increment) => (self.increment.blend(blend), 0),
        }
    }
}

/// The data used for drawing 2D graphics.
///
/// Stores buffers and PSO objects needed for rendering 2D graphics.
pub struct Gfx2d<R: gfx::Resources> {
    // The offset in vertices for colored rendering.
    colored_offset: usize,
    // The current draw state for colored rendering.
    colored_draw_state: DrawState,
    buffer_pos: gfx::handle::Buffer<R, PositionFormat>,
    buffer_color: gfx::handle::Buffer<R, ColorFormat>,
    buffer_uv: gfx::handle::Buffer<R, TexCoordsFormat>,
    colored: PsoStencil<PipelineState<R, pipe_colored::Meta>>,
    textured: PsoStencil<PipelineState<R, pipe_textured::Meta>>,
    textured_color: PsoStencil<PipelineState<R, pipe_textured_color::Meta>>,
}

impl<R: gfx::Resources> Gfx2d<R> {
    /// Creates a new Gfx2d object.
    pub fn new<F>(opengl: OpenGL, factory: &mut F) -> Self
        where F: gfx::Factory<R>
    {
        use gfx::Primitive;
        use gfx::state::Rasterizer;
        use gfx::state::{Blend, Stencil};
        use gfx::traits::*;
        use shaders::{ colored, textured, textured_color };

        let glsl = opengl.to_glsl();

        let colored_program = factory.link_program(
                Shaders::new()
                    .set(GLSL::V1_20, colored::VERTEX_GLSL_120)
                    .set(GLSL::V1_50, colored::VERTEX_GLSL_150_CORE)
                    .get(glsl).unwrap(),
                Shaders::new()
                    .set(GLSL::V1_20, colored::FRAGMENT_GLSL_120)
                    .set(GLSL::V1_50, colored::FRAGMENT_GLSL_150_CORE)
                    .get(glsl).unwrap(),
            ).unwrap();

        let colored_pipeline = |factory: &mut F,
                                blend_preset: Blend,
                                stencil: Stencil,
                                color_mask: gfx::state::ColorMask|
        -> PipelineState<R, pipe_colored::Meta> {
            factory.create_pipeline_from_program(
                &colored_program,
                Primitive::TriangleList,
                Rasterizer::new_fill(),
                pipe_colored::Init {
                    pos: (),
                    color: (),
                    blend_target: ("o_Color", color_mask, blend_preset),
                    stencil_target: stencil,
                    blend_ref: (),
                    scissor: (),
                }
            ).unwrap()
        };

        let colored = PsoStencil::new(factory, colored_pipeline);

        let textured_program = factory.link_program(
                Shaders::new()
                    .set(GLSL::V1_20, textured::VERTEX_GLSL_120)
                    .set(GLSL::V1_50, textured::VERTEX_GLSL_150_CORE)
                    .get(glsl).unwrap(),
                Shaders::new()
                    .set(GLSL::V1_20, textured::FRAGMENT_GLSL_120)
                    .set(GLSL::V1_50, textured::FRAGMENT_GLSL_150_CORE)
                    .get(glsl).unwrap()
            ).unwrap();

        let textured_pipeline = |factory: &mut F,
                                 blend_preset: Blend,
                                 stencil: Stencil,
                                 color_mask: gfx::state::ColorMask|
        -> PipelineState<R, pipe_textured::Meta> {
            factory.create_pipeline_from_program(
                &textured_program,
                Primitive::TriangleList,
                Rasterizer::new_fill(),
                pipe_textured::Init {
                    pos: (),
                    uv: (),
                    color: "color",
                    texture: "s_texture",
                    blend_target: ("o_Color", color_mask, blend_preset),
                    stencil_target: stencil,
                    blend_ref: (),
                    scissor: (),
                }
            ).unwrap()
        };

        let textured = PsoStencil::new(factory, textured_pipeline);

        let textured_color_program = factory.link_program(
                Shaders::new()
                    .set(GLSL::V1_20, textured_color::VERTEX_GLSL_120)
                    .set(GLSL::V1_50, textured_color::VERTEX_GLSL_150_CORE)
                    .get(glsl).unwrap(),
                Shaders::new()
                    .set(GLSL::V1_20, textured_color::FRAGMENT_GLSL_120)
                    .set(GLSL::V1_50, textured_color::FRAGMENT_GLSL_150_CORE)
                    .get(glsl).unwrap()
            ).unwrap();

        let textured_color_pipeline = |factory: &mut F,
                                 blend_preset: Blend,
                                 stencil: Stencil,
                                 color_mask: gfx::state::ColorMask|
        -> PipelineState<R, pipe_textured_color::Meta> {
            factory.create_pipeline_from_program(
                &textured_color_program,
                Primitive::TriangleList,
                Rasterizer::new_fill(),
                pipe_textured_color::Init {
                    pos: (),
                    uv: (),
                    color: (),
                    texture: "s_texture",
                    blend_target: ("o_Color", color_mask, blend_preset),
                    stencil_target: stencil,
                    blend_ref: (),
                    scissor: (),
                }
            ).unwrap()
        };

        let textured_color = PsoStencil::new(factory, textured_color_pipeline);

        let buffer_pos = factory.create_buffer(
            BUFFER_SIZE * CHUNKS,
            gfx::buffer::Role::Vertex,
            gfx::memory::Usage::Dynamic,
            gfx::memory::Bind::empty()
        ).expect("Could not create `buffer_pos`");
        let buffer_color = factory.create_buffer(
            BUFFER_SIZE * CHUNKS,
            gfx::buffer::Role::Vertex,
            gfx::memory::Usage::Dynamic,
            gfx::memory::Bind::empty()
        ).expect("Could not create `buffer_color`");
        let buffer_uv = factory.create_buffer(
            BUFFER_SIZE,
            gfx::buffer::Role::Vertex,
            gfx::memory::Usage::Dynamic,
            gfx::memory::Bind::empty()
        ).expect("Could not create `buffer_uv`");

        Gfx2d {
            colored_offset: 0,
            colored_draw_state: Default::default(),
            buffer_pos: buffer_pos,
            buffer_color: buffer_color,
            buffer_uv: buffer_uv,
            colored: colored,
            textured: textured,
            textured_color: textured_color,
        }
    }

    /// Renders graphics to a Gfx renderer.
    pub fn draw<C, F, U>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        output_color: &gfx::handle::RenderTargetView<R, Srgba8>,
        output_stencil: &gfx::handle::DepthStencilView<R, DepthStencil>,
        viewport: Viewport,
        f: F
    ) -> U
        where C: gfx::CommandBuffer<R>,
              F: FnOnce(Context, &mut GfxGraphics<R, C>) -> U
    {
        let ref mut g = GfxGraphics::new(
            encoder,
            output_color,
            output_stencil,
            self
        );
        let c = Context::new_viewport(viewport);
        let res = f(c, g);
        if g.g2d.colored_offset > 0 {
            g.flush_colored();
        }
        res
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
    /// Provide access to the `gfx::Encoder` in case a user needs to update textures for caching,
    /// etc.
    pub encoder: &'a mut gfx::Encoder<R, C>,
    output_color: &'a gfx::handle::RenderTargetView<R, Srgba8>,
    output_stencil: &'a gfx::handle::DepthStencilView<R, DepthStencil>,
    g2d: &'a mut Gfx2d<R>,
}

impl<'a, R, C> GfxGraphics<'a, R, C>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
{
    /// Creates a new object for rendering 2D graphics.
    pub fn new(encoder: &'a mut gfx::Encoder<R, C>,
               output_color: &'a gfx::handle::RenderTargetView<R, Srgba8>,
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
    pub fn has_texture_alpha(&self, texture: &Texture<R>) -> bool
        where R: gfx::Resources
    {
        texture.surface.get_info().format.get_alpha_stencil_bits() > 0
    }

    fn flush_colored(&mut self) {
        use draw_state::target::Rect;
        use std::u16;

        let &mut GfxGraphics {
            ref mut encoder,
            output_color,
            output_stencil,
            g2d: &mut Gfx2d {
                ref mut colored_offset,
                ref mut colored_draw_state,
                ref mut buffer_pos,
                ref mut buffer_color,
                ref mut colored,
                ..
            },
            ..
        } = self;

        let (pso_colored, stencil_val) = colored.stencil_blend(
            colored_draw_state.stencil,
            colored_draw_state.blend
        );

        let scissor = match colored_draw_state.scissor {
            None => Rect { x: 0, y: 0, w: u16::MAX, h: u16::MAX },
            Some(r) => Rect { x: r[0] as u16, y: r[1] as u16,
                w: r[2] as u16, h: r[3] as u16 }
        };

        let data = pipe_colored::Data {
            pos: buffer_pos.clone(),
            color: buffer_color.clone(),
            blend_target: output_color.clone(),
            stencil_target: (output_stencil.clone(),
                             (stencil_val, stencil_val)),
            // Use white color for blend reference to make invert work.
            blend_ref: [1.0; 4],
            scissor: scissor,
        };

        let slice = gfx::Slice {
            instances: None,
            start: 0,
            end: *colored_offset as u32,
            buffer: gfx::IndexBuffer::Auto,
            base_vertex: 0,
        };
        encoder.draw(&slice, pso_colored, &data);
        *colored_offset = 0;
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
        let color = gamma_srgb_to_linear(color);
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
        where F: FnMut(&mut dyn FnMut(&[[f32; 2]]))
    {
        let color = gamma_srgb_to_linear(*color);

        // Flush when draw state changes.
        if &self.g2d.colored_draw_state != draw_state {
            self.flush_colored();
            self.g2d.colored_draw_state = *draw_state;
        }
        f(&mut |vertices: &[[f32; 2]]| {
            let n = vertices.len();

            // Render if there is not enough room.
            if self.g2d.colored_offset + n > BUFFER_SIZE * CHUNKS {
                self.flush_colored();
            }

            {
                use std::slice::from_raw_parts;

                let &mut GfxGraphics {
                    ref mut encoder,
                    g2d: &mut Gfx2d {
                        ref mut colored_offset,
                        ref mut buffer_pos,
                        ref mut buffer_color,
                        ..
                    },
                    ..
                } = self;

                unsafe {
                    encoder.update_buffer(
                        &buffer_pos,
                        from_raw_parts(
                            vertices.as_ptr() as *const PositionFormat,
                            n
                        ),
                        *colored_offset
                    ).unwrap();
                }

                for i in 0..n {
                    encoder.update_buffer(&buffer_color, &[ColorFormat {
                            color: color
                        }], *colored_offset + i).unwrap();
                }
                *colored_offset += n;
            }
        })
    }

    fn tri_list_c<F>(
        &mut self,
        draw_state: &DrawState,
        mut f: F
    )
        where F: FnMut(&mut dyn FnMut(&[[f32; 2]], &[[f32; 4]]))
    {
        // Flush when draw state changes.
        if &self.g2d.colored_draw_state != draw_state {
            self.flush_colored();
            self.g2d.colored_draw_state = *draw_state;
        }
        f(&mut |vertices: &[[f32; 2]], colors: &[[f32; 4]]| {
            let n = vertices.len();

            // Render if there is not enough room.
            if self.g2d.colored_offset + n > BUFFER_SIZE * CHUNKS {
                self.flush_colored();
            }

            {
                use std::slice::from_raw_parts;

                let &mut GfxGraphics {
                    ref mut encoder,
                    g2d: &mut Gfx2d {
                        ref mut colored_offset,
                        ref mut buffer_pos,
                        ref mut buffer_color,
                        ..
                    },
                    ..
                } = self;

                unsafe {
                    encoder.update_buffer(
                        &buffer_pos,
                        from_raw_parts(
                            vertices.as_ptr() as *const PositionFormat,
                            n
                        ),
                        *colored_offset
                    ).unwrap();
                }

                for (i, color) in colors.iter().enumerate() {
                    encoder.update_buffer(&buffer_color, &[ColorFormat {
                            color: gamma_srgb_to_linear(*color)
                        }], *colored_offset + i).unwrap();
                }
                *colored_offset += n;
            }
        })
    }

    fn tri_list_uv<F>(
        &mut self,
        draw_state: &DrawState,
        color: &[f32; 4],
        texture: &<Self as Graphics>::Texture,
        mut f: F
    )
        where F: FnMut(&mut dyn FnMut(&[[f32; 2]], &[[f32; 2]]))
    {
        use draw_state::target::Rect;
        use std::u16;

        let color = gamma_srgb_to_linear(*color);
        if self.g2d.colored_offset > 0 {
            self.flush_colored();
        }
        let &mut GfxGraphics {
            ref mut encoder,
            output_color,
            output_stencil,
            g2d: &mut Gfx2d {
                ref mut buffer_pos,
                ref mut buffer_uv,
                ref mut textured,
                ..
            },
            ..
        } = self;

        let (pso_textured, stencil_val) = textured.stencil_blend(
            draw_state.stencil,
            draw_state.blend
        );

        let scissor = match draw_state.scissor {
            None => Rect { x: 0, y: 0, w: u16::MAX, h: u16::MAX },
            Some(r) => Rect { x: r[0] as u16, y: r[1] as u16,
                w: r[2] as u16, h: r[3] as u16 }
        };

        let data = pipe_textured::Data {
            pos: buffer_pos.clone(),
            uv: buffer_uv.clone(),
            color: color,
            texture: (texture.view.clone(), texture.sampler.clone()),
            blend_target: output_color.clone(),
            stencil_target: (output_stencil.clone(),
                             (stencil_val, stencil_val)),
            blend_ref: [1.0; 4],
            scissor: scissor,
        };

        f(&mut |vertices: &[[f32; 2]], texture_coords: &[[f32; 2]]| {
            use std::slice::from_raw_parts;

            assert_eq!(
                vertices.len(),
                texture_coords.len()
            );
            let n = vertices.len();
            unsafe {
                encoder.update_buffer(
                    &buffer_pos,
                    from_raw_parts(
                        vertices.as_ptr() as *const PositionFormat,
                        n
                    ),
                    0
                ).unwrap();
                encoder.update_buffer(
                    &buffer_uv,
                    from_raw_parts(
                        texture_coords.as_ptr() as *const TexCoordsFormat,
                        n
                    ),
                    0
                ).unwrap();
            }

            let slice = gfx::Slice {
                instances: None,
                start: 0,
                end: n as u32,
                buffer: gfx::IndexBuffer::Auto,
                base_vertex: 0,
            };
            encoder.draw(&slice, pso_textured, &data);
        })
    }

    fn tri_list_uv_c<F>(
        &mut self,
        draw_state: &DrawState,
        texture: &<Self as Graphics>::Texture,
        mut f: F
    )
        where F: FnMut(&mut dyn FnMut(&[[f32; 2]], &[[f32; 2]], &[[f32; 4]]))
    {
        use draw_state::target::Rect;
        use std::u16;

        if self.g2d.colored_offset > 0 {
            self.flush_colored();
        }
        let &mut GfxGraphics {
            ref mut encoder,
            output_color,
            output_stencil,
            g2d: &mut Gfx2d {
                ref mut buffer_pos,
                ref mut buffer_uv,
                ref mut buffer_color,
                ref mut textured_color,
                ..
            },
            ..
        } = self;

        let (pso_textured_color, stencil_val) = textured_color.stencil_blend(
            draw_state.stencil,
            draw_state.blend
        );

        let scissor = match draw_state.scissor {
            None => Rect { x: 0, y: 0, w: u16::MAX, h: u16::MAX },
            Some(r) => Rect { x: r[0] as u16, y: r[1] as u16,
                w: r[2] as u16, h: r[3] as u16 }
        };

        let data = pipe_textured_color::Data {
            pos: buffer_pos.clone(),
            uv: buffer_uv.clone(),
            color: buffer_color.clone(),
            texture: (texture.view.clone(), texture.sampler.clone()),
            blend_target: output_color.clone(),
            stencil_target: (output_stencil.clone(),
                             (stencil_val, stencil_val)),
            blend_ref: [1.0; 4],
            scissor: scissor,
        };

        f(&mut |vertices: &[[f32; 2]], texture_coords: &[[f32; 2]], colors: &[[f32; 4]]| {
            use std::slice::from_raw_parts;

            assert_eq!(
                vertices.len(),
                texture_coords.len()
            );
            let n = vertices.len();
            unsafe {
                encoder.update_buffer(
                    &buffer_pos,
                    from_raw_parts(
                        vertices.as_ptr() as *const PositionFormat,
                        n
                    ),
                    0
                ).unwrap();
                encoder.update_buffer(
                    &buffer_uv,
                    from_raw_parts(
                        texture_coords.as_ptr() as *const TexCoordsFormat,
                        n
                    ),
                    0
                ).unwrap();
                encoder.update_buffer(
                    &buffer_color,
                    from_raw_parts(
                        colors.as_ptr() as *const ColorFormat,
                        n
                    ),
                    0
                ).unwrap();
            }

            let slice = gfx::Slice {
                instances: None,
                start: 0,
                end: n as u32,
                buffer: gfx::IndexBuffer::Auto,
                base_vertex: 0,
            };
            encoder.draw(&slice, pso_textured_color, &data);
        })
    }
}
