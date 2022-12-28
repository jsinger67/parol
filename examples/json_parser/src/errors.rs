use std::num::ParseFloatError;

use parol_runtime::{FileSource, Location};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JsonError {
    #[error("f64 parse error")]
    ParseF64Failed {
        input: FileSource,
        token: Location,
        source: ParseFloatError,
    },
}
