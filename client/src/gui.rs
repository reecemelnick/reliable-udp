use eframe::egui::{self, CentralPanel, Context, FontFamily, FontId, TextStyle, TopBottomPanel, ScrollArea};
use tokio::sync::mpsc;
use crate::forms::FormSubmission;

pub struct App {
    pub show_form: bool,
    pub show_log: bool,
    pub messages: Vec<String>,
    pub log_rx: Option<mpsc::Receiver<String>>,
    pub input_tx: mpsc::Sender<String>,
    pub input_buffer: String,
    pub listen_ip: String,
    pub listen_port: String,
    pub target_ip: String,
    pub target_port: String,
    pub timeout: String,
    pub max_retries: String,
    pub form_tx: mpsc::Sender<FormSubmission>,
    pub log_ip: String,
    pub log_port: String,
    pub error_msg: Option<String>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        set_styles(ctx);
        show_top_bar(ctx);
        CentralPanel::default().show(ctx, |ui| {
            self.show_from(ui);
            ui.separator();
            let max_messages = 100;
            if let Some(rx) = &mut self.log_rx {
                while let Ok(msg) = rx.try_recv() {
                    self.messages.push(msg);
                }
            }

            if self.messages.len() > max_messages {
                let excess = self.messages.len() - max_messages;
                self.messages.drain(0..excess);
            }

            ui.ctx().request_repaint();
        });
    }
}

impl App {
    pub fn new(form_tx: mpsc::Sender<FormSubmission>, log_rx: mpsc::Receiver<String>, input_tx: mpsc::Sender<String>) -> Self {
        Self {
            show_form: true,
            show_log: false,
            messages: vec![],
            log_rx: Some(log_rx),
            input_tx: input_tx,
            input_buffer: String::new(),
            listen_ip: String::new(),
            listen_port: String::new(),
            target_ip: String::new(),
            target_port: String::new(),
            timeout: String::new(),
            max_retries: String::new(),
            form_tx: form_tx,
            log_ip: String::new(),
            log_port: String::new(),
            error_msg: None,
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
                ui.label("Target IP");
                ui.text_edit_singleline(&mut self.target_ip);
                ui.label("Target Port");
                ui.text_edit_singleline(&mut self.target_port);
                ui.label("Timeout");
                ui.text_edit_singleline(&mut self.timeout);
                ui.label("Max Retries");
                ui.text_edit_singleline(&mut self.max_retries);
                ui.label("Log IP");
                ui.text_edit_singleline(&mut self.log_ip);
                ui.label("Log Port");
                ui.text_edit_singleline(&mut self.log_port);
                
                if let Some(err) = &self.error_msg {
                    ui.colored_label(egui::Color32::RED, err);
                }

                ui.horizontal(|ui| {
                    if ui.button("Submit").clicked() {
                        
                        if !self.listen_ip.is_empty() &&
                            self.listen_ip.parse::<std::net::IpAddr>().is_err()
                        {
                            self.error_msg = Some("Invalid Listen IP".into());
                            return;
                        }

                        if !self.listen_port.is_empty() &&
                            self.listen_port.parse::<u16>().is_err()
                        {
                            self.error_msg = Some("Listen Port must be a valid port".into());
                            return;
                        }

                        if !self.target_ip.is_empty() &&
                            self.target_ip.parse::<std::net::IpAddr>().is_err()
                        {
                            self.error_msg = Some("Invalid Target IP".into());
                            return;
                        }
                        
                        if !self.target_port.is_empty() &&
                            self.target_port.parse::<u16>().is_err()
                        {
                            self.error_msg = Some("Target Port must be a valid port".into());
                            return;
                        }

                        if !self.timeout.is_empty() &&
                            self.timeout.parse::<u16>().is_err()
                        {
                            self.error_msg = Some("Timeout must be a positive number".into());
                            return;
                        }
                        
                        if !self.max_retries.is_empty() &&
                            self.max_retries.parse::<u16>().is_err()
                        {
                            self.error_msg = Some("Max Retries needs to be a positive number".into());
                            return;
                        }

                        if !self.log_ip.is_empty() &&
                            self.log_ip.parse::<std::net::IpAddr>().is_err()
                        {
                            self.error_msg = Some("Invalid Log IP".into());
                            return;
                        }

                        
                        if !self.log_port.is_empty() &&
                            self.log_port.parse::<u16>().is_err()
                        {
                            self.error_msg = Some("Log Port must be a number".into());
                            return;
                        }

                        let msg = FormSubmission {
                            listen_ip: self.listen_ip.clone(),
                            listen_port: self.listen_port.clone(),
                            target_ip: self.target_ip.clone(),
                            target_port: self.target_port.clone(),
                            timeout: self.timeout.clone(),
                            max_retries: self.max_retries.clone(),
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
                        self.target_ip.clear();
                        self.target_port.clear();
                        self.timeout.clear();
                        self.max_retries.clear();
                        self.log_ip.clear();
                        self.log_port.clear();
                    }
                    
                });
            });
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.input_buffer);

            if ui.button("Send").clicked() {
                if !self.input_buffer.is_empty() {
                    let _ = self.input_tx.try_send(self.input_buffer.clone());
                    self.input_buffer.clear();
                }
            }
        });
        
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
