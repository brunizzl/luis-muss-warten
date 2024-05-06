use egui::Label;
use rand::{rngs::ThreadRng, Rng};
use std::time::{Duration, Instant};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct LuisApp {
    zoom: f32,

    #[serde(skip)]
    hidden_message: String,
    #[serde(skip)]
    new_message: String,

    specified_minutes_typing: f32,
    time_can_reverse: bool,

    #[serde(skip)]
    time_left: Duration,
    #[serde(skip)]
    last_typed: Instant,
    #[serde(skip)]
    last_frame: Instant,

    #[serde(skip)]
    char_to_type: egui::Key,
    #[serde(skip)]
    rng: ThreadRng,
}

impl Default for LuisApp {
    fn default() -> Self {
        let now = Instant::now();
        let mut rng = rand::thread_rng();
        for _ in 0..50 {
            //waste some values to get better start
            rng.gen::<usize>();
        }

        let mut res = Self {
            zoom: 1.0,

            hidden_message: "SupersicherPasswort123".to_owned(),
            new_message: String::new(),

            specified_minutes_typing: 1.0,
            time_can_reverse: false,

            time_left: Duration::from_secs(6000000),
            last_typed: now,
            last_frame: now,

            char_to_type: egui::Key::Space,
            rng,
        };
        res.change_char_to_type();
        res
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
    enum Shape {
        Upper,
        Lower,
        Other,
    }
    let mut last = Shape::Other;
    let mut streak = 0;
    for c in msg.chars() {
        if c.is_uppercase() {
            if matches!(last, Shape::Upper) {
                streak += 1;
            } else {
                streak = 1;
            }
            last = Shape::Upper;
        } else if c.is_lowercase() {
            if matches!(last, Shape::Lower) {
                streak += 1;
            } else {
                streak = 1;
            }
            last = Shape::Lower;
        } else {
            last = Shape::Other;
            streak = 0;
        }

        if streak > 2 {
            return Some("Zu wenig Varianz");
        }
    }

    None
}

const HIDDEN_MSG_KEY: &str = "app::hidden-message";

impl LuisApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let mut val: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            val.hidden_message = eframe::get_value(storage, HIDDEN_MSG_KEY).unwrap_or_default();
            return val;
        }

        Default::default()
    }

    fn change_char_to_type(&mut self) {
        const LEN: usize = 26;
        use egui::Key::*;
        const KEYS: [egui::Key; LEN] = [
            A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
        ];
        loop {
            let index: usize = self.rng.gen_range(0..LEN);
            let ch = KEYS[index];
            if ch != self.char_to_type {
                self.char_to_type = ch;
                return;
            }
        }
    }
}

impl eframe::App for LuisApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
        eframe::set_value(storage, HIDDEN_MSG_KEY, &self.hidden_message);
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
                ui.add(
                    egui::DragValue::new(&mut self.zoom)
                        .clamp_range(0.2..=2.0)
                        .fixed_decimals(1)
                        .update_while_editing(false),
                );
            });
            ctx.set_pixels_per_point(self.zoom * 3.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            if !self.time_left.is_zero() {
                let now = Instant::now();
                let frame_diff = now - self.last_frame;
                self.last_frame = now;

                let dur_since_typed = now - self.last_typed;
                if dur_since_typed < Duration::from_secs(3) {
                    self.time_left = self.time_left.saturating_sub(frame_diff);
                } else if dur_since_typed > Duration::from_secs(10) && self.time_can_reverse {
                    self.time_left += frame_diff;
                }

                let max_time = Duration::from_secs_f32(self.specified_minutes_typing * 60.0);
                if self.time_left > max_time {
                    self.time_left = max_time;
                }

                ctx.request_repaint_after(Duration::from_millis(100));
                let nr_secs = self.time_left.as_secs_f32().ceil();
                let secs = if nr_secs != 1.0 {
                    "Sekunden"
                } else {
                    "Sekunde"
                };
                ui.label(format!("Luis muss noch {} {} tippen. 🖮", nr_secs, secs));
                ui.add_space(5.0);
                ui.label(format!(
                    "Nächster Buchstabe:  {}",
                    self.char_to_type.symbol_or_name()
                ));

                ui.input(|info| {
                    if info.key_pressed(self.char_to_type) {
                        self.change_char_to_type();
                        self.last_typed = now;
                    }
                });
            } else {
                ui.label("Luis muss nicht mehr tippen! 🎉  🎊  🎆  🎇");

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
                        if !self.new_message.is_empty() {
                            ui.button("？").on_hover_ui(|ui| {
                                let text = r#"Das Passwort benötigt:
- mindestens 10 Zeichen
- davon mindestens zwei:
   - Großbuchstaben
   - Kleinbuchstaben
   - Ziffern
   - Sonderzeichen
- mit nicht mehr als zwei Groß-/
  Kleinbuchstaben hintereinander."#;
                                let rich = egui::RichText::new(text).size(8.5);
                                ui.add(egui::Label::new(rich).wrap(false));
                            });
                            let warning =
                                egui::RichText::new(err).color(egui::Color32::RED).strong();
                            ui.label(warning);
                        }
                    } else if ui.button("Übernehmen").clicked() {
                        self.hidden_message.clone_from(&self.new_message);
                    }
                });

                ui.add_space(20.0);
                ui.add(
                    egui::Slider::new(&mut self.specified_minutes_typing, 1.0..=15.0)
                        .text("Tippzeit in Zukunft (in Minuten)"),
                );

                ui.add_space(20.0);
                ui.add(egui::Checkbox::new(
                    &mut self.time_can_reverse,
                    "Zeit kann rückwärts laufen",
                ))
                .on_hover_text(
                    "Wenn 10 Sekunden nicht der richtige Buchstabe getippt \
                     wurde, wird die Restdauer wieder größer.",
                );
            }

            if !self.time_left.is_zero() {
                ui.add_space(50.0);
                ui.label("Stattdessen produktiv sein: ");
                ui.horizontal(|ui| {
                    ui.add(Label::new("                ").selectable(false));
                    let text = egui::RichText::new("😤").size(30.0);
                    if ui.button(text).on_hover_text("Huraaa!").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(10.0);
                ui.add(Label::new("   "))
                    .on_hover_text("Grüße von Bruno 🙋");
            }
        });
    }
}
