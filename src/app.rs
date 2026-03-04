// -- Types: --------------------------------------------------------------
type Point2D = Pos2;

// -- Uses: ---------------------------------------------------------------
use delegate::delegate;
use egui::{emath, pos2, Color32, Frame, Pos2, Rect, Sense, Stroke, Ui, Vec2, Window};

// -- Traits: -------------------------------------------------------------

trait AppUi {
    fn create_drawing_widget(&mut self, ui: &mut Ui) -> egui::Response;
    fn create_stroke_widget(&mut self, ui: &mut Ui) -> egui::Response;
    fn draw_point(&mut self, p: Point2D, zoom: f32, painter: &egui::Painter);
    fn draw_lines(&mut self, lines: &Vec<Pos2>, painter: &egui::Painter);
}

// -- Types: --------------------------------------------------------------
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FallingSandApp {
    // Data
    data: u8,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FallingSandAppUi {
    pub fsapp: FallingSandApp,
    pub stroke: Stroke,
}

// -- Impl: ---------------------------------------------------------------
impl Default for FallingSandApp {
    fn default() -> Self {
        Self { data: 0 }
    }
}

impl Default for FallingSandAppUi {
    fn default() -> Self {
        Self {
            // Example stuff:
            stroke: Stroke::new(2.0, Color32::LIGHT_RED.linear_multiply(1.25)),
            fsapp: Default::default(),
        }
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
}

// -- Impl For: -----------------------------------------------------------
impl AppUi for FallingSandAppUi {
    fn create_drawing_widget(&mut self, ui: &mut Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(
            Vec2::new(ui.available_width(), ui.available_height() - 50.0),
            Sense::CLICK,
        );

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let from_screen = to_screen.inverse();

        // 2. Comprobamos el click izquierdo
        if response.clicked() {
            println!("¡Click izquierdo detectado en el Painter!");
            if let Some(pos) = response.interact_pointer_pos() {
                println!("Click en la posición: {:?}", pos);
            }
        }

        // 3. Dibujamos algo basado en el estado
        let color = if response.hovered() {
            // egui::Color32::RED
            self.stroke.color
        } else {
            egui::Color32::BLUE
        };
        painter.circle_filled(response.rect.center(), 40.0, color);

        response
    }

    fn create_stroke_widget(&mut self, ui: &mut egui::Ui) -> egui::Response {
        //let mut stroke: Stroke = self.stroke;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Grains props.").color(egui::Color32::YELLOW));
            ui.add(&mut self.stroke);
            ui.separator();
            if ui.button("Clear Canvas").clicked() {
                //self.lines.clear();
            }
        })
        .response
    }

    fn draw_point(&mut self, p: Point2D, zoom: f32, painter: &egui::Painter) {
        // También puedes obtener los límites
        // let min = painter.clip_rect().min; // Esquina superior izquierda (Pos2)
        // let max = painter.clip_rect().max; // Esquina inferior derecha (Pos2)

        let centro = pos2(p.x, p.y);
        let mut radio = zoom;
        // let radio = zoom.min(3.5);
        // let radio = ((zoom + 0.125) / 2.5).max(3.5);
        // let color = Color32::from_rgb(255, 255, 255);
        let color = Color32::CYAN;

        if zoom < 1.5 {
            radio = 1.5;
        }
        if zoom > 4.0 {
            radio = 4.0;
        }

        painter.circle_filled(centro, radio, color);
    }

    fn draw_lines(&mut self, lines: &Vec<Pos2>, painter: &egui::Painter) {
        let stroke = Stroke::new(0.5, egui::Color32::LIGHT_YELLOW);
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
