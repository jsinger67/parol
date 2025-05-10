use crate::{generate_name, Cfg, Pr, Symbol, SymbolAttribute};

/// Augment the grammar with a new start symbol if the current start symbol has more than one
/// production.
/// The new start symbol is created by adding a new production to the grammar at the beginning.
/// This is necessary for LR parsing to have a single start production with implicit EOF at the end.
pub fn augment_grammar(cfg: &Cfg) -> Cfg {
    let start_symbol_production_count = cfg.matching_productions(&cfg.st).len();
    if start_symbol_production_count == 1 {
        return cfg.clone();
    }
    let mut new_cfg = cfg.clone();
    let new_start = generate_name(cfg.get_non_terminal_set().iter(), cfg.st.clone());
    new_cfg.st.clone_from(&new_start);
    new_cfg.pr.insert(
        0,
        Pr::new(
            &new_start,
            vec![Symbol::N(cfg.st.clone(), SymbolAttribute::None, None, None)],
        ),
    );
    new_cfg
}
