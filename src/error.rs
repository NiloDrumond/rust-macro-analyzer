#[derive(Clone, Debug)]
pub struct Error {
    pub path: Option<String>,
    pub message: ErrorMessage,
}

#[derive(Clone, Debug)]
pub enum ErrorMessage {
    DeriveMacroExpectedTokenTree,
    FailedToReadDirectory,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "[Path: {}] Error: {:?}", path, self.message)
        } else {
            write!(f, "Error: {:?}", self.message)
        }
    }
}

impl Error {
    pub fn add_path(&self, path: &str) -> Self {
        Self {
            message: self.message.clone(),
            path: Some(path.to_string()),
        }
    }
}

impl std::error::Error for Error {}
