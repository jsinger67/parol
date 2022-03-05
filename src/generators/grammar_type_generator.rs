use crate::analysis::lookahead_dfa::ProductionIndex;
use crate::generators::{generate_terminal_name, NamingHelper as NmHlp};
use crate::grammar::{ProductionAttribute, SymbolAttribute};
use crate::Cfg;
use miette::Result;
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
    /// Will be generated to a Token structure
    Token(String),
    /// A type name
    TypeRef(String),
    /// A struct, i.e. a collection of types
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
        let me = Self::new(cfg);
        // me.deduce_actions(cfg)?;
        // me.deduce_type_of_non_terminals(cfg)?;
        Ok(me)
    }
}
