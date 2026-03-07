// Copyright (C) 2026  Antonio-Miguel Corbi Bellot
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

// -- Consts: -------------------------------------------------------------
const NELEMENTS: usize = 50;
const STROKE_W: f32 = 0.3;

// -- Types: --------------------------------------------------------------
type Point2D = Pos2;
type Canvas = Vec<Vec<u8>>;

// -- Uses: ---------------------------------------------------------------
use delegate::delegate;
use egui::{
    emath::{self, RectTransform},
    pos2, Color32, Frame, PointerButton, Pos2, Rect, Sense, Stroke, Ui, Vec2, Window,
};
use std::ops::{Index, IndexMut};

// -- Traits: -------------------------------------------------------------

trait AppUi {
    fn create_drawing_widget(&mut self, ui: &mut Ui) -> egui::Painter;
    fn create_stroke_widget(&mut self, ui: &mut Ui) -> egui::Response;
    fn draw_point(&self, p: Point2D, color: Color32, zoom: f32, painter: &egui::Painter);
    fn draw_lines(&self, lines: &Vec<Pos2>, color: Color32, painter: &egui::Painter);
}

// -- Types: --------------------------------------------------------------
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FallingSandApp {
    // Data
    data: Canvas,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FallingSandAppUi {
    pub fsapp: FallingSandApp,
    pub stroke: Stroke,
    pub world_rect: Rect,
    pub screen_rect: Rect,
    pub w2s: RectTransform,
    pub s2w: RectTransform,
}

// -- Impl: ---------------------------------------------------------------
impl Default for FallingSandApp {
    fn default() -> Self {
        Self {
            data: FallingSandApp::create_data(),
        }
    }
}

impl Default for FallingSandAppUi {
    fn default() -> Self {
        let fsapp = FallingSandApp::default();
        let world_rect = fsapp.world_rect();
        let screen_rect = Rect::ZERO;
        let w2s = emath::RectTransform::from_to(world_rect, screen_rect);
        let s2w = w2s.inverse();

        // println!("Just created app has:");
        // fsapp.show_data();

        Self {
            // Example stuff:
            fsapp,
            stroke: Stroke::new(STROKE_W, Color32::CYAN.linear_multiply(1.25)),
            world_rect,
            screen_rect,
            w2s,
            s2w,
        }
    }
}

impl FallingSandApp {
    pub fn nrows(&self) -> usize {
        self.data.len()
    }

    pub fn ncols(&self) -> usize {
        self.data[0].len()
    }

    fn world_rect(&self) -> Rect {
        let min = pos2(0.0, 0.0);
        let w = self.ncols() as f32 - 1.0;
        let h = self.nrows() as f32 - 1.0;
        //let size: Vec2 = Vec2::new(w, h);
        let max = pos2(w, h);
        //Rect::from_min_size(min, size)
        Rect::from_min_max(min, max)
    }

    pub fn create_data() -> Canvas {
        vec![vec![0; NELEMENTS]; NELEMENTS]
    }

    pub fn clear_data(&mut self) {
        println!("----------------------------------");
        for r in 0..self.nrows() {
            for c in 0..self.ncols() {
                self.data[r][c] = 0;
            }
        }
    }

    pub fn show_data(&self) {
        // println!("r:{} / c: {}", self.nrows(), self.ncols());
        println!("----------------------------------");
        for r in 0..self.nrows() {
            for c in 0..self.ncols() {
                let e = if self.data[r][c] == 0 { '.' } else { '+' };
                print!("{:1}", e);
            }
            println!();
        }
        // println!("----------------------------------");
    }

    pub fn next_step(&mut self) {
        let mut next_data = FallingSandApp::create_data();
        for r in 0..self.nrows() {
            for c in 0..self.ncols() {
                let state = self.data[r][c];
                let nextr = if r < self.nrows() - 1 { r + 1 } else { r };
                let nextc = if c < self.ncols() - 1 { c + 1 } else { c };
                if state == 1 {
                    let below = self.data[nextr][c];
                    if below == 0 {
                        next_data[r][c] = 0;
                        next_data[nextr][c] = 1;
                    }
                }
            }
        }
        self.data = next_data;
    }
}

impl FallingSandAppUi {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        // } else {
        //     Default::default()
        // }
        Default::default()
    }

    pub fn update_transforms(&mut self, screen_rect: Rect) {
        // Store the canvas rect
        self.screen_rect = screen_rect;

        // Compute world2screen and screen2world transforms
        self.w2s = emath::RectTransform::from_to(self.world_rect, self.screen_rect);
        self.s2w = self.w2s.inverse();
    }

    pub fn pos2_to_screen(&self, pos: Pos2) -> Pos2 {
        self.w2s.transform_pos_clamped(pos)
    }

    pub fn pos2_to_world(&self, pos: Pos2) -> Pos2 {
        self.s2w.transform_pos_clamped(pos)
    }

    pub fn rect_to_screen(&self, rect: Rect) -> Rect {
        self.w2s.transform_rect(rect)
    }

    pub fn rect_to_world(&self, rect: Rect) -> Rect {
        self.s2w.transform_rect(rect)
    }

    fn draw_contents_ascii(&self) {
        println!("----------------------------------");
        self.fsapp.data.iter().for_each(|col| {
            col.iter().for_each(|item| {
                print!("{}", if *item == 0 { '.' } else { '*' });
                //println!("Immutable reference (for_each): {}", item);
            });
            println!();
        });
        println!("----------------------------------\n");
    }

    pub fn draw_contents(&self, painter: egui::Painter) {
        // self.draw_contents_ascii();
        self.fsapp.data.iter().enumerate().for_each(|(ridx, col)| {
            col.iter().enumerate().for_each(|(cidx, item)| {
                let wpos = pos2(cidx as f32, ridx as f32);
                let pos = self.pos2_to_screen(wpos);
                if *item == 1 {
                    let mut zoom = self.screen_rect.width() / NELEMENTS as f32;
                    zoom *= self.stroke.width;
                    //painter.circle_filled(pos, 2.0, self.stroke.color);
                    self.draw_point(pos, self.stroke.color, zoom, &painter);
                }
            });
        });
    }
    // -- Delegates: ----------------------------------------------------------
    delegate! {
          to self.fsapp {
            pub fn show_data(&self);
            pub fn next_step(&mut self);
          }
    }
}

// -- Impl For: -----------------------------------------------------------

impl Index<usize> for FallingSandAppUi {
    type Output = Vec<u8>;

    fn index(&self, index: usize) -> &Self::Output {
        // println!("Accessing {index:?}-side of balance immutably");
        &self.fsapp.data[index]
    }
}

impl IndexMut<usize> for FallingSandAppUi {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        // println!("Accessing {index:?}-side of balance immutably");
        &mut self.fsapp.data[index]
    }
}

impl AppUi for FallingSandAppUi {
    fn create_drawing_widget(&mut self, ui: &mut Ui) -> egui::Painter {
        let (response, painter) = ui.allocate_painter(
            Vec2::new(ui.available_width(), ui.available_height() - 50.0),
            Sense::DRAG | Sense::CLICK,
        );

        // Compute transforms: w2s + s2w
        self.update_transforms(response.rect);

        // 2. Comprobamos el click izquierdo
        if response.secondary_clicked() {
            //println!("¡Click derecho detectado en el Painter!");
            if let Some(pos) = response.interact_pointer_pos() {
                println!("Click en la posición: {:?}", pos);
            }
        }

        if response.middle_clicked() {
            //println!("¡Click central detectado en el Painter!");
            if let Some(pos) = response.interact_pointer_pos() {
                println!("Click en la posición: {:?}", pos);
            }
            self.show_data();
        }

        if response.clicked() {
            //println!("¡Click izquierdo detectado en el Painter!");
            if let Some(pos) = response.interact_pointer_pos() {
                let wpos = self.pos2_to_world(pos);
                let wx = wpos.x.round() as usize;
                let wy = wpos.y.round() as usize;
                // println!(
                //     "Click en la posición screen:{:?} / world: {:?} / w.x: {} · w.y: {} ",
                //     pos, wpos, wx, wy,
                // );
                self[wy][wx] = 1;
            }
        }

        if response.dragged_by(PointerButton::Primary) {
            // Obtenemos la posición actual del puntero
            if let Some(pos) = response.interact_pointer_pos() {
                let ctx = ui.ctx();
                ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(false));
                // Dibujamos un círculo donde esté el ratón mientras arrastramos
                // painter.circle_filled(pos, 2.0, Color32::BLUE);

                // También puedes obtener cuánto se ha movido desde el frame anterior
                // let delta = response.drag_delta();
                // println!("Moviendo: {:?}", delta);

                let wpos = self.pos2_to_world(pos);
                let wx = wpos.x.round() as usize;
                let wy = wpos.y.round() as usize;
                // println!(
                //     "Click en la posición screen:{:?} / world: {:?} / w.x: {} · w.y: {} ",
                //     pos, wpos, wx, wy,
                // );
                self[wy][wx] = 1;
            }
        }

        if response.drag_stopped_by(PointerButton::Primary) {
            let ctx = ui.ctx();
            // Útil si el usuario estaba arrastrando y soltó el botón
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(true));
        }

        // 3. Dibujamos algo basado en el estado
        // let color = if response.hovered() {
        //     // egui::Color32::RED
        //     self.stroke.color
        // } else {
        //     egui::Color32::LIGHT_GRAY
        // };

        // Feedback
        // self.draw_point(response.rect.center(), color, 40.0, &painter);
        //painter.circle_filled(response.rect.center(), 40.0, color);

        //response
        painter
    }

    fn create_stroke_widget(&mut self, ui: &mut egui::Ui) -> egui::Response {
        //let mut stroke: Stroke = self.stroke;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Grains props.").color(egui::Color32::GREEN));
            ui.add(&mut self.stroke);
            ui.separator();
            if ui.button("Clear Canvas").clicked() {
                self.fsapp.clear_data();
            }
        })
        .response
    }

    fn draw_point(&self, p: Point2D, color: Color32, zoom: f32, painter: &egui::Painter) {
        // También puedes obtener los límites
        // let min = painter.clip_rect().min; // Esquina superior izquierda (Pos2)
        // let max = painter.clip_rect().max; // Esquina inferior derecha (Pos2)

        let centro = pos2(p.x, p.y);
        let mut radio = zoom;
        // let radio = zoom.min(3.5);
        // let radio = ((zoom + 0.125) / 2.5).max(3.5);
        // let color = Color32::from_rgb(255, 255, 255);

        if zoom < 0.5 {
            radio = 0.5;
        }
        // if zoom > 4.0 {
        //     radio = 4.0;
        // }

        painter.circle_filled(centro, radio, color);
    }

    fn draw_lines(&self, lines: &Vec<Pos2>, color: Color32, painter: &egui::Painter) {
        let stroke = Stroke::new(0.5, color);
        painter.line(lines.to_vec(), stroke);
    }
}

impl eframe::App for FallingSandAppUi {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_zoom_factor(2.0);

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading(egui::RichText::new("·:Falling Sand App:·").color(egui::Color32::RED));

            self.create_stroke_widget(ui);

            // ui.horizontal(|ui| {
            //     ui.label("Write something: ");
            //     ui.text_edit_singleline(&mut self.label);
            // });

            // ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            // if ui.button("Increment").clicked() {
            //     self.value += 1.0;
            // }

            ui.separator();

            let painter = self.create_drawing_widget(ui);

            // ╔══════════════╗
            // ║ Evolve model ║
            // ╚══════════════╝
            self.next_step();
            // ╔════════════════╗
            // ║ Draw new model ║
            // ╚════════════════╝
            self.draw_contents(painter);

            // ui.add(egui::github_link_file!(
            //     "https://github.com/emilk/eframe_template/blob/main/",
            //     "Source code."
            // ));

            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });

        ctx.request_repaint();
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
