///
/// Common formatting for Token and OwnedToken
///
pub trait FormatToken {
    ///
    /// Generates a formatted position which an editor can follow via mouse click.
    ///
    fn format(&self, terminal_names: &'static [&'static str]) -> String;
}
