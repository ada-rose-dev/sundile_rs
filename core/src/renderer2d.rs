///
/// 2d.rs 
/// 
/// Contains API for in-game GUI / 2d overlay drawing.
/// 

// Needed functionality:
// Draw quad w/ color & alpha
// Draw textured quad
// Draw text
use sundile_graphics::prelude::*;
use sundile_assets::prelude::*;
use wgpu::*;
use wgpu::util::*;
use std::collections::HashMap;
use wgpu_glyph::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vert2d {
    position: [f32;3],
    color: [f32;4],
    texcoords: [f32;2],
}

impl Vertex for Vert2d {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        use std::mem;
        VertexBufferLayout {
            array_stride: mem::size_of::<Vert2d>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x2,
                }
            ],
        }
    }
}

/// Sprite struct. Always contained within a TextureAtlas.
#[allow(dead_code)]
pub struct Sprite {
    frames: Vec<[u32;2]>, //xy in pixel coordinates on the texture
    width: u32,
    height: u32,
    num_frames: f32,
    pub frame: f32,
    pub fps: f32,
}
#[allow(dead_code)]
impl Sprite {
    pub fn new(frames: Vec<[u32;2]>, width: u32, height: u32, num_frames: f32, fps: f32) -> Self {
        Self {
            frames,
            num_frames,
            width,
            height,
            frame: 0.0,
            fps,
        }
    }
    pub fn update(&mut self, dt: f32) {
        if self.num_frames > 0.0 {
            self.frame += self.fps * dt;
            if self.frame > self.num_frames {
                self.frame -= self.num_frames;
            }
        }
    }
    pub fn current_frame(&self) -> [u32;2] {
        self.frames[self.frame.floor() as usize]
    }
}

struct TextureAtlas { //Move this into assets library.
    texture: texture::Texture,
    bind_group: BindGroup,
    sprites: HashMap<&'static str, Sprite>,
}
impl TextureAtlas {
    pub fn new(render_target: &RenderTarget, texture_bind_group_layout: &BindGroupLayout, bytes: &[u8]) -> Self {
        //let texture = texture::Texture::load(&render_target.device, &render_target.queue, "assets/textures/atlas_0.png", false).expect("Unable to create texture atlas!");
        let texture = texture::Texture::from_bytes(&render_target.device, &render_target.queue, bytes, "2D Texture Atlas", false).expect("Unable to create texture atlas!");

        let bind_group = render_target.device.create_bind_group(&BindGroupDescriptor {
            label: Some("2D Texture Atlas Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture.sampler),
                }
            ],
        });

        Self {
            texture,
            bind_group,
            sprites: HashMap::from_iter(vec![
                ("default", Sprite::new(vec![[0,0]], 16, 16, 1.0, 0.0) ), 
                ("circle", Sprite::new(vec![[16,0]], 16, 16, 1.0, 0.0) ),
            ]),
        }
    }
}

struct Quad<'a> {
    sprite: Option<&'a str>,
    vertices: [f32;4],
    color: sundile_graphics::Color,
}

pub struct Renderer2d<'a> {
    texture_atlas: TextureAtlas,
    queue: Vec<Quad<'a>>,
    pipeline: RenderPipeline,
    color: sundile_graphics::Color,
    screen_size: [u32;2],
    
    text_wrapper: TextWrapper,
    text_queue: Vec<Section<'a>>,
    text_bounds: (f32, f32),
    font_size: f32,
    current_font: Option<FontId>,
}

#[allow(dead_code)]
impl<'a> Renderer2d<'a> {
    pub fn new(render_target: &RenderTarget, assets: &AssetTypeMap,) -> Self {
        let (device, config, ) = (
            &render_target.device,
            &render_target.config,
        );

        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Renderer2D Texture Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: false},
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None
                },
            ],
        });
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Renderer2D Pipeline Layout Descriptor"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        // let shader = device.create_shader_module(&ShaderModuleDescriptor{
        //     label: Some("Renderer2D Default Shader"),
        //     source: ShaderSource::Wgsl(assets.shaders["2d"].clone().into()),
        // });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Renderer2D Pipeline"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &assets.shaders["2d"],
                entry_point: "vs_main",
                buffers: &[Vert2d::desc()],
            },
            fragment: Some(FragmentState {
                module: &assets.shaders["2d"],
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let text_wrapper = TextWrapper::new(&render_target, assets.fonts.clone());

        Self {
            texture_atlas: TextureAtlas::new(&render_target, &texture_bind_group_layout, &assets.textures["atlas_0"][..]),
            queue: vec![],
            pipeline,
            color: sundile_graphics::Color::from_rgb(1.0, 1.0, 1.0),
            screen_size: [render_target.config.width, render_target.config.height],

            text_wrapper,
            text_queue: Vec::<Section<'a>>::new(),
            text_bounds: (render_target.config.width as f32, render_target.config.height as f32),
            font_size: 16.0,
            current_font: None,
        }
    }

    pub fn render(&mut self, render_target: &mut RenderTarget) {
        // smoosh quads into batch
        let mut vertices: Vec<Vert2d> = vec![];
        let mut indices: Vec<u32> = vec![];

        while let Some(quad) = self.queue.pop() {
            let sprite = &self.texture_atlas.sprites[quad.sprite.unwrap_or("default")];

            let tw = self.texture_atlas.texture.size.width as f32;
            let th = self.texture_atlas.texture.size.height as f32;
            let (x1, y1, x2, y2) = (
                quad.vertices[0],
                quad.vertices[1],
                quad.vertices[0] + quad.vertices[2],
                quad.vertices[1] + quad.vertices[3],
            );
            let sxy = sprite.current_frame();
            let (tx1, ty1, tx2, ty2) = (
                sxy[0] as f32 / tw,
                sxy[1] as f32 / th,
                (sxy[0] + sprite.width) as f32 / tw,
                (sxy[1] + sprite.height) as f32 / th,
            );

            vertices.push(Vert2d {
                position: [x1, y1, 0.0],
                texcoords: [tx1, ty1],
                color: quad.color.as_array(),
            });
            vertices.push(Vert2d {
                position: [x1, y2, 0.0],
                texcoords: [tx1, ty2],
                color: quad.color.as_array(),
            });
            vertices.push(Vert2d {
                position: [x2, y1, 0.0],
                texcoords: [tx2, ty1],
                color: quad.color.as_array(),
            });
            vertices.push(Vert2d {
                position: [x2, y2, 0.0],
                texcoords: [tx2, ty2],
                color: quad.color.as_array(),
            });

            let i0 = (vertices.len()-4) as u32;
            indices.push(i0);
            indices.push(i0+1);
            indices.push(i0+2);
            indices.push(i0+1);
            indices.push(i0+3);
            indices.push(i0+2);
        }

        let device = &render_target.device;
        let vertex_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("2D Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("2D Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        // render batch
        let mut rp = render_target.get_render_pass(false, false);
        rp.set_pipeline(&self.pipeline);
        rp.set_vertex_buffer(0, vertex_buffer.slice(..));
        rp.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
        rp.set_bind_group(0, &self.texture_atlas.bind_group, &[]);
        rp.draw_indexed(0..indices.len() as u32, 0, 0..1);        
        drop(rp);

        // render text
        self.text_wrapper.start_pass();
        while let Some(section) = self.text_queue.pop() {
            self.text_wrapper.queue_section(section);
        }
        self.text_wrapper.end_pass(render_target);
    }

    pub fn set_color(&mut self, color: sundile_graphics::Color) {
        self.color = color;
    }

    /// Draws at the given screen coordinates in the range (-1.0, 1.0).
    fn push_quad(&mut self, x: f32, y:f32, width: f32, height: f32, sprite: Option<&'static str>) {
        self.queue.push(Quad{vertices:[x, y, width, height], sprite, color: self.color});
    }

    /// Draw quad at relative coordinates, in range (0.0, 1.0)
    pub fn draw_quad_rel(&mut self, x:f32, y:f32, width:f32, height:f32) {
        self.push_quad(x * 2.0 - 1.0, y * 2.0 - 1.0, width * 2.0, height * 2.0, None);
    }

    /// Draw quad at pixel coordinates, with top-left at (0,0)
    pub fn draw_quad(&mut self, x:f32, y:f32, width:f32, height:f32) {
        let sw = self.screen_size[0] as f32;
        let sh = self.screen_size[1] as f32;
        self.push_quad(
            (x / sw) * 2.0 - 1.0,
            (y / sh) * 2.0 - 1.0,
            (width / sw) * 2.0,
            (height / sh) * 2.0,
            None,
        )
    }

    /// Draw a sprite at the given pixel coordinates, with top-left at (0,0).
    pub fn draw_sprite(&mut self, x:f32, y:f32, width_multiplier:f32, height_multiplier:f32, sprite: &'static str) {
        let spr = &self.texture_atlas.sprites[sprite]; //TODO: Don't borrow the sprite here for performance reasons?
        let spr_width = spr.width as f32;
        let spr_height = spr.height as f32;
        let sw = self.screen_size[0] as f32;
        let sh = self.screen_size[1] as f32;
        self.push_quad(
            (x / sw) * 2.0 - 1.0,
            (y / sh) * 2.0 - 1.0,
            (spr_width * width_multiplier / sw) * 2.0,
            (spr_height * height_multiplier / sh) * 2.0,
            Some(sprite),
        );
    }

    /// Sets bounding box for text.
    pub fn set_text_bounds(&mut self, width: f32, height: f32) {
        self.text_bounds = (width, height);
    }

    /// Sets current font.
    pub fn set_font(&mut self, font: &'static str, font_size: f32) {
        self.current_font = Some(self.text_wrapper.font(font));
        self.font_size = font_size;
    }

    /// Draws text at the given pixel coordinates.
    pub fn draw_text(&mut self, text: &'a str, x: f32, y: f32) {
        self.text_queue.push(Section {
            screen_position: (x,y),
            bounds: self.text_bounds,
            text: vec![
                Text::new(text)
                .with_color(self.color.as_array())
                .with_scale(self.font_size)
                .with_font_id(self.current_font.unwrap_or(FontId(0)))
            ],
            ..Section::default()
        });
    }
}