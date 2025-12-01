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
    pub target_ip: String,
    pub target_port: String,
    pub client_drop: String,
    pub server_drop: String,
    pub client_delay: String,
    pub server_delay: String,
    pub client_delay_time_min: String,
    pub client_delay_time_max: String,
    pub server_delay_time_min: String,
    pub server_delay_time_max: String,
    pub tx: mpsc::Sender<FormSubmission>,
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
    pub fn new(tx: mpsc::Sender<FormSubmission>, log_rx: mpsc::Receiver<String>) -> Self {
        Self {
            show_form: true,
            show_log: false,
            messages: vec![],
            log_rx: Some(log_rx),
            listen_ip: String::new(),
            listen_port: String::new(),
            target_ip: String::new(),
            target_port: String::new(),
            client_drop: String::new(),
            server_drop: String::new(),
            client_delay: String::new(),
            server_delay: String::new(),
            client_delay_time_min: String::new(),
            client_delay_time_max: String::new(),
            server_delay_time_min: String::new(),
            server_delay_time_max: String::new(),
            tx: tx,
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
                ui.label("Client Drop");
                ui.text_edit_singleline(&mut self.client_drop);
                ui.label("Server Drop");
                ui.text_edit_singleline(&mut self.server_drop);
                ui.label("Client Delay");
                ui.text_edit_singleline(&mut self.client_delay);
                ui.label("Server Delay");
                ui.text_edit_singleline(&mut self.server_delay);
                ui.label("Client Delay Time Min");
                ui.text_edit_singleline(&mut self.client_delay_time_min);
                ui.label("Client Delay Time Max");
                ui.text_edit_singleline(&mut self.client_delay_time_max);
                ui.label("Server Delay Time Min");
                ui.text_edit_singleline(&mut self.server_delay_time_min);
                ui.label("Server Delay Time Max");
                ui.text_edit_singleline(&mut self.server_delay_time_max);
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

                        if !self.client_drop.is_empty() &&
                            self.client_drop.parse::<u32>().is_err()
                        {
                            self.error_msg = Some("Client Drop Percentage must be a positive number".into());
                            return;
                        }
                        
                        if !self.server_drop.is_empty() &&
                            self.server_drop.parse::<u32>().is_err()
                        {
                            self.error_msg = Some("Server Drop Percentage must be a postive number".into());
                            return;
                        }

                        if !self.client_delay.is_empty() &&
                            self.client_delay.parse::<u32>().is_err()
                        {
                            self.error_msg = Some("Client Delay Percentage must be a positive number".into());
                            return;
                        }

                        if !self.server_delay.is_empty() &&
                            self.server_delay.parse::<u32>().is_err()
                        {
                            self.error_msg = Some("Server Delay Percentage must be a positive number".into());
                            return;
                        }

                        if !self.client_delay_time_min.is_empty() &&
                            self.client_delay_time_min.parse::<u32>().is_err()
                        {
                            self.error_msg = Some("Client delay time must be a positive number".into());
                            return;
                        }

                        if !self.client_delay_time_max.is_empty() &&
                            self.client_delay_time_max.parse::<u32>().is_err()
                        {
                            self.error_msg = Some("Client delay time must be a positive number".into());
                            return;
                        }

                        if !self.server_delay_time_min.is_empty() &&
                            self.server_delay_time_min.parse::<u32>().is_err()
                        {
                            self.error_msg = Some("Server delay time must be a positive number".into());
                            return;
                        }

                        if !self.server_delay_time_max.is_empty() &&
                            self.server_delay_time_max.parse::<u32>().is_err()
                        {
                            self.error_msg = Some("Server delay time must be a positive number".into());
                            return;
                        }

                        if self.server_delay_time_min.is_empty() && !self.server_delay_time_max.is_empty() ||
                        !self.server_delay_time_min.is_empty() && self.server_delay_time_max.is_empty() {
                            self.error_msg = Some("Either Pick Both or None for Timeouts".into());
                            return;
                        }

                        if self.client_delay_time_min.is_empty() && !self.client_delay_time_max.is_empty() ||
                        !self.client_delay_time_min.is_empty() && self.client_delay_time_max.is_empty() {
                            self.error_msg = Some("Either Pick Both or None for Timeouts".into());
                            return;
                        }

                        if !self.client_delay_time_min.is_empty() && !self.client_delay_time_max.is_empty() {
                            if let (Ok(min), Ok(max)) = (
                                self.client_delay_time_min.parse::<u32>(),
                                self.client_delay_time_max.parse::<u32>(),
                            ) {
                                if min > max {
                                    self.error_msg = Some("Min Timeout Cannot Exceed Max".into());
                                    return;
                                }
                            } else {
                                self.error_msg = Some("Timeout values must be valid numbers".into());
                                return;
                            }
                        }
                        
                        if !self.server_delay_time_min.is_empty() && !self.server_delay_time_max.is_empty() {
                            if let (Ok(min), Ok(max)) = (
                                self.server_delay_time_min.parse::<u32>(),
                                self.server_delay_time_max.parse::<u32>(),
                            ) {
                                if min > max {
                                    self.error_msg = Some("Min Timeout Cannot Exceed Max".into());
                                    return;
                                }
                            } else {
                                self.error_msg = Some("Timeout values must be valid numbers".into());
                                return;
                            }
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
                            client_drop: self.client_drop.clone(),
                            server_drop: self.server_drop.clone(),
                            client_delay: self.client_delay.clone(),
                            server_delay: self.server_delay.clone(),
                            client_delay_time_min: self.client_delay_time_min.clone(),
                            client_delay_time_max: self.client_delay_time_max.clone(),
                            server_delay_time_min: self.server_delay_time_min.clone(),
                            server_delay_time_max: self.server_delay_time_max.clone(),
                            log_ip: self.log_ip.clone(),
                            log_port: self.log_port.clone(),
                        };

                        self.show_form = false;
                        self.show_log = true;
                        let _ = self.tx.try_send(msg);
                    }

                    if ui.button("Clear").clicked() {
                        self.listen_ip.clear();
                        self.listen_port.clear();
                        self.target_ip.clear();
                        self.target_port.clear();
                        self.client_drop.clear();
                        self.server_drop.clear();
                        self.client_delay.clear();
                        self.server_delay.clear();
                        self.client_delay_time_min.clear();
                        self.client_delay_time_max.clear();
                        self.server_delay_time_min.clear();
                        self.server_delay_time_max.clear();
                        self.log_ip.clear();
                        self.log_port.clear();
                    }
                    
                });
            });
        }

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
        (TextStyle::Body, FontId::new(12.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(22.0, FontFamily::Monospace)),
        (TextStyle::Small, FontId::new(14.0, FontFamily::Monospace))
        ]
        .into();
    ctx.set_style(style);
}
