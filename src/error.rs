use crate::types::PlayerChoice;


#[derive(Debug)]
pub enum Error {
    /// The player tried to split a hand that cannot be split
    CannotSplit,
    /// The player tried to double down when it is not allowed
    CannotDoubleDown,
    /// Invalid input was provided
    InvalidInput(String),
    /// Invalid choice
    InvalidChoice(PlayerChoice),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_message = match self {
            Error::CannotSplit => "You cannot split this hand.".to_string(),
            Error::CannotDoubleDown => "You cannot double down at this time.".to_string(),
            Error::InvalidInput(msg) => format!("Invalid input: {}", msg),
            Error::InvalidChoice(choice) => format!("Invalid choice: {}", choice),
        };
        write!(f, "{}", error_message)
    }
}
