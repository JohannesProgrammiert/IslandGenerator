//! Render egui in allegro as overlay
//!
//! Example:
//!
//! ```rust
//! use std::sync::Rc;
//! use egui_allegro_backend::Backend;
//! const SCREEN_SIZE: euclid::Size2D<f32> = euclid::Size2D::new(1280.0, 720.0);
//! fn main() {
//!     let allegro_core = Rc::new(allegro::Core::init().unwrap());
//!     let allegro_display = Rc::new(allegro::Display::new(&core, SCREEN_SIZE.width() as i32, SCREEN_SIZE.height() as i32).unwrap());
//!     let allegro_primitves_addon = Rc::new(allegro_primitves::PrimitvesAddon::init(&core).unwrap());
//!     // ... initialize allegro engine ...
//!     let backend = Backend::new(
//!         egui::Rect {
//!             min: egui::Pos2{x: 0.0, y: 0.0},
//!             max: egui::Pos2{x: SCREEN_SIZE.width(), y: SCREEN_SIZE.height()},
//!         },
//!         allegro_core.clone(),
//!         allegro_display.clone(),
//!         allegro_primitives_addon.clone(),
//!     );
//! }
//! ```
use std::rc::Rc;

pub struct Backend {
    egui_ctx: egui::Context,
    egui_textures: Vec<allegro::Bitmap>,
    egui_texture_sizes: Vec<(i32, i32)>,
    egui_input: egui::RawInput,
    allegro_core: Rc<allegro::core::Core>,
    allegro_display: Rc<allegro::display::Display>,
    allegro_primitives_addon: Rc<allegro_primitives::PrimitivesAddon>,
}

impl Backend {
    pub fn new(
        egui_ctx: egui::Context,
        screen_size: egui::Rect,
        allegro_core: Rc<allegro::core::Core>,
        allegro_display: Rc<allegro::display::Display>,
        allegro_primitives_addon: Rc<allegro_primitives::PrimitivesAddon>,
    ) -> Self {
        let mut egui_input = egui::RawInput::default();
        egui_input.screen_rect = Some(screen_size);
        Backend {
            egui_ctx,
            egui_textures: Vec::new(),
            egui_texture_sizes: Vec::new(),
            egui_input,
            allegro_core,
            allegro_display,
            allegro_primitives_addon,
        }
    }

    pub fn handle_allegro_event(&mut self, event: &allegro::Event) {
        match event {
            allegro::TimerTick { .. } => {}
            allegro::MouseAxes {
                x, y, dz, dx, dy, ..
            } => {
                if *dz != 0 {
                    self.egui_input.events.push(egui::Event::Scroll(egui::Vec2 {
                        x: *dz as f32,
                        y: 0.0,
                    }))
                }
                if *dx != 0 || *dy != 0 {
                    self.egui_input
                        .events
                        .push(egui::Event::PointerMoved(egui::Pos2 {
                            x: *x as f32,
                            y: *y as f32,
                        }));
                }
            }
            allegro::MouseButtonDown { x, y, button, .. } => {
                let pos = egui::Pos2 {
                    x: *x as f32,
                    y: *y as f32,
                };
                let egui_button: egui::PointerButton;
                match button {
                    1 => egui_button = egui::PointerButton::Primary,
                    3 => egui_button = egui::PointerButton::Middle,
                    2 => egui_button = egui::PointerButton::Secondary,
                    _ => return,
                }
                self.egui_input.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui_button,
                    pressed: true,
                    modifiers: egui::Modifiers {
                        alt: false,
                        ctrl: false,
                        shift: false,
                        mac_cmd: false,
                        command: false,
                    },
                });
            }
            allegro::MouseButtonUp { x, y, button, .. } => {
                let pos = egui::Pos2 {
                    x: *x as f32,
                    y: *y as f32,
                };
                let egui_button: egui::PointerButton;
                match button {
                    1 => egui_button = egui::PointerButton::Primary,
                    3 => egui_button = egui::PointerButton::Middle,
                    2 => egui_button = egui::PointerButton::Secondary,
                    _ => return,
                }
                self.egui_input.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui_button,
                    pressed: false,
                    modifiers: egui::Modifiers {
                        alt: false,
                        ctrl: false,
                        shift: false,
                        mac_cmd: false,
                        command: false,
                    },
                });
            }
            allegro::DisplayResize { width, height, .. } => {
                self.egui_input.screen_rect = Some(egui::Rect {
                    min: egui::Pos2 { x: 0.0, y: 0.0 },
                    max: egui::Pos2 {
                        x: *width as f32,
                        y: *height as f32,
                    },
                });
            }
            _ => {}
        }
    }

    pub fn draw<T>(&mut self, gui: fn(ctx: &egui::Context, args: &mut T), gui_args: &mut T) {
        // Gather input (mouse, touches, keyboard, screen size, etc):
        let output = self.egui_ctx.run(self.egui_input.clone(), |ctx| {
            gui(ctx, gui_args);
        });
        self.egui_input.events.clear();
        self.upload_egui_textures(output.textures_delta);

        // create triangles to paint
        let clipped_primitives = self.egui_ctx.tessellate(output.shapes);

        // backup Allegro state
        // let last_transform = self.allegro_core.get_current_transform();
        let last_clip = self.allegro_core.get_clipping_rectangle();
        // draw mesh
        for clipped_primitive in clipped_primitives {
            let mesh: epaint::Mesh;
            if let epaint::Primitive::Mesh(m) = clipped_primitive.primitive {
                mesh = m;
            }
            else {
                continue;
            }
            debug_assert!(mesh.is_valid());
            // a mesh is
            // + a set of Vertices
            // + a set of associated indices
            // + a texture id

            let texture_id: usize;
            match mesh.texture_id {
                egui::TextureId::Managed(id) => {
                    if (id as usize) >= self.egui_textures.len()
                        || (id as usize >= self.egui_texture_sizes.len()) {
                            log::error!("Requested TextureId out of range");
                            continue;
                        }
                    else {
                        texture_id = id as usize;
                    }
                },
                egui::TextureId::User(_) => {
                    log::warn!("Unsupported: TextureId of type 'User'");
                    continue;
                },
            }
            // convert egui-vertices to allegro-vertices
            let vertex_buffer = VertexBuffer {
                data: mesh
                    .vertices
                    .iter()
                    .map(|vert| {
                        let col = vert.color.to_array();
                        let mut u = vert.uv.x * self.egui_texture_sizes[texture_id].0 as f32;
                        let mut v = vert.uv.y * self.egui_texture_sizes[texture_id].1 as f32;
                        let color = allegro::Color::from_rgba(
                                 col[0], col[1], col[2], col[3]);
                        // for some reason the point (0.0, 0.0) needs offset
                        // allegro docs say all points need this offset, it does work like this however
                        if u == 0.0 && v == 0.0 {
                            u += 0.5;
                            v += 0.5;
                        }
                        return allegro_primitives::Vertex {
                            color,
                            u,
                            v,
                            x: vert.pos.x,
                            y: vert.pos.y,
                            z: 0.0,
                        };
                    })
                    .collect(),
            };
            // convert indices
            let indices: Vec<i32> = mesh.indices.iter().map(|index| *index as i32).collect();

            // set clipping recangle before drawing
            self.allegro_core.set_clipping_rectangle(
                clipped_primitive.clip_rect.min.x as i32,
                clipped_primitive.clip_rect.min.y as i32,
                clipped_primitive.clip_rect.max.x as i32,
                clipped_primitive.clip_rect.max.y as i32,
            );

            // draw resulting allegro texture
            self.allegro_primitives_addon
                .draw_indexed_prim::<VertexBuffer, allegro::Bitmap>(
                    &vertex_buffer,
                    Some(&self.egui_textures[texture_id]),
                    &indices[..],
                    0,
                    indices.len() as u32,
                    allegro_primitives::PrimType::TriangleList,
                );
        }

        // restore Allegro state
        self.allegro_core.set_clipping_rectangle(
            last_clip.0,
            last_clip.1,
            last_clip.2,
            last_clip.3,
        );
        self.allegro_core.set_blender(
            allegro::core::BlendOperation::Add,
            allegro::core::BlendMode::One,
            allegro::core::BlendMode::InverseAlpha,
        );
    }

    const GAMMA: f32 = 1.0;

    /// convert egui texture to `allegro::Bitmap`
    fn upload_egui_textures(&mut self, textures_delta: egui::TexturesDelta) {
        if textures_delta.is_empty() {
            return;
        }
        // iterate obsolete textures and remove them
        for _ in textures_delta.free {
            log::debug!("Unsupported: Free'ing textures");
        }
        // iterate new textures and apply updates
        for (texture_id, image_delta) in textures_delta.set {
            if image_delta.filter != epaint::textures::TextureFilter::Linear {
                log::warn!("Unsupported: Filter {:?}", image_delta.filter);
                continue;
            }
            self.allegro_core.set_new_bitmap_flags(allegro::MEMORY_BITMAP | allegro::MAG_LINEAR);
            let font_image: epaint::image::FontImage;
            if let epaint::image::ImageData::Font(image) = image_delta.image {
                font_image = image;
            }
            else {
                log::warn!("Unsupported: ImageData of type 'Color'");
                continue;
            }
            if let Some(_) = image_delta.pos {
                log::warn!("Unsupported: partial texture update");
                continue;
            }
            let tex: allegro::Bitmap;
            match allegro::Bitmap::new(
                &self.allegro_core,
                font_image.width() as i32,
                font_image.height() as i32,
            ) {
                Err(_) => {
                    log::error!("Cannot create allegro bitmap: allegro::Bitmap::new() failed");
                    continue;
                }
                Ok(t) => tex = t,
            }
            let iter = font_image.srgba_pixels(Backend::GAMMA);

            let pixels: Vec<allegro::Color> = iter
                .map(|x| {
                    let color = x.to_array();
                    allegro::Color::from_rgba(color[0], color[1], color[2], color[3])
                })
                .collect();
            let mut iter = pixels.iter();
            self.allegro_core.set_target_bitmap(Some(&tex));
            for y in 0..font_image.height() as i32 {
                for x in 0..font_image.width() as i32 {
                    let color = *iter.next().unwrap();
                    self.allegro_core.put_pixel(x, y, color);
                }
            }
            self.allegro_core.set_target_bitmap(Some(self.allegro_display.get_backbuffer()));
            self.allegro_core.set_new_bitmap_flags(
                allegro::VIDEO_BITMAP | allegro::MAG_LINEAR
            );
            self.allegro_core.set_new_bitmap_format(allegro::color::PixelFormat::PixelFormatAbgr8888Le);

            // resolve TextureID and insert bitmap to struct
            if let egui::TextureId::Managed(id) = texture_id {
                if (id as usize) < self.egui_textures.len() {
                    log::warn!("Texture ID already exists. Skipping update");
                    continue;
                }
                else if (id as usize) != self.egui_textures.len() {
                    // for this we need to store textures inside a HashMap instead of vector
                    panic!("Texture IDs have gaps, egui backend cannot handle this");
                }
                else {
                    match self.allegro_display.convert_bitmap(&tex) {
                        Ok(allegro_bitmap) => self.egui_textures.push(allegro_bitmap),
                        Err(_) => {
                            log::error!("Failed to convert egui bitmap to allegro bitmap");
                            continue;
                        },
                    }
                    self.egui_texture_sizes.push((
                        font_image.width() as i32,
                        font_image.height() as i32,
                    ));
                }
            }
            else {
                log::warn!("Unsupported: TextureId of type 'user");
            }
        }
    }
}

struct VertexBuffer {
    data: Vec<allegro_primitives::Vertex>,
}
impl allegro_primitives::VertexSource for VertexBuffer {
    type VertexType = allegro_primitives::Vertex;
    fn get_ptr(&self) -> *const u8 {
        self.data.as_ptr() as *const u8
    }
}
