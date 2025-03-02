#[derive(thiserror::Error, Debug)]
pub enum InternalError {
    #[error("Generic error: {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    TomlParseError(#[from] toml::de::Error),
}
