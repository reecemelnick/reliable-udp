#[derive(Debug, Clone)]
pub struct FormSubmission {
    pub listen_ip: String,
    pub listen_port: String,
    pub log_ip: String,
    pub log_port: String,
}

#[derive(Debug, Clone)]
pub struct FormSubmissionClean {
    pub listen_ip: String,
    pub listen_port: u16,
    pub log_ip: String,
    pub log_port: u16,
}

impl FormSubmission {
    pub fn with_defaults(&self) -> FormSubmissionClean {
        FormSubmissionClean {
            listen_ip: if self.listen_ip.is_empty() { "127.0.0.1".to_string() } else { self.listen_ip.clone() },
            listen_port: self.listen_port.parse().unwrap_or(9080),
            log_ip: if self.log_ip.is_empty() { "127.0.0.1".to_string() } else { self.log_ip.clone() },
            log_port: self.log_port.parse().unwrap_or(8000),
        }
    }
}