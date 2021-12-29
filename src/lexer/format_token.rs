use std::fmt::Debug;

///
/// Common formatting for Token and OwnedToken
///
pub trait FormatToken {
    ///
    /// Generates a formatted position which an editor can follow via mouse click.
    ///
    fn format<T>(&self, file_name: &T, terminal_names: &'static [&'static str]) -> String
    where
        T: Debug;
}
