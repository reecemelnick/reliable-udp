#[derive(Debug, Clone)]
pub struct FormSubmission {
    pub listen_ip: String,
    pub listen_port: String,
    pub target_ip: String,
    pub target_port: String,
    pub timeout: String,
    pub max_retries: String,
    pub log_ip: String,
    pub log_port: String,
}

#[derive(Debug, Clone)]
pub struct FormSubmissionClean {
    pub listen_ip: String,
    pub listen_port: u16,
    pub target_ip: String,
    pub target_port: u16,
    pub timeout: usize,
    pub max_retries: i32,
    pub log_ip: String,
    pub log_port: u16,
}

impl FormSubmission {
    pub fn with_defaults(&self) -> FormSubmissionClean {
        FormSubmissionClean {
            listen_ip: if self.target_ip.is_empty() { "127.0.0.1".to_string() } else { self.listen_ip.clone() },
            listen_port: self.listen_port.parse().unwrap_or(7080),
            target_ip: if self.target_ip.is_empty() { "127.0.0.1".to_string() } else { self.target_ip.clone() },
            target_port: self.target_port.parse().unwrap_or(8080),
            timeout: self.timeout.parse().unwrap_or(5000),
            max_retries: self.max_retries.parse().unwrap_or(3),
            log_ip: if self.log_ip.is_empty() { "127.0.0.1".to_string() } else { self.log_ip.clone() },
            log_port: self.log_port.parse().unwrap_or(8000),
        }
    }
}