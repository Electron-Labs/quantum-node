
#[derive( Debug, Clone)]
pub enum CustomError {
    // #[resp("{0}")]
    DB(String),
}


impl std::fmt::Display for CustomError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
           CustomError::DB(err_msg) => write!(fmt, "Error {}.", err_msg),
           _ => write!(fmt, "An unknown error occurred"), 
        }
    }
}

impl std::error::Error for CustomError {}