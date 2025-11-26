use eframe::egui::{self, CentralPanel, Context, FontFamily, FontId, TextStyle, TopBottomPanel, ScrollArea};
use tokio::sync::mpsc;
use crate::forms::FormSubmission;

pub struct App {
    pub show_form: bool,
    pub show_log: bool,
    pub messages: Vec<String>,
    pub log_rx: Option<mpsc::Receiver<String>>,
    pub listen_ip: String,
    pub listen_port: String,
    pub form_tx: mpsc::Sender<FormSubmission>,
    pub log_ip: String,
    pub log_port: String,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        set_styles(ctx);
        show_top_bar(ctx);
        CentralPanel::default().show(ctx, |ui| {
            self.show_from(ui);
            ui.separator();
            let MAX_MESSAGES = 100;
            if let Some(rx) = &mut self.log_rx {
                while let Ok(msg) = rx.try_recv() {
                    self.messages.push(msg);
                }
            }

            if self.messages.len() > MAX_MESSAGES {
                let excess = self.messages.len() - MAX_MESSAGES;
                self.messages.drain(0..excess);
            }

            ui.ctx().request_repaint();
        });
    }
}

impl App {
    pub fn new(form_tx: mpsc::Sender<FormSubmission>, log_rx: mpsc::Receiver<String>) -> Self {
        Self {
            show_form: true,
            show_log: false,
            messages: vec![],
            log_rx: Some(log_rx),
            listen_ip: String::new(),
            listen_port: String::new(),
            form_tx: form_tx,
            log_ip: String::new(),
            log_port: String::new(),
        }
    }
}

impl App {
    fn show_from(&mut self, ui: &mut egui::Ui) {
        if self.show_form {
            ui.vertical_centered_justified(|ui| {
                ui.label("Listen IP");
                ui.text_edit_singleline(&mut self.listen_ip);
                ui.label("Listen Port");
                ui.text_edit_singleline(&mut self.listen_port);
                ui.label("Log IP");
                ui.text_edit_singleline(&mut self.log_ip);
                ui.label("Log Port");
                ui.text_edit_singleline(&mut self.log_port);
                ui.horizontal(|ui| {
                    if ui.button("Submit").clicked() {
                        let msg = FormSubmission {
                            listen_ip: self.listen_ip.clone(),
                            listen_port: self.listen_port.clone(),
                            log_ip: self.log_ip.clone(),
                            log_port: self.log_port.clone(),
                        };

                        self.show_form = false;
                        self.show_log = true;
                        let _ = self.form_tx.try_send(msg);
                    }

                    if ui.button("Clear").clicked() {
                        self.listen_ip.clear();
                        self.listen_port.clear();
                        self.log_ip.clear();
                        self.log_port.clear();
                    }
                    
                });
            });
        }

        ui.separator();

        if self.show_log {
            ui.separator();
            ui.label("Log Output:");
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(600.0)
                .show(ui, |ui| {
                    for line in &self.messages {
                        ui.label(line);
                    }
                });
        }
    }
}


fn show_top_bar(ctx: &Context) {
    TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui|{
            if ui.button("Exit").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    });
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
