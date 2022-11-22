/// Module with conversion to dot format
pub mod dot;
pub use dot::{render_dfa_dot_string, render_nt_dot_string};

/// Module with conversion to parol's PAR format
pub mod par;
pub use par::render_par_string;
