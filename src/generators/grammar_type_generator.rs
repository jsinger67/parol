use crate::analysis::lookahead_dfa::ProductionIndex;
use crate::generators::{generate_terminal_name, NamingHelper as NmHlp};
use crate::grammar::{ProductionAttribute, SymbolAttribute};
use crate::{Cfg, Pr, Symbol, Terminal};
use log::trace;
use miette::{miette, IntoDiagnostic, Result};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Type information used for auto-generation
///
#[derive(Debug, Clone, PartialEq)]
pub enum ASTType {
    /// Not specified
    None,
    /// Unit type ()
    Unit,
    /// Will be generated as Token structure
    Token(String),
    /// A type name
    TypeRef(String),
    /// A struct, i.e. a named collection of (name, type) tuples
    Struct(String, Vec<(String, ASTType)>),
    /// Will be generated as enum with given name
    Enum(String, Vec<ASTType>),
    /// Will be generated as Vec<T> where T is the type, similar to TypeRef
    Repeat(String),
}

impl ASTType {
    pub(crate) fn type_name(&self) -> String {
        match self {
            Self::None => "*TypeError*".to_owned(),
            Self::Unit => "()".to_owned(),
            Self::Token(t) => format!("OwnedToken /* {} */", t),
            Self::TypeRef(r) => format!("Box<{}>", r),
            Self::Struct(n, _) => n.to_string(),
            Self::Enum(n, _) => n.to_string(),
            Self::Repeat(r) => format!("Vec<{}>", r),
        }
    }
}

impl Display for ASTType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::None => write!(f, "-"),
            Self::Unit => write!(f, "()"),
            Self::Token(t) => write!(f, "OwnedToken /* {} */", t),
            Self::TypeRef(r) => write!(f, "Box<{}>", r),
            Self::Struct(n, m) => write!(
                f,
                "struct {} {{ {} }}",
                n,
                m.iter()
                    .map(|(n, t)| format!("{}: {}", n, t))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Enum(n, t) => write!(
                f,
                "enum {} {{ {} }}",
                n,
                t.iter()
                    .enumerate()
                    .map(|(i, t)| format!("{}{}({})", n, i, t))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Repeat(r) => write!(f, "Vec<{}>", r),
        }
    }
}

impl Default for ASTType {
    fn default() -> Self {
        Self::None
    }
}

///
/// An argument of a semantic action
///
#[derive(Builder, Clone, Debug, Default)]
pub struct Argument {
    /// Argument's name
    pub(crate) name: String,
    /// Argument's type
    pub(crate) arg_type: ASTType,
    /// Argument index or position
    pub(crate) index: Option<usize>,
    /// Indicates if the argument is used
    pub(crate) used: bool,
    /// Semantic information
    pub(crate) sem: SymbolAttribute,
}

impl Argument {
    /// Set the argument's name
    pub fn name(&mut self, name: String) -> &mut Self {
        self.name = NmHlp::to_lower_snake_case(&name);
        self
    }
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        let arg_index = if let Some(index) = self.index {
            format!("_{}", index)
        } else {
            String::default()
        };

        write!(
            f,
            "{}{}{}: {}",
            NmHlp::item_unused_indicator(self.used),
            self.name,
            arg_index,
            self.arg_type.type_name()
        )
    }
}

///
/// A semantic action
///
/// For each production there exists an associated semantic action.
/// Any action has a kind of `input` information which can be deduced from the production's
/// right-hand side and resemble the action's argument list.
/// These arguments are feed at prase time by the parser automatically.
/// But in practice not all arguments provided by the parser are actually used because actions have
/// tow possible ways to obtain their input value:
///
/// * First the corresponding values can be obtained from the actions's parameter list
/// * Second the values can be popped from the AST stack
///
/// The first way would actually be used for simple tokens.
/// The second way is applicable if there are already more complex items on the AST stack which
/// is the case for any non-terminals.
///
///
#[derive(Builder, Clone, Debug, Default)]
pub struct Action {
    /// Associated non-terminal
    pub(crate) non_terminal: String,

    /// Production number
    /// The production index is identical for associated actions and productions, i.e. you can use
    /// this index in Cfg.pr and in GrammarTypeInfo.actions to obtain a matching pair of
    /// production and action.
    pub(crate) prod_num: ProductionIndex,

    /// The function name
    pub(crate) fn_name: String,

    /// The argument list as they are provided by the parser
    pub(crate) args: Vec<Argument>,

    /// The output type, i.e. the return type of the action which corresponds to the constructed
    /// new value pushed on the AST stack.
    /// If there exists an associated semantic action of the user's `input` grammar this type is
    /// used to call it with.
    pub(crate) out_type: ASTType,

    /// Semantic specification
    pub(crate) sem: ProductionAttribute,
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(
            f,
            "/* {} */ (({}) -> {})  {{ {} }}",
            self.prod_num,
            self.args
                .iter()
                .map(|a| a.arg_type.type_name())
                .collect::<Vec<String>>()
                .join(", "),
            self.out_type,
            self.sem,
        )
    }
}

///
/// Type information for a given grammar
///
#[derive(Debug, Default)]
pub struct GrammarTypeInfo {
    /// All semantic actions, indices correspond to production indices in Cfg
    pub(crate) actions: Vec<Action>,

    /// Calculated type of non-terminals
    pub(crate) non_terminal_types: BTreeMap<String, ASTType>,

    /// Helper
    terminals: Vec<String>,
    terminal_names: Vec<String>,
}

impl GrammarTypeInfo {
    /// Add non-terminal type
    fn add_non_terminal_type(&mut self, non_terminal: &str, nt_type: ASTType) -> Result<()> {
        self.non_terminal_types
            .insert(non_terminal.to_owned(), nt_type)
            .map_or_else(
                || {
                    trace!("Setting type for non-terminal {}", non_terminal);
                    Ok(())
                },
                |_| {
                    Err(miette!(
                        "Type for non-terminal {} already specified",
                        non_terminal
                    ))
                },
            )
    }

    fn build_argument_list(&self, prod: &Pr) -> Result<Vec<Argument>> {
        let mut types = prod.get_r().iter().filter(|s| s.is_t() || s.is_n()).fold(
            Ok(Vec::new()),
            |acc, s| {
                acc.and_then(|mut acc| {
                    Self::deduce_type_of_symbol(s).map(|t| {
                        acc.push((t, s.attribute()));
                        acc
                    })
                })
            },
        )?;

        Ok(
            NmHlp::generate_member_names(prod.get_r(), &self.terminals, &self.terminal_names)
                .iter()
                .enumerate()
                .zip(types.drain(..))
                .map(|((i, n), (t, a))| {
                    // Tokens are taken from the parameter list per definition.
                    let used = matches!(t, ASTType::Token(_));
                    ArgumentBuilder::default()
                        .name(n.to_string())
                        .arg_type(t)
                        .used(used)
                        .index(Some(i))
                        .sem(a)
                        .build()
                        .unwrap()
                })
                .collect::<Vec<Argument>>(),
        )
    }

    fn deduce_action_type_from_production(&self, prod: &Pr) -> Result<ASTType> {
        match prod.effective_len() {
            0 => match prod.2 {
                ProductionAttribute::None => Ok(ASTType::Unit), // Normal empty production
                ProductionAttribute::CollectionStart => Ok(ASTType::Repeat(
                    NmHlp::to_upper_camel_case(prod.0.get_n_ref().unwrap()),
                )),
                ProductionAttribute::AddToCollection => Err(miette!(
                    "AddToCollection attribute should not be applied on an empty production"
                )),
            },
            _ => Ok(self.struct_data_of_production(prod)?),
        }
    }

    /// Creates the list of actions from the Cfg.
    fn deduce_actions(&mut self, cfg: &Cfg) -> Result<()> {
        self.actions = Vec::with_capacity(cfg.pr.len());
        for (i, pr) in cfg.pr.iter().enumerate() {
            self.actions.push(
                ActionBuilder::default()
                    .non_terminal(pr.get_n())
                    .prod_num(i)
                    .fn_name(NmHlp::to_lower_snake_case(&format!(
                        "{}_{}",
                        pr.get_n_str(),
                        i
                    )))
                    .args(self.build_argument_list(pr)?)
                    .out_type(self.deduce_action_type_from_production(pr)?)
                    .sem(pr.2.clone())
                    .build()
                    .into_diagnostic()?,
            );
        }
        Ok(())
    }

    fn deduce_type_of_non_terminal(&mut self, actions: Vec<usize>) -> Option<ASTType> {
        match actions.len() {
            // Productions can be optimized away, when they have duplicates!
            0 => None,
            // Only one production for this non-terminal: we take the out-type of the single action
            1 => Some(self.actions[actions[0]].out_type.clone()),
            _ => {
                let actions = actions
                    .iter()
                    .map(|i| &self.actions[*i])
                    .collect::<Vec<&Action>>();
                match &actions[..] {
                    [Action {
                        non_terminal,
                        args,
                        sem: _s0 @ ProductionAttribute::AddToCollection,
                        ..
                    }, Action {
                        sem: _s1 @ ProductionAttribute::CollectionStart,
                        ..
                    }] => {
                        let mut arguments = args.clone();
                        Some(ASTType::Struct(
                            NmHlp::to_upper_camel_case(&non_terminal),
                            arguments
                                .drain(..)
                                .map(|arg| (arg.name, arg.arg_type))
                                .collect::<Vec<(String, ASTType)>>(),
                        ))
                    }
                    [Action {
                        sem: _s0 @ ProductionAttribute::CollectionStart,
                        ..
                    }, Action {
                        non_terminal,
                        args,
                        sem: _s1 @ ProductionAttribute::AddToCollection,
                        ..
                    }] => {
                        let mut arguments = args.clone();
                        Some(ASTType::Struct(
                            NmHlp::to_upper_camel_case(&non_terminal),
                            arguments
                                .drain(..)
                                .map(|arg| (arg.name, arg.arg_type))
                                .collect::<Vec<(String, ASTType)>>(),
                        ))
                    }
                    _ => {
                        // Otherwise: we generate an Enum form the out-types of each action
                        let nt_ref = &actions[0].non_terminal;
                        Some(ASTType::Enum(
                            NmHlp::to_upper_camel_case(nt_ref),
                            actions
                                .iter()
                                .map(|a| a.out_type.clone())
                                .collect::<Vec<ASTType>>(),
                        ))
                    }
                }
            }
        }
    }

    fn deduce_type_of_non_terminals(&mut self, cfg: &Cfg) -> Result<()> {
        for nt in cfg.get_non_terminal_set() {
            let actions = self.matching_actions(&nt);
            if let Some(nt_type) = self.deduce_type_of_non_terminal(actions) {
                self.add_non_terminal_type(&nt, nt_type)?;
            }
        }
        Ok(())
    }

    fn deduce_type_of_symbol(symbol: &Symbol) -> Result<ASTType> {
        match symbol {
            Symbol::T(Terminal::Trm(t, _)) => Ok(ASTType::Token(t.to_string())),
            Symbol::N(n, a) => {
                let inner_type_name = NmHlp::to_upper_camel_case(n);
                match a {
                    SymbolAttribute::None => Ok(ASTType::TypeRef(inner_type_name)),
                    SymbolAttribute::RepetitionAnchor => Ok(ASTType::Repeat(inner_type_name)),
                }
            }
            _ => Err(miette!("Unexpected symbol kind: {}", symbol)),
        }
    }

    ///
    /// Returns a vector of action indices matching the given non-terminal n
    ///
    fn matching_actions(&self, n: &str) -> Vec<usize> {
        self.actions
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut acc, (i, a)| {
                if a.non_terminal == n {
                    acc.push(i);
                }
                acc
            })
    }

    /// Create a new item
    /// Initializes the helper data `terminals` and `terminal_names`.
    pub fn new(cfg: &Cfg) -> Self {
        let mut me = Self::default();
        me.terminals = cfg
            .get_ordered_terminals()
            .iter()
            .map(|(t, _)| t.to_string())
            .collect::<Vec<String>>();

        me.terminal_names = me.terminals.iter().fold(Vec::new(), |mut acc, e| {
            let n = generate_terminal_name(e, None, cfg);
            acc.push(n);
            acc
        });
        me
    }

    fn struct_data_of_production(&self, prod: &Pr) -> Result<ASTType> {
        let mut arguments = self.build_argument_list(prod)?;
        if matches!(prod.2, ProductionAttribute::AddToCollection) {
            Ok(ASTType::Repeat(NmHlp::to_upper_camel_case(
                prod.get_n_str(),
            )))
        } else {
            Ok(ASTType::Struct(
                NmHlp::to_upper_camel_case(prod.get_n_str()),
                arguments
                    .drain(..)
                    .map(|arg| (arg.name, arg.arg_type))
                    .collect::<Vec<(String, ASTType)>>(),
            ))
        }
    }
}

impl Display for GrammarTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        for action in &self.actions {
            writeln!(f, "{}", action)?;
        }
        writeln!(f)?;
        for (non_terminal, ast_type) in &self.non_terminal_types {
            writeln!(f, "{}:  {}", non_terminal, ast_type)?;
        }
        Ok(())
    }
}

impl TryFrom<&Cfg> for GrammarTypeInfo {
    type Error = miette::Error;
    fn try_from(cfg: &Cfg) -> Result<Self> {
        let mut me = Self::new(cfg);
        me.deduce_actions(cfg)?;
        me.deduce_type_of_non_terminals(cfg)?;
        Ok(me)
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::GrammarTypeInfo;
    use crate::{left_factor, obtain_grammar_config_from_string, render_par_string, Cfg};
    use std::convert::TryInto;

    static GRAMMAR1: &str = r#"%start S %% S: "a" {"b-rpt"} "c" {"d-rpt"};"#;
    static GRAMMAR2: &str = r#"%start S %% S: "a" ["b-opt"] "c" ["d-opt"];"#;

    lazy_static! {
        /*
        S: "a" {"b-rpt"} "c" {"d-rpt"};
        =>
        /* 0 */ S: "a" SList /* Vec */ "c" SList1 /* Vec */;
        /* 1 */ SList1: "d-rpt" SList1; // Vec<T>::Push
        /* 2 */ SList1: ; // Vec<T>::New
        /* 3 */ SList: "b-rpt" SList; // Vec<T>::Push
        /* 4 */ SList: ; // Vec<T>::New
        */
        static ref G1: Cfg = left_factor(
            &obtain_grammar_config_from_string(GRAMMAR1, false).unwrap().cfg);
        static ref TYPE_INFO1: GrammarTypeInfo = (&*G1).try_into().unwrap();

        /*
        S: "a" ["b-opt"] "c" ["d-opt"];
        =>
        /* 0 */ S: "a" "b-opt" "c" "d-opt";
        /* 1 */ S: "a" "b-opt" "c";
        /* 2 */ S: "a" "c" "d-opt";
        /* 3 */ S: "a" "c";
        */
        static ref G2: Cfg = left_factor(
            &obtain_grammar_config_from_string(GRAMMAR2, false).unwrap().cfg);
        static ref TYPE_INFO2: GrammarTypeInfo = (&*G2).try_into().unwrap();

        static ref RX_NEWLINE: Regex = Regex::new(r"\r?\n").unwrap();
    }

    #[test]
    fn test_presentation_of_grammar_1() {
        let expected = r#"%start S

%%

/* 0 */ S: "a" SList /* Vec */ "c" SList1 /* Vec */;
/* 1 */ SList1: "d-rpt" SList1; // Vec<T>::Push
/* 2 */ SList1: ; // Vec<T>::New
/* 3 */ SList: "b-rpt" SList; // Vec<T>::Push
/* 4 */ SList: ; // Vec<T>::New
"#;

        let par_str = render_par_string(
            &obtain_grammar_config_from_string(GRAMMAR1, false).unwrap(),
            true,
        )
        .unwrap();

        assert_eq!(
            RX_NEWLINE.replace_all(expected, "\n"),
            RX_NEWLINE.replace_all(&par_str, "\n")
        );
    }

    #[test]
    fn test_presentation_of_grammar_2() {
        let expected = r#"%start S

%%

/* 0 */ S: "a" "b-opt" "c" "d-opt";
/* 1 */ S: "a" "b-opt" "c";
/* 2 */ S: "a" "c" "d-opt";
/* 3 */ S: "a" "c";
"#;

        let par_str = render_par_string(
            &obtain_grammar_config_from_string(GRAMMAR2, false).unwrap(),
            true,
        )
        .unwrap();

        assert_eq!(
            RX_NEWLINE.replace_all(expected, "\n"),
            RX_NEWLINE.replace_all(&par_str, "\n")
        );
    }

    #[test]
    fn test_presentation_of_type_info_1() {
        let expected = r#"/* 0 */ ((OwnedToken /* a */, Vec<SList>) -> struct S { s_list_1: OwnedToken /* a */, s_list1_3: Vec<SList> })  { - }
/* 1 */ ((OwnedToken /* d-rpt */) -> Vec<SList1>)  { Vec<T>::Push }
/* 2 */ (() -> Vec<SList1>)  { Vec<T>::New }
/* 3 */ ((OwnedToken /* b-rpt */) -> Vec<SList>)  { Vec<T>::Push }
/* 4 */ (() -> Vec<SList>)  { Vec<T>::New }

S:  struct S { s_list_1: OwnedToken /* a */, s_list1_3: Vec<SList> }
SList:  struct SList { s_list_1: OwnedToken /* b-rpt */ }
SList1:  struct SList1 { s_list1_1: OwnedToken /* d-rpt */ }
"#;

        let presentation = format!("{}", *TYPE_INFO1);

        assert_eq!(
            RX_NEWLINE.replace_all(expected, "\n"),
            RX_NEWLINE.replace_all(&presentation, "\n")
        );
    }

    #[test]
    fn test_presentation_of_type_info_2() {
        let expected = r#"/* 0 */ ((OwnedToken /* a */) -> struct S { s_suffix2_1: OwnedToken /* a */ })  { - }
/* 1 */ ((OwnedToken /* c */) -> struct SSuffix2 { s_suffix1_1: OwnedToken /* c */ })  { - }
/* 2 */ ((OwnedToken /* b-opt */) -> struct SSuffix2 { s_suffix_2: OwnedToken /* b-opt */ })  { - }
/* 3 */ (() -> struct SSuffix1 {  })  { - }
/* 4 */ (() -> ())  { - }
/* 5 */ (() -> struct SSuffix {  })  { - }
/* 6 */ (() -> ())  { - }

S:  struct S { s_suffix2_1: OwnedToken /* a */ }
SSuffix:  enum SSuffix { SSuffix0(struct SSuffix {  }), SSuffix1(()) }
SSuffix1:  enum SSuffix1 { SSuffix10(struct SSuffix1 {  }), SSuffix11(()) }
SSuffix2:  enum SSuffix2 { SSuffix20(struct SSuffix2 { s_suffix1_1: OwnedToken /* c */ }), SSuffix21(struct SSuffix2 { s_suffix_2: OwnedToken /* b-opt */ }) }
"#;
        let presentation = format!("{}", *TYPE_INFO2);

        assert_eq!(
            RX_NEWLINE.replace_all(expected, "\n"),
            RX_NEWLINE.replace_all(&presentation, "\n")
        );
    }
}
