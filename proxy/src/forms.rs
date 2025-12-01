
#[derive(Debug, Clone)]
pub struct FormSubmission {
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
    pub log_ip: String,
    pub log_port: String,
}

#[derive(Debug, Clone)]
pub struct FormSubmissionClean {
    pub listen_ip: String,
    pub listen_port: u16,
    pub target_ip: String,
    pub target_port: u16,
    pub client_drop: i32,
    pub server_drop: i32,
    pub client_delay: i32,
    pub server_delay: i32,
    pub client_delay_time_min: usize,
    pub client_delay_time_max: usize,
    pub server_delay_time_min: usize,
    pub server_delay_time_max: usize,
    pub log_ip: String,
    pub log_port: u16,
}

impl FormSubmission {
    pub fn with_defaults(&self) -> FormSubmissionClean {
        FormSubmissionClean {
            listen_ip: if self.listen_ip.is_empty() { "127.0.0.1".to_string() } else { self.listen_ip.clone() },
            listen_port: self.listen_port.parse().unwrap_or(8080),
            target_ip: if self.target_ip.is_empty() { "127.0.0.1".to_string() } else { self.target_ip.clone() },
            target_port: self.target_port.parse().unwrap_or(9080),
            client_drop: self.client_drop.parse().unwrap_or(0),
            server_drop: self.server_drop.parse().unwrap_or(0),
            client_delay: self.client_delay.parse().unwrap_or(0),
            server_delay: self.server_delay.parse().unwrap_or(0),
            client_delay_time_min: self.client_delay_time_min.parse().unwrap_or(0),
            client_delay_time_max: self.client_delay_time_max.parse().unwrap_or(0),
            server_delay_time_min: self.server_delay_time_min.parse().unwrap_or(0),
            server_delay_time_max: self.server_delay_time_max.parse().unwrap_or(0),
            log_ip: if self.log_ip.is_empty() { "127.0.0.1".to_string() } else { self.log_ip.clone() },
            log_port: self.log_port.parse().unwrap_or(8000),
        }
    }
}