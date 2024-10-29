#[derive(Clone)]
pub struct Logger {
    pub context: String,
}

impl Logger {
    pub fn new(context: String) -> Logger {
        Logger { context }
    }

    pub fn append_context(&self, context: String) -> Logger {
        Logger {
            context: format!("{}: {}", self.context, context),
        }
    }

    fn log(&self, message: &str) {
        println!("{}: {}", self.context, message);
    }

    pub fn info(&self, message: &str) {
        self.log(message);
    }

    pub fn error(&self, message: &str) {
        self.log(message);
    }
}
