use std::rc::Rc;

use allegro::BitmapLike;
pub struct Backend {
    egui_ctx: egui::CtxRef,
    egui_texture: Option<allegro::Bitmap>,
    egui_texture_size: Option<egui::Pos2>,
    egui_texture_version: Option<u64>,
    egui_input: egui::RawInput,
    allegro_core: Rc<allegro::core::Core>,
    allegro_display: Rc<allegro::display::Display>,
    allegro_primitives_addon: Rc<allegro_primitives::PrimitivesAddon>,
}

impl Backend {
    pub fn new(
        screen_size: egui::Rect,
        allegro_core: Rc<allegro::core::Core>,
        allegro_display: Rc<allegro::display::Display>,
        allegro_primitives_addon: Rc<allegro_primitives::PrimitivesAddon>,
    ) -> Self {
        let mut egui_input = egui::RawInput::default();
        egui_input.screen_rect = Some(screen_size);
        let egui_ctx = egui::CtxRef::default();
        let mut style = (*egui_ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.window_shadow.extrusion = 0.0;
        egui_ctx.set_style(style);
        Backend {
            egui_ctx,
            egui_texture: None,
            egui_texture_size: None,
            egui_texture_version: None,
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

    pub fn draw<T>(&mut self, gui: fn(ctx: &egui::CtxRef, args: &mut T), gui_args: &mut T) {
        // TODO egui output, is for clipboard, link opening, cursor position...
        // Gather input (mouse, touches, keyboard, screen size, etc):
        let (_output, shapes) = self.egui_ctx.run(self.egui_input.clone(), |ctx| {
            gui(ctx, gui_args);
        });
        self.egui_input.events.clear();
        self.upload_egui_texture(&self.egui_ctx.font_image());

        let clipped_meshes = self.egui_ctx.tessellate(shapes); // creates triangles to paint
                                                               // TODO avoid minimized rendering

        // backup Allegro state
        // let last_transform = self.allegro_core.get_current_transform();
        let last_clip = self.allegro_core.get_clipping_rectangle();
        // draw mesh
        for egui::ClippedMesh(clip_rect, mesh) in clipped_meshes {
            debug_assert!(mesh.is_valid());

            let vertex_buffer = VertexBuffer {
                data: mesh
                    .vertices
                    .iter()
                    .map(|vert| {
                        let col = vert.color.to_array();
                        let mut u = vert.uv.x * self.egui_texture_size.unwrap().x;
                        let mut v = vert.uv.y * self.egui_texture_size.unwrap().y;
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
            let indices: Vec<i32> = mesh.indices.iter().map(|index| *index as i32).collect();
            self.allegro_core.set_clipping_rectangle(
                clip_rect.min.x as i32,
                clip_rect.min.y as i32,
                clip_rect.max.x as i32,
                clip_rect.max.y as i32,
            );
            if let Some(texture) = self.get_texture(mesh.texture_id) {
                self.allegro_primitives_addon
                    .draw_indexed_prim::<VertexBuffer, allegro::Bitmap>(
                        &vertex_buffer,
                        Some(texture),
                        &indices[..],
                        0,
                        indices.len() as u32,
                        allegro_primitives::PrimType::TriangleList,
                    );
            }
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
        /*my_integration.set_cursor_icon(output.cursor_icon);
        if !output.copied_text.is_empty() {
            my_integration.set_clipboard_text(output.copied_text);
        }*/
        // See `egui::Output` for more
    }
    const GAMMA: f32 = 1.0;
    fn upload_egui_texture(&mut self, font_image: &egui::FontImage) {
        if self.egui_texture_version == Some(font_image.version) {
            return; // no change
        }

        self.allegro_core
            .set_new_bitmap_flags(allegro::MEMORY_BITMAP | allegro::MAG_LINEAR);
        // self.allegro_core
        //     .set_new_bitmap_format(allegro::color::PixelFormat::PixelFormatRgba8888);
        let tex = allegro::Bitmap::new(
            &self.allegro_core,
            font_image.width as i32,
            font_image.height as i32,
        )
        .expect("Unable to create bitmap");
        let iter = font_image.srgba_pixels(Backend::GAMMA);
        let pixels: Vec<allegro::Color> = iter
            .map(|x| {
                let color = x.to_array();
                allegro::Color::from_rgba(color[0], color[1], color[2], color[3])
            })
            .collect();
        let mut iter = pixels.iter();
        self.allegro_core.set_target_bitmap(Some(&tex));
        for y in 0..font_image.height as i32 {
            for x in 0..font_image.width as i32 {
                let color = *iter.next().unwrap();
                // self.allegro_core.draw_pixel(x as f32, y as f32, color);
                self.allegro_core.put_pixel(x, y, color);
            }
        }
        // unsafe {
        //     let c_str = std::ffi::CString::new("egui_tex.png").unwrap();
        //     allegro_sys::al_save_bitmap(c_str.as_ptr(), tex.get_allegro_bitmap());
        // }
        self.allegro_core
            .set_target_bitmap(Some(self.allegro_display.get_backbuffer()));
        self.allegro_core.set_new_bitmap_flags(
            allegro::VIDEO_BITMAP | allegro::MAG_LINEAR
        );
        self.allegro_core
            .set_new_bitmap_format(allegro::color::PixelFormat::PixelFormatAbgr8888Le);
        self.egui_texture = Some(self.allegro_display.convert_bitmap(&tex).unwrap());
        // self.egui_texture = Some(tex);
        self.egui_texture_size = Some(egui::Pos2 {
            x: font_image.width as f32,
            y: font_image.height as f32,
        });
        self.egui_texture_version = Some(font_image.version);
    }

    fn get_texture(&self, texture_id: egui::TextureId) -> Option<&allegro::Bitmap> {
        match texture_id {
            egui::TextureId::Egui => self.egui_texture.as_ref(),
            egui::TextureId::User(_) => None,
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
