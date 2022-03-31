use crate::analysis::lookahead_dfa::ProductionIndex;
use crate::generators::{generate_terminal_name, NamingHelper as NmHlp};
use crate::grammar::{ProductionAttribute, SymbolAttribute};
use crate::{Cfg, GrammarConfig, Pr, Symbol, Terminal};
use log::trace;
use miette::{miette, IntoDiagnostic, Result};
use std::collections::{BTreeMap, HashSet};
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
    /// A type name (without Box semantic)
    TypeName(String),
    /// A struct, i.e. a named collection of (name, type) tuples
    Struct(String, Vec<(String, ASTType)>),
    /// Will be generated as enum with given name
    Enum(String, Vec<(String, ASTType)>),
    /// Will be generated as Vec<T> where T is the type, similar to TypeRef
    Repeat(String),
}

impl ASTType {
    pub(crate) fn type_name(&self) -> String {
        match self {
            Self::None => "*TypeError*".to_owned(),
            Self::Unit => "()".to_owned(),
            Self::Token(t) => format!("Token<'t> /* {} */", t),
            Self::TypeRef(r) => format!("Box<{}<'t>>", r),
            Self::TypeName(n) => n.clone(),
            Self::Struct(n, _) => n.to_string(),
            Self::Enum(n, _) => n.to_string(),
            Self::Repeat(r) => format!("Vec<{}<'t>>", r),
        }
    }

    pub(crate) fn inner_type_name(&self) -> String {
        match self {
            Self::None => "*TypeError*".to_owned(),
            Self::Unit => "()".to_owned(),
            Self::Token(t) => format!("Token<'t> /* {} */", t),
            Self::TypeRef(r) => r.clone(),
            Self::TypeName(n) => n.clone(),
            Self::Struct(n, _) => n.to_string(),
            Self::Enum(n, _) => n.to_string(),
            Self::Repeat(r) => r.clone(),
        }
    }

    /// Change the type's name
    pub(crate) fn with_name(self, name: String) -> Self {
        let name = NmHlp::to_upper_camel_case(&name);
        match self {
            Self::None => self,
            Self::Unit => self,
            Self::Token(_) => self,
            Self::TypeRef(_) => Self::TypeRef(name),
            Self::TypeName(_) => Self::TypeName(name),
            Self::Struct(_, m) => Self::Struct(name, m),
            Self::Enum(_, m) => Self::Struct(name, m),
            Self::Repeat(_) => self,
        }
    }

    pub(crate) fn has_lifetime(&self) -> bool {
        match self {
            Self::None | Self::Unit => false,
            Self::Token(_) | Self::TypeRef(_) | Self::TypeName(_) | Self::Repeat(_) => true,
            Self::Struct(_, m) => m.iter().any(|e| e.1.has_lifetime()),
            Self::Enum(_, m) => m.iter().any(|e| e.1.has_lifetime()),
        }
    }

    pub(crate) fn lifetime(&self) -> String {
        match self {
            Self::None | Self::Unit => "".to_owned(),
            Self::Token(_) | Self::TypeRef(_) | Self::TypeName(_) | Self::Repeat(_) => {
                "<'t>".to_owned()
            }
            Self::Struct(_, _) | Self::Enum(_, _) => {
                if self.has_lifetime() {
                    "<'t>".to_owned()
                } else {
                    "".to_owned()
                }
            }
        }
    }
}

impl Display for ASTType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::None => write!(f, "-"),
            Self::Unit => write!(f, "()"),
            Self::Token(t) => write!(f, "Token<'t> /* {} */", t),
            Self::TypeRef(r) => write!(f, "Box<{}<'t>>", r),
            Self::TypeName(n) => write!(f, "{}", n),
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
                    .map(|(c, t)| format!("{}({})", c, t))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Repeat(r) => write!(f, "Vec<{}<'t>>", r),
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
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = NmHlp::to_lower_snake_case(&name);
        self
    }

    /// Get the argument's name
    pub fn name(&self) -> String {
        let name = if !self.used && self.name.starts_with("r#") {
            self.name[2..].to_string()
        } else {
            self.name.clone()
        };
        format!("{}{}", NmHlp::item_unused_indicator(self.used), name)
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

    /// Formatted production in PAR syntax.
    pub(crate) prod_string: String,

    /// The argument list as they are provided by the parser
    pub(crate) args: Vec<Argument>,

    /// The output type, i.e. the return type of the action which corresponds to the constructed
    /// new value pushed on the AST stack.
    /// If there exists an associated semantic action of the user's `input` grammar this type is
    /// used to call it with.
    pub(crate) out_type: ASTType,

    /// Number of alternatives, the number of productions that exist in the grammar which have the
    /// same non-terminal
    pub(crate) alts: usize,

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

impl Action {
    fn adjust_arguments_used(&mut self, used: bool) {
        self.args.iter_mut().for_each(|a| a.used &= used);
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

    /// The type completely comprising the whole structural information that could be generated by
    /// the given expanded grammar.
    /// It is a type of enum kind.
    /// We use this as ASTType for the generated source.
    pub(crate) ast_enum_type: ASTType,

    /// Indicates if the auto generation mode is active
    pub(crate) auto_generate: bool,

    /// Helper
    terminals: Vec<String>,
    terminal_names: Vec<String>,

    // Contains non-terminals that should be represented as vectors in the AST Enum type
    vector_typed_non_terminals: HashSet<String>,
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

    fn build(&mut self, grammar_config: &GrammarConfig) -> Result<()> {
        self.deduce_actions(grammar_config)?;
        self.deduce_type_of_non_terminals(&grammar_config.cfg)?;
        self.generate_ast_enum_type()
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

        if prod.2 == ProductionAttribute::AddToCollection {
            let ref_mut_last_type = &mut types.last_mut().unwrap().0;
            *ref_mut_last_type = ASTType::Repeat(ref_mut_last_type.inner_type_name());
        }

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

    fn deduce_type_of_production(&self, prod: &Pr, prod_num: ProductionIndex) -> Result<ASTType> {
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
            _ => Ok(self.struct_data_of_production(prod, prod_num)?),
        }
    }

    /// Creates the list of actions from the Cfg.
    fn deduce_actions(&mut self, grammar_config: &GrammarConfig) -> Result<()> {
        let scanner_state_resolver = grammar_config.get_scanner_state_resolver();
        self.actions = Vec::with_capacity(grammar_config.cfg.pr.len());
        for (i, pr) in grammar_config.cfg.pr.iter().enumerate() {
            self.actions.push(
                ActionBuilder::default()
                    .non_terminal(pr.get_n())
                    .prod_num(i)
                    .fn_name(NmHlp::to_lower_snake_case(&format!(
                        "{}_{}",
                        pr.get_n_str(),
                        i
                    )))
                    .prod_string(pr.format(&scanner_state_resolver)?)
                    .args(self.build_argument_list(pr)?)
                    .out_type(self.deduce_type_of_production(pr, i)?)
                    .sem(pr.2.clone())
                    .alts(
                        grammar_config
                            .cfg
                            .matching_productions(pr.get_n_str())
                            .len(),
                    )
                    .build()
                    .into_diagnostic()?,
            );
        }
        Ok(())
    }

    fn deduce_type_of_non_terminal(&mut self, actions: Vec<usize>, cfg: &Cfg) -> Option<ASTType> {
        let mut vector_typed_non_terminal_opt = None;
        let result_type = match actions.len() {
            // Productions can be optimized away, when they have duplicates!
            0 => None,
            // Only one production for this non-terminal: we take the out-type of the single action
            // but change the name to not contain the production number
            1 => Some(
                self.actions[actions[0]]
                    .out_type
                    .clone()
                    .with_name(self.actions[actions[0]].non_terminal.clone()),
            ),
            // Some(ASTType::TypeName(NmHlp::to_upper_camel_case(
            //     &self.actions[actions[0]].non_terminal,
            // ))),
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
                        arguments.pop(); // Remove the recursive part. Vec is wrapped outside.
                        vector_typed_non_terminal_opt = Some(non_terminal.clone());
                        Some(ASTType::Struct(
                            NmHlp::to_upper_camel_case(non_terminal),
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
                        arguments.pop(); // Remove the recursive part. Vec is wrapped outside.
                        vector_typed_non_terminal_opt = Some(non_terminal.clone());
                        Some(ASTType::Struct(
                            NmHlp::to_upper_camel_case(non_terminal),
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
                                .map(|a| {
                                    (
                                        NmHlp::to_upper_camel_case(&format!(
                                            "{}_{}",
                                            a.non_terminal, a.prod_num
                                        )),
                                        ASTType::TypeName(format!(
                                            "{}_{}{}",
                                            NmHlp::to_upper_camel_case(nt_ref),
                                            a.prod_num,
                                            if cfg[a.prod_num].effective_len() > 0 {
                                                "<'t>"
                                            } else {
                                                ""
                                            },
                                        )),
                                    )
                                })
                                .collect::<Vec<(String, ASTType)>>(),
                        ))
                    }
                }
            }
        };

        if let Some(vector_typed_non_terminal) = vector_typed_non_terminal_opt {
            self.vector_typed_non_terminals
                .insert(vector_typed_non_terminal);
        }

        result_type
    }

    fn deduce_type_of_non_terminals(&mut self, cfg: &Cfg) -> Result<()> {
        for nt in cfg.get_non_terminal_set() {
            let actions = self.matching_actions(&nt);
            if let Some(nt_type) = self.deduce_type_of_non_terminal(actions, cfg) {
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

    fn generate_ast_enum_type(&mut self) -> Result<()> {
        self.ast_enum_type = ASTType::Enum(
            "ASTType".to_owned(),
            self.non_terminal_types
                .iter()
                .map(|(n, t)| {
                    (
                        NmHlp::to_upper_camel_case(n),
                        if self.vector_typed_non_terminals.contains(n) {
                            ASTType::Repeat(t.type_name())
                        } else {
                            ASTType::TypeName(t.type_name() + &t.lifetime())
                        },
                    )
                })
                .collect::<Vec<(String, ASTType)>>(),
        );
        Ok(())
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

    /// Set the auto-generate mode
    /// Internally it adjust the used flags on the arguments of the actions.
    /// The arguments keep their used state only if auto generation is active.
    pub(crate) fn set_auto_generate(&mut self, auto_generate: bool) {
        self.auto_generate = auto_generate;
        self.adjust_arguments_used(auto_generate)
    }

    fn adjust_arguments_used(&mut self, used: bool) {
        self.actions
            .iter_mut()
            .for_each(|a| a.adjust_arguments_used(used))
    }

    fn struct_data_of_production(&self, prod: &Pr, prod_num: ProductionIndex) -> Result<ASTType> {
        let mut arguments = self.build_argument_list(prod)?;
        if matches!(prod.2, ProductionAttribute::AddToCollection) {
            Ok(ASTType::Repeat(NmHlp::to_upper_camel_case(
                prod.get_n_str(),
            )))
        } else {
            Ok(ASTType::Struct(
                NmHlp::to_upper_camel_case(&format!("{}{}", prod.get_n_str(), prod_num)),
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
        writeln!(f)?;
        writeln!(f, "{}", self.ast_enum_type)?;
        Ok(())
    }
}

impl TryFrom<&GrammarConfig> for GrammarTypeInfo {
    type Error = miette::Error;
    fn try_from(grammar_config: &GrammarConfig) -> Result<Self> {
        let mut me = Self::new(&grammar_config.cfg);
        me.build(grammar_config)?;
        Ok(me)
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::GrammarTypeInfo;
    use crate::{left_factor, obtain_grammar_config_from_string, render_par_string, GrammarConfig};
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
        static ref GC1: GrammarConfig = {
            let mut gc1 = obtain_grammar_config_from_string(GRAMMAR1, false).unwrap();
            let cfg = left_factor(&gc1.cfg);
            gc1.update_cfg(cfg);
            gc1
        };
        static ref TYPE_INFO1: GrammarTypeInfo = (&*GC1).try_into().unwrap();

        /*
        S: "a" ["b-opt"] "c" ["d-opt"];
        =>
        /* 0 */ S: "a" "b-opt" "c" "d-opt";
        /* 1 */ S: "a" "b-opt" "c";
        /* 2 */ S: "a" "c" "d-opt";
        /* 3 */ S: "a" "c";
        */
        static ref GC2: GrammarConfig ={
            let mut gc2 = obtain_grammar_config_from_string(GRAMMAR2, false).unwrap();
            let cfg = left_factor(&gc2.cfg);
            gc2.update_cfg(cfg);
            gc2
        };
        static ref TYPE_INFO2: GrammarTypeInfo = (&*GC2).try_into().unwrap();

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
        let expected = r#"/* 0 */ ((Token<'t> /* a */, Vec<SList<'t>>, Token<'t> /* c */, Vec<SList1<'t>>) -> struct S0 { a_0: Token<'t> /* a */, s_list_1: Vec<SList<'t>>, c_2: Token<'t> /* c */, s_list1_3: Vec<SList1<'t>> })  { - }
/* 1 */ ((Token<'t> /* d-rpt */, Vec<SList1<'t>>) -> Vec<SList1<'t>>)  { Vec<T>::Push }
/* 2 */ (() -> Vec<SList1<'t>>)  { Vec<T>::New }
/* 3 */ ((Token<'t> /* b-rpt */, Vec<SList<'t>>) -> Vec<SList<'t>>)  { Vec<T>::Push }
/* 4 */ (() -> Vec<SList<'t>>)  { Vec<T>::New }

S:  struct S { a_0: Token<'t> /* a */, s_list_1: Vec<SList<'t>>, c_2: Token<'t> /* c */, s_list1_3: Vec<SList1<'t>> }
SList:  struct SList { b_minus_rpt_0: Token<'t> /* b-rpt */ }
SList1:  struct SList1 { d_minus_rpt_0: Token<'t> /* d-rpt */ }

enum ASTType { S(S<'t>), SList(Vec<SList<'t>>), SList1(Vec<SList1<'t>>) }
"#;

        let presentation = format!("{}", *TYPE_INFO1);

        assert_eq!(
            RX_NEWLINE.replace_all(expected, "\n"),
            RX_NEWLINE.replace_all(&presentation, "\n")
        );
    }

    #[test]
    fn test_presentation_of_type_info_2() {
        let expected = r#"/* 0 */ ((Token<'t> /* a */, Box<SSuffix2<'t>>) -> struct S0 { a_0: Token<'t> /* a */, s_suffix2_1: Box<SSuffix2<'t>> })  { - }
/* 1 */ ((Token<'t> /* c */, Box<SSuffix1<'t>>) -> struct SSuffix21 { c_0: Token<'t> /* c */, s_suffix1_1: Box<SSuffix1<'t>> })  { - }
/* 2 */ ((Token<'t> /* b-opt */, Token<'t> /* c */, Box<SSuffix<'t>>) -> struct SSuffix22 { b_minus_opt_0: Token<'t> /* b-opt */, c_1: Token<'t> /* c */, s_suffix_2: Box<SSuffix<'t>> })  { - }
/* 3 */ ((Token<'t> /* d-opt */) -> struct SSuffix13 { d_minus_opt_0: Token<'t> /* d-opt */ })  { - }
/* 4 */ (() -> ())  { - }
/* 5 */ ((Token<'t> /* d-opt */) -> struct SSuffix5 { d_minus_opt_0: Token<'t> /* d-opt */ })  { - }
/* 6 */ (() -> ())  { - }

S:  struct S { a_0: Token<'t> /* a */, s_suffix2_1: Box<SSuffix2<'t>> }
SSuffix:  enum SSuffix { SSuffix5(SSuffix_5<'t>), SSuffix6(SSuffix_6) }
SSuffix1:  enum SSuffix1 { SSuffix1_3(SSuffix1_3<'t>), SSuffix1_4(SSuffix1_4) }
SSuffix2:  enum SSuffix2 { SSuffix2_1(SSuffix2_1<'t>), SSuffix2_2(SSuffix2_2<'t>) }

enum ASTType { S(S<'t>), SSuffix(SSuffix<'t>), SSuffix1(SSuffix1<'t>), SSuffix2(SSuffix2<'t>) }
"#;
        let presentation = format!("{}", *TYPE_INFO2);

        assert_eq!(
            RX_NEWLINE.replace_all(expected, "\n"),
            RX_NEWLINE.replace_all(&presentation, "\n")
        );
    }
}
