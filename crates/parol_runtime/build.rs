fn main() {
    let default_enabled = std::env::var("CARGO_FEATURE_DEFAULT").is_ok();
    let regex_automata_enabled = std::env::var("CARGO_FEATURE_REGEX_AUTOMATA").is_ok();

    if default_enabled && regex_automata_enabled {
        panic!("The features `default` and `regex_automata` are mutually exclusive.");
    }
}
