use rand::{rngs::ThreadRng, Rng};
use std::time::{Duration, Instant};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    scale: f32,

    hidden_message: String,
    #[serde(skip)]
    new_message: String,

    #[serde(skip)]
    start_time: Instant,
    waiting_time: f32,
    #[serde(skip)]
    done_waiting: bool,

    characters_to_type: usize,
    #[serde(skip)]
    characters_typed: usize,
    #[serde(skip)]
    curr_character_to_type: Option<egui::Key>,
    #[serde(skip)]
    done_typing: bool,
    #[serde(skip)]
    char_generator: ThreadRng,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            scale: 3.0,

            hidden_message: "SupersicherPasswort123".to_owned(),
            new_message: String::new(),

            start_time: Instant::now(),
            waiting_time: 1.0,
            done_waiting: false,

            characters_to_type: 20,
            characters_typed: 0,
            curr_character_to_type: None,
            done_typing: false,
            char_generator: rand::thread_rng(),
        }
    }
}

fn reason_message_is_bad(msg: &str) -> Option<&str> {
    if msg.len() < 10 {
        return Some("Zu kurz");
    }
    if msg.chars().filter(|c| c.is_numeric()).count() < 2 {
        return Some("Zu wenig Ziffern");
    }
    if msg.chars().filter(|c| c.is_uppercase()).count() < 2 {
        return Some("Zu wenig Großbuchstaben");
    }
    if msg.chars().filter(|c| c.is_lowercase()).count() < 2 {
        return Some("Zu wenig Kleinbuchstaben");
    }
    if msg.chars().filter(|c| !c.is_alphanumeric()).count() < 2 {
        return Some("Zu wenig Sonderzeichen");
    }

    None
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

            ui.horizontal(|ui| {
                egui::widgets::global_dark_light_mode_buttons(ui);
                ui.label("     Zoom: ");
                ui.add(egui::DragValue::new(&mut self.scale).clamp_range(0.2..=2.0));
            });            
            ctx.set_pixels_per_point(self.scale * 3.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            let now = Instant::now();
            let diff = now.duration_since(self.start_time);
            let waiting_time = Duration::from_secs((self.waiting_time * 60.0) as u64);
            self.done_waiting |= diff > waiting_time;
            
            if !self.done_waiting {
                ui.horizontal(|ui| {
                    ui.add(egui::widgets::Spinner::new());
                    let time_left = waiting_time - diff;
                    ui.label(format!(
                        "Luis muss noch {} Sekunden warten. ☕",
                        time_left.as_secs() + 1
                    ));
                });
                ui.add_space(20.0);
            }

            if self.curr_character_to_type.is_none() {
                const LEN: usize = 26;
                use egui::Key::*;
                const KEYS: [egui::Key; LEN] = [
                    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
                ];
                let index: usize = self.char_generator.gen_range(0..LEN);
                self.curr_character_to_type = Some(KEYS[index]);
            }
            if let Some(key) = self.curr_character_to_type {
                if !self.done_typing {
                    let nr_left = self.characters_to_type - self.characters_typed;
                    ui.label(format!(
                        "Luis muss noch {} Buchstaben tippen. 🖮  Nächster: {}",
                        nr_left,
                        key.symbol_or_name()
                    ));
                }

                ui.input(|info| {
                    if info.key_pressed(key) {
                        self.characters_typed += 1;
                        self.curr_character_to_type = None;
                        if self.characters_typed >= self.characters_to_type {
                            self.done_typing = true;
                        }
                    }
                });
            }

            if self.done_waiting && self.done_typing {
                ui.label("Luis muss nicht mehr warten und nicht mehr tippen. 🎉  🎊  🎆  🎇");
                
                ui.add_space(50.0);

                ui.horizontal(|ui| {
                    ui.label("Hier bitte: ");
                    ui.add(egui::Label::new(&self.hidden_message).selectable(false));
                });

                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.label("Passwort ändern: ");
                    ui.text_edit_singleline(&mut self.new_message);
                    if let Some(err) = reason_message_is_bad(&self.new_message) {
                        if self.new_message.len() > 0 {
                            let warning = egui::RichText::new(err).color(egui::Color32::RED).strong();
                            ui.label(warning);
                        }
                    }
                    else if ui.button("Übernehmen").clicked() {
                        self.hidden_message.clone_from(&self.new_message);
                    }
                });

                ui.add_space(20.0);
                ui.add(
                    egui::Slider::new(&mut self.waiting_time, 0.1..=15.0)
                        .text("Wartezeit in der Zukunft (min)"),
                );
                ui.add_space(20.0);
                ui.add(
                    egui::Slider::new(&mut self.characters_to_type, 5..=250)
                        .text("Anzahl Buchstaben in der Zukunft"),
                );
            }
        });
    }
}
