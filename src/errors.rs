#[derive(Error, Debug)]
pub enum RuntimeError {
    /// Error from the id_tree crate
    #[error(transparent)]
    IdTreeError {
        #[from]
        source: id_tree::NodeIdError,
    },

    /// Error in generated parser data
    #[error("{0}")]
    DataError(&'static str),

    /// Syntax error in input prevents prediction of next production
    #[error("{0}")]
    PredictionError(String),

    /// Unexpected internal state
    #[error("{0}")]
    InternalError(String),
}
