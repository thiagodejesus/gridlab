use std::error::Error;

#[derive(Debug)]
pub struct GridError {
    message: String,
    description: String,
    error: Option<Box<dyn Error>>,
}

impl GridError {
    pub fn new(message: &str, description: &str, error: Option<Box<dyn Error>>) -> GridError {
        GridError {
            message: message.to_string(),
            description: description.to_string(),
            error,
        }
    }

    pub fn get_message(&self) -> String {
        format!("{}: {}", self.message, self.error.as_ref().unwrap())
    }
}
