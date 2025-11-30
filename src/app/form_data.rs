#[derive(Debug, Clone)]
pub struct FormData {
    pub(crate) openai_api_key: String,
    pub(crate) generation_model: String,
    pub(crate) host_port: String,
    pub(crate) ai_service_port: String,
    pub(crate) current_field: usize,
    pub(crate) editing: bool,
    pub(crate) error_message: String,
}

impl FormData {
    pub fn new() -> Self {
        Self {
            openai_api_key: String::new(),
            generation_model: "gpt-4o-mini".to_string(),
            host_port: "3000".to_string(),
            ai_service_port: "5555".to_string(),
            current_field: 0,
            editing: false,
            error_message: String::new(),
        }
    }

    pub fn validate(&mut self) -> bool {
        if self.openai_api_key.trim().is_empty() {
            self.error_message = "OpenAI API Key is required!".to_string();
            return false;
        }

        if !self.openai_api_key.starts_with("sk-") {
            self.error_message =
                "Invalid OpenAI API Key format (should start with 'sk-')".to_string();
            return false;
        }

        self.error_message.clear();
        true
    }

    pub fn get_current_value_mut(&mut self) -> &mut String {
        match self.current_field {
            0 => &mut self.openai_api_key,
            1 => &mut self.generation_model,
            2 => &mut self.host_port,
            3 => &mut self.ai_service_port,
            _ => &mut self.openai_api_key,
        }
    }
}
