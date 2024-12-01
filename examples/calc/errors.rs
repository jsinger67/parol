use parol_runtime::{FileSource, Location};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CalcError {
    #[error("{context} Undeclared variable")]
    UndeclaredVariable {
        context: String,
        input: FileSource,
        token: Location,
    },
}
