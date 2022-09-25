use crate::glob::types::*;
#[derive(PartialEq)]
pub enum SidePanelTab {
    Main,
    Settings,
    Debug,
}
pub struct GuiInfo {
    pub drawn_tiles: usize,
    pub active_side_panel_tab: SidePanelTab,
    pub min_side_panel_width: f32,
    pub fps: f32,
    pub rendered_rect: ScreenRect,
    pub mouse_pos: WorldCoordinate,
    pub show_map: bool,
}

pub fn draw_gui(ctx: &egui::Context, args: &mut GuiInfo) {
    egui::SidePanel::right("Game")
        .min_width(args.min_side_panel_width)
        .show(&ctx, |ui| {
            ui.heading("Side Panel");
            ui.separator();
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut args.active_side_panel_tab,
                    SidePanelTab::Debug,
                    "Debug",
                );
                ui.selectable_value(&mut args.active_side_panel_tab, SidePanelTab::Main, "Main");
                ui.selectable_value(
                    &mut args.active_side_panel_tab,
                    SidePanelTab::Settings,
                    "Settings",
                );
            });
            ui.separator();
            match args.active_side_panel_tab {
                SidePanelTab::Main => {
                    ui.label("New Game");
                }
                SidePanelTab::Settings => {
                    ui.label("Settings");
                }
                SidePanelTab::Debug => {
                    ui.label(format!("Drawn Tiles: {}", args.drawn_tiles));
                    ui.label(format!("FPS: {}", args.fps));
                    let mut render_width = args.rendered_rect.width();
                    let mut render_height = args.rendered_rect.height();
                    ui.add(egui::Slider::new(&mut render_width, 1.0..=4000.0).text("Rendered width"));
                    ui.add(egui::Slider::new(&mut render_height, 1.0..=4000.0).text("Rendered height"));
                    args.rendered_rect = ScreenRect::new(ScreenCoordinate::new(0.0, 0.0), ScreenVector::new(render_width, render_height).to_size());
                    ui.label(format!("Mouse X: {}", args.mouse_pos.x));
                    ui.label(format!("Mouse Y: {}", args.mouse_pos.y));
                }
            }
        });
    egui::TopBottomPanel::bottom("Toolbar").show(&ctx, |ui| {
        if ui.button("Map").clicked() {
            args.show_map = !args.show_map;
        }
    });
}

impl Default for GuiInfo {
    fn default() -> Self {
        GuiInfo {
            drawn_tiles: 0,
            active_side_panel_tab: SidePanelTab::Main,
            min_side_panel_width: 200.0,
            fps: 0.0,
            rendered_rect: ScreenRect::from_size(ScreenVector::new(1.0, 1.0).to_size()),
            mouse_pos: WorldCoordinate::new(0.0, 0.0),
            show_map: false,
        }
    }
}
