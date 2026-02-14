mod format;
pub(crate) use format::Format;

mod indent;

mod fmt_options;
pub(crate) use fmt_options::{FmtOptions, LineEnd, Padding};

mod settings;
pub(crate) use settings::FormattingSettings;

mod line;
pub(crate) use line::Line;

mod comments;
pub(crate) use comments::Comments;

mod context;
