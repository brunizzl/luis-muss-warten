use std::time::{Duration, Instant};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[serde(skip)]
    start_time: Instant,
    hidden_message: String,
    waiting_time: f32,
    #[serde(skip)]
    done_waiting: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            hidden_message: "SupersicherPasswort123".to_owned(),
            waiting_time: 2.7,
            done_waiting: false,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::widgets::global_dark_light_mode_buttons(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            let now = Instant::now();
            let diff = now.duration_since(self.start_time);
            let waiting_time = Duration::from_secs((self.waiting_time * 60.0) as u64);
            self.done_waiting |= diff > waiting_time;

            if self.done_waiting {
                ui.heading(format!("Luis muss nicht mehr warten :)"));
                ui.horizontal(|ui| {
                    ui.label("Hier bitte: ");
                    ui.text_edit_singleline(&mut self.hidden_message);
                });

                ui.add(
                    egui::Slider::new(&mut self.waiting_time, 1.0..=15.0)
                        .text("Wartezeit für Zukunft (min)"),
                );
            } else {
                ui.horizontal(|ui| {
                    ui.add(egui::widgets::Spinner::new());
                    let time_left = waiting_time - diff;
                    ui.heading(format!(
                        "Luis muss noch warten (für {} Sekunden)",
                        time_left.as_secs() + 1
                    ));
                });
            }

            ui.separator();
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
