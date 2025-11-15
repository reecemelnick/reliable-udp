use eframe::egui::{self, CentralPanel, Context, FontFamily, FontId, TextStyle, TopBottomPanel, ComboBox, ScrollArea};
use rss::Channel;

#[derive(Default)]
struct App {
    subs: Vec<(String, String)>,
    name_input: String,
    url_input: String,
    selected_value: Option<usize>,
    channel: Option<Channel>
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        set_styles(ctx);
        show_top_bar(ctx);
        CentralPanel::default().show(ctx, |ui| {
            self.show_from(ui);
            ui.separator();
            self.show_combo_box(ui);
            if let Some(channel) = &self.channel {
                ui.separator();
                ui.heading(channel.title());
                ui.label(channel.description());
                ui.separator();
                ScrollArea::vertical().show(ui, |ui| {
                    for item in channel.items() {
                        ui.heading(item.title().unwrap_or("No Title"));
                        if let Some(pat) = item.link() {
                            let _ = ui.link(pat);
                        }
                        ui.label(item.description().unwrap_or("No Description"));
                        ui.separator();
                    }
                });
            }
        });
    }
}

impl App {
    fn show_combo_box(&mut self, ui: &mut egui::Ui) {
        ComboBox::from_label("Select Rss")
            .selected_text(if let Some(index) = self.selected_value {
                if let Some(sub) = self.subs.get(index) {
                    &sub.0
                } else {
                    "Select me"
                }
            } else {
                "Select me"
            })
            .show_ui(ui, |ui| {
                for (i, rss) in self.subs.iter().enumerate() {
                    if ui.selectable_value(
                        &mut self.selected_value,
                        Some(i),
                        &rss.0,
                    ).clicked() {
                        if let Some(sub) = self.subs.get(i) {
                            match get_feed(&sub.1) {
                                Ok(channel) => self.channel = Some(channel),
                                Err(e) => println!("{}", e.to_string())
                            }
                        }
                    }
                }
            });
    }

    fn show_from(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("New Rss", |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut self.name_input);
                ui.label("Url");
                ui.text_edit_singleline(&mut self.url_input);
                ui.horizontal(|ui| {
                    if ui.button("Submit").clicked() {
                        self.subs.push((self.name_input.clone(), self.url_input.clone()));
                        self.name_input.clear();
                        self.url_input.clear();
                    }
    
                    if ui.button("Clear").clicked() {
                        self.name_input.clear();
                        self.url_input.clear();
                    }
                    
                });
            });
        });
    }
}


fn show_top_bar(ctx: &Context) {
    TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui|{
            ui.menu_button("File", |ui| {
                if ui.button("Exit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    });
}

fn main() -> Result<(), eframe::Error> {
    println!("Hello, world!");
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_resizable(true)
            .with_inner_size([520.0, 840.0]),
        ..Default::default()
    };
    eframe::run_native("reliable-udp", options, Box::new(|_cc|Ok(Box::<App>::default())))
}

fn set_styles(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(30.0, FontFamily::Monospace)),
        (TextStyle::Body, FontId::new(18.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(22.0, FontFamily::Monospace)),
        (TextStyle::Small, FontId::new(14.0, FontFamily::Monospace))
        ]
        .into();
    ctx.set_style(style);
}

fn get_feed(url: &str) -> Result<Channel, Box<dyn std::error::Error>> {
    let content = reqwest::blocking::get(url)?.bytes()?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}