#[derive(Debug, Clone, Copy)]
pub enum Status {
    Error,
    Success,
    Info,
    Warning,
}

impl Status {
    pub fn class_name(&self) -> &'static str {
        match self {
            Status::Error => "error",
            Status::Success => "success",
            Status::Info => "info",
            Status::Warning => "warning",
        }
    }

    pub fn from_str(class: &str) -> Option<Self> {
        match class {
            "error" => Some(Status::Error),
            "success" => Some(Status::Success),
            "info" => Some(Status::Info),
            "warning" => Some(Status::Warning),
            _ => None,
        }
    }
}
