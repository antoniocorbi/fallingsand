// -- Types: --------------------------------------------------------------
type Point2D = Pos2;

// -- Uses: ---------------------------------------------------------------
use delegate::delegate;
use egui::{
    emath::{self, RectTransform},
    pos2, Color32, Frame, Pos2, Rect, Sense, Stroke, Ui, Vec2, Window,
};

// -- Traits: -------------------------------------------------------------

trait AppUi {
    fn create_drawing_widget(&mut self, ui: &mut Ui) -> egui::Response;
    fn create_stroke_widget(&mut self, ui: &mut Ui) -> egui::Response;
    fn draw_point(&mut self, p: Point2D, color: Color32, zoom: f32, painter: &egui::Painter);
    fn draw_lines(&mut self, lines: &Vec<Pos2>, color: Color32, painter: &egui::Painter);
}

// -- Types: --------------------------------------------------------------
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FallingSandApp {
    // Data
    data: Vec<Vec<u8>>,
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
            data: vec![vec![0; 256]; 256],
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

        Self {
            // Example stuff:
            fsapp: Default::default(),
            stroke: Stroke::new(2.0, Color32::LIGHT_RED.linear_multiply(1.25)),
            world_rect,
            screen_rect,
            w2s,
            s2w,
        }
    }
}

impl FallingSandApp {
    fn nrows(&self) -> usize {
        self.data.len()
    }

    fn ncols(&self) -> usize {
        self.data[0].len()
    }

    fn world_rect(&self) -> Rect {
        let min = pos2(0.0, 0.0);
        let size: Vec2 = Vec2::new(self.ncols() as f32, self.nrows() as f32);
        Rect::from_min_size(min, size)
    }
}

impl FallingSandAppUi {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    pub fn update_transforms(&mut self, screen_rect: Rect) {
        // Store the canvas rect
        self.screen_rect = screen_rect;

        // Compute world2screen and screen2world
        self.w2s = emath::RectTransform::from_to(self.world_rect, self.screen_rect);
        self.s2w = self.w2s.inverse();
    }

    pub fn pos2_to_screen(&self, pos: Pos2) -> Pos2 {
        self.w2s.transform_pos(pos)
    }

    pub fn pos2_to_world(&self, pos: Pos2) -> Pos2 {
        self.s2w.transform_pos(pos)
    }

    pub fn rect_to_screen(&self, rect: Rect) -> Rect {
        self.w2s.transform_rect(rect)
    }

    pub fn rect_to_world(&self, rect: Rect) -> Rect {
        self.s2w.transform_rect(rect)
    }
}

// -- Impl For: -----------------------------------------------------------
impl AppUi for FallingSandAppUi {
    fn create_drawing_widget(&mut self, ui: &mut Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(
            Vec2::new(ui.available_width(), ui.available_height() - 50.0),
            Sense::CLICK,
        );

        // Compute transforms: w2s + s2w
        self.update_transforms(response.rect);

        // 2. Comprobamos el click izquierdo
        if response.secondary_clicked() {
            println!("¡Click derecho detectado en el Painter!");
            if let Some(pos) = response.interact_pointer_pos() {
                println!("Click en la posición: {:?}", pos);
            }
        }

        if response.middle_clicked() {
            println!("¡Click central detectado en el Painter!");
            if let Some(pos) = response.interact_pointer_pos() {
                println!("Click en la posición: {:?}", pos);
            }
        }

        if response.clicked() {
            println!("¡Click izquierdo detectado en el Painter!");
            if let Some(pos) = response.interact_pointer_pos() {
                println!(
                    "Click en la posición screen:{:?} / world: {:?}",
                    pos,
                    self.pos2_to_world(pos)
                );
            }
        }

        // 3. Dibujamos algo basado en el estado
        let color = if response.hovered() {
            // egui::Color32::RED
            self.stroke.color
        } else {
            egui::Color32::LIGHT_GRAY
        };
        //painter.circle_filled(response.rect.center(), 40.0, color);
        self.draw_point(response.rect.center(), color, 40.0, &painter);

        response
    }

    fn create_stroke_widget(&mut self, ui: &mut egui::Ui) -> egui::Response {
        //let mut stroke: Stroke = self.stroke;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Grains props.").color(egui::Color32::GREEN));
            ui.add(&mut self.stroke);
            ui.separator();
            if ui.button("Clear Canvas").clicked() {
                //self.lines.clear();
            }
        })
        .response
    }

    fn draw_point(&mut self, p: Point2D, color: Color32, zoom: f32, painter: &egui::Painter) {
        // También puedes obtener los límites
        // let min = painter.clip_rect().min; // Esquina superior izquierda (Pos2)
        // let max = painter.clip_rect().max; // Esquina inferior derecha (Pos2)

        let centro = pos2(p.x, p.y);
        let mut radio = zoom;
        // let radio = zoom.min(3.5);
        // let radio = ((zoom + 0.125) / 2.5).max(3.5);
        // let color = Color32::from_rgb(255, 255, 255);

        if zoom < 1.5 {
            radio = 1.5;
        }
        // if zoom > 4.0 {
        //     radio = 4.0;
        // }

        painter.circle_filled(centro, radio, color);
    }

    fn draw_lines(&mut self, lines: &Vec<Pos2>, color: Color32, painter: &egui::Painter) {
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

            self.create_drawing_widget(ui);

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
