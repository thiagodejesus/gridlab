#[derive(Debug)]
pub struct GridError {
    message: String,
}

impl GridError {
    pub fn new(message: &str) -> GridError {
        GridError {
            message: message.to_string(),
        }
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}
