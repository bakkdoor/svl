use thiserror::Error;

pub type Result<T> = std::result::Result<T, SVLError>;

#[derive(Error, Debug)]
pub enum SVLError {
    #[error("IO error: {0:?}")]
    IOError(#[from] std::io::Error),
    #[error("Invalid state")]
    InvalidState,
    #[error("Unknown error: {0:?}")]
    Unknown(Option<String>)
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
