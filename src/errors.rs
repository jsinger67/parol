use miette::Diagnostic;

#[derive(Error, Diagnostic, Debug)]
pub enum ServerError {}
