use parol_runtime::parser::parse_tree_type::{
    ChildAttribute, ChildKind, ExpectedChildren, ExpectedChildrenKinds, Node, NodeKind,
    NonTerminalEnum, TerminalEnum,
};
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NonTerminalKind {
    AddOp,
    AssignItem,
    AssignOp,
    Assignment,
    AssignmentList,
    BitwiseAnd,
    BitwiseAndList,
    BitwiseAndOp,
    BitwiseOr,
    BitwiseOrList,
    BitwiseOrOp,
    BitwiseShift,
    BitwiseShiftList,
    BitwiseShiftOp,
    Calc,
    CalcList,
    Equality,
    EqualityList,
    EqualityOp,
    Factor,
    Id,
    IdRef,
    Instruction,
    LogicalAnd,
    LogicalAndList,
    LogicalAndOp,
    LogicalOr,
    LogicalOrList,
    LogicalOrOp,
    Minus,
    Mult,
    MultList,
    MultOp,
    Negate,
    Number,
    Plus,
    PowOp,
    Power,
    PowerList,
    Relational,
    RelationalList,
    RelationalOp,
    Summ,
    SummList,
    Root,
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TerminalKind {
    NewLine,
    Whitespace,
    LineComment,
    BlockComment,
    Semicolon,
    EqualityOp,
    AssignOp,
    OrOr,
    AmpAmp,
    Or,
    Amp,
    BitwiseShiftOp,
    RelationalOp,
    Plus,
    Minus,
    StarStar,
    MultOp,
    LParen,
    RParen,
    Number,
    Id,
}
impl TerminalEnum for TerminalKind {
    fn from_terminal_index(index: u16) -> Self {
        match index {
            1 => Self::NewLine,
            2 => Self::Whitespace,
            3 => Self::LineComment,
            4 => Self::BlockComment,
            5 => Self::Semicolon,
            6 => Self::EqualityOp,
            7 => Self::AssignOp,
            8 => Self::OrOr,
            9 => Self::AmpAmp,
            10 => Self::Or,
            11 => Self::Amp,
            12 => Self::BitwiseShiftOp,
            13 => Self::RelationalOp,
            14 => Self::Plus,
            15 => Self::Minus,
            16 => Self::StarStar,
            17 => Self::MultOp,
            18 => Self::LParen,
            19 => Self::RParen,
            20 => Self::Number,
            21 => Self::Id,
            _ => panic!("Invalid terminal index: {}", index),
        }
    }
    fn is_builtin_new_line(&self) -> bool {
        matches!(self, TerminalKind::NewLine)
    }
    fn is_builtin_whitespace(&self) -> bool {
        matches!(self, TerminalKind::Whitespace)
    }
}

impl NonTerminalEnum for NonTerminalKind {
    fn from_non_terminal_name(name: &str) -> Self {
        match name {
            "AddOp" => Self::AddOp,
            "AssignItem" => Self::AssignItem,
            "AssignOp" => Self::AssignOp,
            "Assignment" => Self::Assignment,
            "AssignmentList" => Self::AssignmentList,
            "BitwiseAnd" => Self::BitwiseAnd,
            "BitwiseAndList" => Self::BitwiseAndList,
            "BitwiseAndOp" => Self::BitwiseAndOp,
            "BitwiseOr" => Self::BitwiseOr,
            "BitwiseOrList" => Self::BitwiseOrList,
            "BitwiseOrOp" => Self::BitwiseOrOp,
            "BitwiseShift" => Self::BitwiseShift,
            "BitwiseShiftList" => Self::BitwiseShiftList,
            "BitwiseShiftOp" => Self::BitwiseShiftOp,
            "Calc" => Self::Calc,
            "CalcList" => Self::CalcList,
            "Equality" => Self::Equality,
            "EqualityList" => Self::EqualityList,
            "EqualityOp" => Self::EqualityOp,
            "Factor" => Self::Factor,
            "Id" => Self::Id,
            "IdRef" => Self::IdRef,
            "Instruction" => Self::Instruction,
            "LogicalAnd" => Self::LogicalAnd,
            "LogicalAndList" => Self::LogicalAndList,
            "LogicalAndOp" => Self::LogicalAndOp,
            "LogicalOr" => Self::LogicalOr,
            "LogicalOrList" => Self::LogicalOrList,
            "LogicalOrOp" => Self::LogicalOrOp,
            "Minus" => Self::Minus,
            "Mult" => Self::Mult,
            "MultList" => Self::MultList,
            "MultOp" => Self::MultOp,
            "Negate" => Self::Negate,
            "Number" => Self::Number,
            "Plus" => Self::Plus,
            "PowOp" => Self::PowOp,
            "Power" => Self::Power,
            "PowerList" => Self::PowerList,
            "Relational" => Self::Relational,
            "RelationalList" => Self::RelationalList,
            "RelationalOp" => Self::RelationalOp,
            "Summ" => Self::Summ,
            "SummList" => Self::SummList,
            "" => Self::Root,
            _ => panic!("Invalid non-terminal name: {}", name),
        }
    }
}
impl std::fmt::Display for TerminalKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewLine => write!(f, stringify!(NewLine)),
            Self::Whitespace => write!(f, stringify!(Whitespace)),
            Self::LineComment => write!(f, stringify!(LineComment)),
            Self::BlockComment => write!(f, stringify!(BlockComment)),
            Self::Semicolon => write!(f, stringify!(Semicolon)),
            Self::EqualityOp => write!(f, stringify!(EqualityOp)),
            Self::AssignOp => write!(f, stringify!(AssignOp)),
            Self::OrOr => write!(f, stringify!(OrOr)),
            Self::AmpAmp => write!(f, stringify!(AmpAmp)),
            Self::Or => write!(f, stringify!(Or)),
            Self::Amp => write!(f, stringify!(Amp)),
            Self::BitwiseShiftOp => write!(f, stringify!(BitwiseShiftOp)),
            Self::RelationalOp => write!(f, stringify!(RelationalOp)),
            Self::Plus => write!(f, stringify!(Plus)),
            Self::Minus => write!(f, stringify!(Minus)),
            Self::StarStar => write!(f, stringify!(StarStar)),
            Self::MultOp => write!(f, stringify!(MultOp)),
            Self::LParen => write!(f, stringify!(LParen)),
            Self::RParen => write!(f, stringify!(RParen)),
            Self::Number => write!(f, stringify!(Number)),
            Self::Id => write!(f, stringify!(Id)),
        }
    }
}

impl std::fmt::Display for NonTerminalKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddOp => write!(f, stringify!(AddOp)),
            Self::AssignItem => write!(f, stringify!(AssignItem)),
            Self::AssignOp => write!(f, stringify!(AssignOp)),
            Self::Assignment => write!(f, stringify!(Assignment)),
            Self::AssignmentList => write!(f, stringify!(AssignmentList)),
            Self::BitwiseAnd => write!(f, stringify!(BitwiseAnd)),
            Self::BitwiseAndList => write!(f, stringify!(BitwiseAndList)),
            Self::BitwiseAndOp => write!(f, stringify!(BitwiseAndOp)),
            Self::BitwiseOr => write!(f, stringify!(BitwiseOr)),
            Self::BitwiseOrList => write!(f, stringify!(BitwiseOrList)),
            Self::BitwiseOrOp => write!(f, stringify!(BitwiseOrOp)),
            Self::BitwiseShift => write!(f, stringify!(BitwiseShift)),
            Self::BitwiseShiftList => write!(f, stringify!(BitwiseShiftList)),
            Self::BitwiseShiftOp => write!(f, stringify!(BitwiseShiftOp)),
            Self::Calc => write!(f, stringify!(Calc)),
            Self::CalcList => write!(f, stringify!(CalcList)),
            Self::Equality => write!(f, stringify!(Equality)),
            Self::EqualityList => write!(f, stringify!(EqualityList)),
            Self::EqualityOp => write!(f, stringify!(EqualityOp)),
            Self::Factor => write!(f, stringify!(Factor)),
            Self::Id => write!(f, stringify!(Id)),
            Self::IdRef => write!(f, stringify!(IdRef)),
            Self::Instruction => write!(f, stringify!(Instruction)),
            Self::LogicalAnd => write!(f, stringify!(LogicalAnd)),
            Self::LogicalAndList => write!(f, stringify!(LogicalAndList)),
            Self::LogicalAndOp => write!(f, stringify!(LogicalAndOp)),
            Self::LogicalOr => write!(f, stringify!(LogicalOr)),
            Self::LogicalOrList => write!(f, stringify!(LogicalOrList)),
            Self::LogicalOrOp => write!(f, stringify!(LogicalOrOp)),
            Self::Minus => write!(f, stringify!(Minus)),
            Self::Mult => write!(f, stringify!(Mult)),
            Self::MultList => write!(f, stringify!(MultList)),
            Self::MultOp => write!(f, stringify!(MultOp)),
            Self::Negate => write!(f, stringify!(Negate)),
            Self::Number => write!(f, stringify!(Number)),
            Self::Plus => write!(f, stringify!(Plus)),
            Self::PowOp => write!(f, stringify!(PowOp)),
            Self::Power => write!(f, stringify!(Power)),
            Self::PowerList => write!(f, stringify!(PowerList)),
            Self::Relational => write!(f, stringify!(Relational)),
            Self::RelationalList => write!(f, stringify!(RelationalList)),
            Self::RelationalOp => write!(f, stringify!(RelationalOp)),
            Self::Summ => write!(f, stringify!(Summ)),
            Self::SummList => write!(f, stringify!(SummList)),
            Self::Root => write!(f, stringify!()),
        }
    }
}
impl ExpectedChildren<TerminalKind, NonTerminalKind> for NonTerminalKind {
    fn expected_children(&self) -> ExpectedChildrenKinds<TerminalKind, NonTerminalKind> {
        match self {
            Self::AddOp => ExpectedChildrenKinds::OneOf(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Plus),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Minus),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::AssignItem => ExpectedChildrenKinds::Sequence(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Id),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::AssignOp),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::AssignOp => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::AssignOp),
                attribute: ChildAttribute::Normal,
            }]),
            Self::Assignment => ExpectedChildrenKinds::Sequence(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::AssignItem),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::AssignmentList),
                    attribute: ChildAttribute::Vec,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::LogicalOr),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::AssignmentList => ExpectedChildrenKinds::Recursion(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::AssignmentList),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::AssignItem),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::BitwiseAnd => ExpectedChildrenKinds::Sequence(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Equality),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseAndList),
                    attribute: ChildAttribute::Vec,
                },
            ]),
            Self::BitwiseAndList => ExpectedChildrenKinds::Recursion(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseAndList),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseAndOp),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Equality),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::BitwiseAndOp => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::Amp),
                attribute: ChildAttribute::Normal,
            }]),
            Self::BitwiseOr => ExpectedChildrenKinds::Sequence(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseAnd),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseOrList),
                    attribute: ChildAttribute::Vec,
                },
            ]),
            Self::BitwiseOrList => ExpectedChildrenKinds::Recursion(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseOrList),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseOrOp),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseAnd),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::BitwiseOrOp => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::Or),
                attribute: ChildAttribute::Normal,
            }]),
            Self::BitwiseShift => ExpectedChildrenKinds::Sequence(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Summ),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseShiftList),
                    attribute: ChildAttribute::Vec,
                },
            ]),
            Self::BitwiseShiftList => ExpectedChildrenKinds::Recursion(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseShiftList),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseShiftOp),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Summ),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::BitwiseShiftOp => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::BitwiseShiftOp),
                attribute: ChildAttribute::Normal,
            }]),
            Self::Calc => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::NonTerminal(NonTerminalKind::CalcList),
                attribute: ChildAttribute::Vec,
            }]),
            Self::CalcList => ExpectedChildrenKinds::Recursion(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::CalcList),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Instruction),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::Equality => ExpectedChildrenKinds::Sequence(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Relational),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::EqualityList),
                    attribute: ChildAttribute::Vec,
                },
            ]),
            Self::EqualityList => ExpectedChildrenKinds::Recursion(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::EqualityList),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::EqualityOp),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Relational),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::EqualityOp => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::EqualityOp),
                attribute: ChildAttribute::Normal,
            }]),
            Self::Factor => ExpectedChildrenKinds::OneOf(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Number),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Negate),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::IdRef),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::Id => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::Id),
                attribute: ChildAttribute::Normal,
            }]),
            Self::IdRef => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::NonTerminal(NonTerminalKind::Id),
                attribute: ChildAttribute::Normal,
            }]),
            Self::Instruction => ExpectedChildrenKinds::OneOf(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Assignment),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::LogicalOr),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::LogicalAnd => ExpectedChildrenKinds::Sequence(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseOr),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::LogicalAndList),
                    attribute: ChildAttribute::Vec,
                },
            ]),
            Self::LogicalAndList => ExpectedChildrenKinds::Recursion(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::LogicalAndList),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::LogicalAndOp),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseOr),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::LogicalAndOp => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::AmpAmp),
                attribute: ChildAttribute::Normal,
            }]),
            Self::LogicalOr => ExpectedChildrenKinds::Sequence(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::LogicalAnd),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::LogicalOrList),
                    attribute: ChildAttribute::Vec,
                },
            ]),
            Self::LogicalOrList => ExpectedChildrenKinds::Recursion(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::LogicalOrList),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::LogicalOrOp),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::LogicalAnd),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::LogicalOrOp => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::OrOr),
                attribute: ChildAttribute::Normal,
            }]),
            Self::Minus => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::Minus),
                attribute: ChildAttribute::Normal,
            }]),
            Self::Mult => ExpectedChildrenKinds::Sequence(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Power),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::MultList),
                    attribute: ChildAttribute::Vec,
                },
            ]),
            Self::MultList => ExpectedChildrenKinds::Recursion(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::MultList),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::MultOp),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Power),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::MultOp => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::MultOp),
                attribute: ChildAttribute::Normal,
            }]),
            Self::Negate => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::NonTerminal(NonTerminalKind::Minus),
                attribute: ChildAttribute::Normal,
            }]),
            Self::Number => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::Number),
                attribute: ChildAttribute::Normal,
            }]),
            Self::Plus => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::Plus),
                attribute: ChildAttribute::Normal,
            }]),
            Self::PowOp => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::StarStar),
                attribute: ChildAttribute::Normal,
            }]),
            Self::Power => ExpectedChildrenKinds::Sequence(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Factor),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::PowerList),
                    attribute: ChildAttribute::Vec,
                },
            ]),
            Self::PowerList => ExpectedChildrenKinds::Recursion(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::PowerList),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::PowOp),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Factor),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::Relational => ExpectedChildrenKinds::Sequence(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseShift),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::RelationalList),
                    attribute: ChildAttribute::Vec,
                },
            ]),
            Self::RelationalList => ExpectedChildrenKinds::Recursion(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::RelationalList),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::RelationalOp),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::BitwiseShift),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::RelationalOp => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::Terminal(TerminalKind::RelationalOp),
                attribute: ChildAttribute::Normal,
            }]),
            Self::Summ => ExpectedChildrenKinds::Sequence(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Mult),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::SummList),
                    attribute: ChildAttribute::Vec,
                },
            ]),
            Self::SummList => ExpectedChildrenKinds::Recursion(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::SummList),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::AddOp),
                    attribute: ChildAttribute::Normal,
                },
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::Mult),
                    attribute: ChildAttribute::Normal,
                },
            ]),
            Self::Root => ExpectedChildrenKinds::Sequence(&[ChildKind {
                kind: NodeKind::NonTerminal(NonTerminalKind::Calc),
                attribute: ChildAttribute::Normal,
            }]),
        }
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddOp<T> {
    Plus(Plus<T>),
    Minus(Minus<T>),
    Invalid(T),
}
#[allow(dead_code)]
impl<'a, N> AddOp<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        match node.kind() {
            NodeKind::NonTerminal(NonTerminalKind::Plus) => Self::Plus(Plus::new(node)),
            NodeKind::NonTerminal(NonTerminalKind::Minus) => Self::Minus(Minus::new(node)),
            _ => AddOp::Invalid(node),
        }
    }
    pub fn node(&self) -> &N {
        match self {
            Self::Plus(node) => node.node(),
            Self::Minus(node) => node.node(),
            Self::Invalid(node) => node,
        }
    }
    pub fn node_mut(&mut self) -> &mut N {
        match self {
            Self::Plus(node) => node.node_mut(),
            Self::Minus(node) => node.node_mut(),
            Self::Invalid(node) => node,
        }
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AssignItem<T>(T);
#[allow(dead_code)]
impl<'a, N> AssignItem<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        AssignItem(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_id(&self, cursor: usize) -> Result<Option<(usize, AssignItem<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Id))
            .map(|option| option.map(|(i, node)| (i, AssignItem::new(node))))
    }
    pub fn find_assign_op(&self, cursor: usize) -> Result<Option<(usize, AssignItem<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::AssignOp))
            .map(|option| option.map(|(i, node)| (i, AssignItem::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AssignOp<T>(T);
#[allow(dead_code)]
impl<'a, N> AssignOp<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        AssignOp(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_assign_op(&self, cursor: usize) -> Result<Option<(usize, AssignOp<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::AssignOp))
            .map(|option| option.map(|(i, node)| (i, AssignOp::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Assignment<T>(T);
#[allow(dead_code)]
impl<'a, N> Assignment<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        Assignment(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_assign_item(&self, cursor: usize) -> Result<Option<(usize, Assignment<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::AssignItem))
            .map(|option| option.map(|(i, node)| (i, Assignment::new(node))))
    }
    pub fn find_assignment_list(&self, cursor: usize) -> Result<Option<(usize, Assignment<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::AssignmentList),
            )
            .map(|option| option.map(|(i, node)| (i, Assignment::new(node))))
    }
    pub fn find_logical_or(&self, cursor: usize) -> Result<Option<(usize, Assignment<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::LogicalOr))
            .map(|option| option.map(|(i, node)| (i, Assignment::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AssignmentList<T>(T);
#[allow(dead_code)]
impl<'a, N> AssignmentList<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        AssignmentList(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_assignment_list(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, AssignmentList<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::AssignmentList),
            )
            .map(|option| option.map(|(i, node)| (i, AssignmentList::new(node))))
    }
    pub fn find_assign_item(&self, cursor: usize) -> Result<Option<(usize, AssignmentList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::AssignItem))
            .map(|option| option.map(|(i, node)| (i, AssignmentList::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitwiseAnd<T>(T);
#[allow(dead_code)]
impl<'a, N> BitwiseAnd<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        BitwiseAnd(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_equality(&self, cursor: usize) -> Result<Option<(usize, BitwiseAnd<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Equality))
            .map(|option| option.map(|(i, node)| (i, BitwiseAnd::new(node))))
    }
    pub fn find_bitwise_and_list(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, BitwiseAnd<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::BitwiseAndList),
            )
            .map(|option| option.map(|(i, node)| (i, BitwiseAnd::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitwiseAndList<T>(T);
#[allow(dead_code)]
impl<'a, N> BitwiseAndList<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        BitwiseAndList(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_bitwise_and_list(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, BitwiseAndList<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::BitwiseAndList),
            )
            .map(|option| option.map(|(i, node)| (i, BitwiseAndList::new(node))))
    }
    pub fn find_bitwise_and_op(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, BitwiseAndList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::BitwiseAndOp))
            .map(|option| option.map(|(i, node)| (i, BitwiseAndList::new(node))))
    }
    pub fn find_equality(&self, cursor: usize) -> Result<Option<(usize, BitwiseAndList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Equality))
            .map(|option| option.map(|(i, node)| (i, BitwiseAndList::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitwiseAndOp<T>(T);
#[allow(dead_code)]
impl<'a, N> BitwiseAndOp<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        BitwiseAndOp(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_amp(&self, cursor: usize) -> Result<Option<(usize, BitwiseAndOp<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::Amp))
            .map(|option| option.map(|(i, node)| (i, BitwiseAndOp::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitwiseOr<T>(T);
#[allow(dead_code)]
impl<'a, N> BitwiseOr<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        BitwiseOr(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_bitwise_and(&self, cursor: usize) -> Result<Option<(usize, BitwiseOr<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::BitwiseAnd))
            .map(|option| option.map(|(i, node)| (i, BitwiseOr::new(node))))
    }
    pub fn find_bitwise_or_list(&self, cursor: usize) -> Result<Option<(usize, BitwiseOr<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::BitwiseOrList),
            )
            .map(|option| option.map(|(i, node)| (i, BitwiseOr::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitwiseOrList<T>(T);
#[allow(dead_code)]
impl<'a, N> BitwiseOrList<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        BitwiseOrList(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_bitwise_or_list(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, BitwiseOrList<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::BitwiseOrList),
            )
            .map(|option| option.map(|(i, node)| (i, BitwiseOrList::new(node))))
    }
    pub fn find_bitwise_or_op(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, BitwiseOrList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::BitwiseOrOp))
            .map(|option| option.map(|(i, node)| (i, BitwiseOrList::new(node))))
    }
    pub fn find_bitwise_and(&self, cursor: usize) -> Result<Option<(usize, BitwiseOrList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::BitwiseAnd))
            .map(|option| option.map(|(i, node)| (i, BitwiseOrList::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitwiseOrOp<T>(T);
#[allow(dead_code)]
impl<'a, N> BitwiseOrOp<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        BitwiseOrOp(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_or(&self, cursor: usize) -> Result<Option<(usize, BitwiseOrOp<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::Or))
            .map(|option| option.map(|(i, node)| (i, BitwiseOrOp::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitwiseShift<T>(T);
#[allow(dead_code)]
impl<'a, N> BitwiseShift<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        BitwiseShift(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_summ(&self, cursor: usize) -> Result<Option<(usize, BitwiseShift<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Summ))
            .map(|option| option.map(|(i, node)| (i, BitwiseShift::new(node))))
    }
    pub fn find_bitwise_shift_list(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, BitwiseShift<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::BitwiseShiftList),
            )
            .map(|option| option.map(|(i, node)| (i, BitwiseShift::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitwiseShiftList<T>(T);
#[allow(dead_code)]
impl<'a, N> BitwiseShiftList<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        BitwiseShiftList(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_bitwise_shift_list(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, BitwiseShiftList<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::BitwiseShiftList),
            )
            .map(|option| option.map(|(i, node)| (i, BitwiseShiftList::new(node))))
    }
    pub fn find_bitwise_shift_op(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, BitwiseShiftList<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::BitwiseShiftOp),
            )
            .map(|option| option.map(|(i, node)| (i, BitwiseShiftList::new(node))))
    }
    pub fn find_summ(&self, cursor: usize) -> Result<Option<(usize, BitwiseShiftList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Summ))
            .map(|option| option.map(|(i, node)| (i, BitwiseShiftList::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitwiseShiftOp<T>(T);
#[allow(dead_code)]
impl<'a, N> BitwiseShiftOp<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        BitwiseShiftOp(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_bitwise_shift_op(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, BitwiseShiftOp<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::BitwiseShiftOp))
            .map(|option| option.map(|(i, node)| (i, BitwiseShiftOp::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calc<T>(T);
#[allow(dead_code)]
impl<'a, N> Calc<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        Calc(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_calc_list(&self, cursor: usize) -> Result<Option<(usize, Calc<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::CalcList))
            .map(|option| option.map(|(i, node)| (i, Calc::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalcList<T>(T);
#[allow(dead_code)]
impl<'a, N> CalcList<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        CalcList(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_calc_list(&self, cursor: usize) -> Result<Option<(usize, CalcList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::CalcList))
            .map(|option| option.map(|(i, node)| (i, CalcList::new(node))))
    }
    pub fn find_instruction(&self, cursor: usize) -> Result<Option<(usize, CalcList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Instruction))
            .map(|option| option.map(|(i, node)| (i, CalcList::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Equality<T>(T);
#[allow(dead_code)]
impl<'a, N> Equality<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        Equality(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_relational(&self, cursor: usize) -> Result<Option<(usize, Equality<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Relational))
            .map(|option| option.map(|(i, node)| (i, Equality::new(node))))
    }
    pub fn find_equality_list(&self, cursor: usize) -> Result<Option<(usize, Equality<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::EqualityList))
            .map(|option| option.map(|(i, node)| (i, Equality::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EqualityList<T>(T);
#[allow(dead_code)]
impl<'a, N> EqualityList<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        EqualityList(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_equality_list(&self, cursor: usize) -> Result<Option<(usize, EqualityList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::EqualityList))
            .map(|option| option.map(|(i, node)| (i, EqualityList::new(node))))
    }
    pub fn find_equality_op(&self, cursor: usize) -> Result<Option<(usize, EqualityList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::EqualityOp))
            .map(|option| option.map(|(i, node)| (i, EqualityList::new(node))))
    }
    pub fn find_relational(&self, cursor: usize) -> Result<Option<(usize, EqualityList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Relational))
            .map(|option| option.map(|(i, node)| (i, EqualityList::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EqualityOp<T>(T);
#[allow(dead_code)]
impl<'a, N> EqualityOp<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        EqualityOp(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_equality_op(&self, cursor: usize) -> Result<Option<(usize, EqualityOp<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::EqualityOp))
            .map(|option| option.map(|(i, node)| (i, EqualityOp::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Factor<T> {
    Number(Number<T>),
    Negate(Negate<T>),
    IdRef(IdRef<T>),
    Invalid(T),
}
#[allow(dead_code)]
impl<'a, N> Factor<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        match node.kind() {
            NodeKind::NonTerminal(NonTerminalKind::Number) => Self::Number(Number::new(node)),
            NodeKind::NonTerminal(NonTerminalKind::Negate) => Self::Negate(Negate::new(node)),
            NodeKind::NonTerminal(NonTerminalKind::IdRef) => Self::IdRef(IdRef::new(node)),
            _ => Factor::Invalid(node),
        }
    }
    pub fn node(&self) -> &N {
        match self {
            Self::Number(node) => node.node(),
            Self::Negate(node) => node.node(),
            Self::IdRef(node) => node.node(),
            Self::Invalid(node) => node,
        }
    }
    pub fn node_mut(&mut self) -> &mut N {
        match self {
            Self::Number(node) => node.node_mut(),
            Self::Negate(node) => node.node_mut(),
            Self::IdRef(node) => node.node_mut(),
            Self::Invalid(node) => node,
        }
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Id<T>(T);
#[allow(dead_code)]
impl<'a, N> Id<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        Id(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_id(&self, cursor: usize) -> Result<Option<(usize, Id<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::Id))
            .map(|option| option.map(|(i, node)| (i, Id::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IdRef<T>(T);
#[allow(dead_code)]
impl<'a, N> IdRef<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        IdRef(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_id(&self, cursor: usize) -> Result<Option<(usize, IdRef<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Id))
            .map(|option| option.map(|(i, node)| (i, IdRef::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction<T> {
    Assignment(Assignment<T>),
    LogicalOr(LogicalOr<T>),
    Invalid(T),
}
#[allow(dead_code)]
impl<'a, N> Instruction<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        match node.kind() {
            NodeKind::NonTerminal(NonTerminalKind::Assignment) => {
                Self::Assignment(Assignment::new(node))
            }
            NodeKind::NonTerminal(NonTerminalKind::LogicalOr) => {
                Self::LogicalOr(LogicalOr::new(node))
            }
            _ => Instruction::Invalid(node),
        }
    }
    pub fn node(&self) -> &N {
        match self {
            Self::Assignment(node) => node.node(),
            Self::LogicalOr(node) => node.node(),
            Self::Invalid(node) => node,
        }
    }
    pub fn node_mut(&mut self) -> &mut N {
        match self {
            Self::Assignment(node) => node.node_mut(),
            Self::LogicalOr(node) => node.node_mut(),
            Self::Invalid(node) => node,
        }
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalAnd<T>(T);
#[allow(dead_code)]
impl<'a, N> LogicalAnd<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        LogicalAnd(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_bitwise_or(&self, cursor: usize) -> Result<Option<(usize, LogicalAnd<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::BitwiseOr))
            .map(|option| option.map(|(i, node)| (i, LogicalAnd::new(node))))
    }
    pub fn find_logical_and_list(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, LogicalAnd<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::LogicalAndList),
            )
            .map(|option| option.map(|(i, node)| (i, LogicalAnd::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalAndList<T>(T);
#[allow(dead_code)]
impl<'a, N> LogicalAndList<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        LogicalAndList(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_logical_and_list(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, LogicalAndList<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::LogicalAndList),
            )
            .map(|option| option.map(|(i, node)| (i, LogicalAndList::new(node))))
    }
    pub fn find_logical_and_op(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, LogicalAndList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::LogicalAndOp))
            .map(|option| option.map(|(i, node)| (i, LogicalAndList::new(node))))
    }
    pub fn find_bitwise_or(&self, cursor: usize) -> Result<Option<(usize, LogicalAndList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::BitwiseOr))
            .map(|option| option.map(|(i, node)| (i, LogicalAndList::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalAndOp<T>(T);
#[allow(dead_code)]
impl<'a, N> LogicalAndOp<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        LogicalAndOp(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_amp_amp(&self, cursor: usize) -> Result<Option<(usize, LogicalAndOp<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::AmpAmp))
            .map(|option| option.map(|(i, node)| (i, LogicalAndOp::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalOr<T>(T);
#[allow(dead_code)]
impl<'a, N> LogicalOr<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        LogicalOr(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_logical_and(&self, cursor: usize) -> Result<Option<(usize, LogicalOr<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::LogicalAnd))
            .map(|option| option.map(|(i, node)| (i, LogicalOr::new(node))))
    }
    pub fn find_logical_or_list(&self, cursor: usize) -> Result<Option<(usize, LogicalOr<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::LogicalOrList),
            )
            .map(|option| option.map(|(i, node)| (i, LogicalOr::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalOrList<T>(T);
#[allow(dead_code)]
impl<'a, N> LogicalOrList<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        LogicalOrList(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_logical_or_list(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, LogicalOrList<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::LogicalOrList),
            )
            .map(|option| option.map(|(i, node)| (i, LogicalOrList::new(node))))
    }
    pub fn find_logical_or_op(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, LogicalOrList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::LogicalOrOp))
            .map(|option| option.map(|(i, node)| (i, LogicalOrList::new(node))))
    }
    pub fn find_logical_and(&self, cursor: usize) -> Result<Option<(usize, LogicalOrList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::LogicalAnd))
            .map(|option| option.map(|(i, node)| (i, LogicalOrList::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalOrOp<T>(T);
#[allow(dead_code)]
impl<'a, N> LogicalOrOp<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        LogicalOrOp(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_or_or(&self, cursor: usize) -> Result<Option<(usize, LogicalOrOp<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::OrOr))
            .map(|option| option.map(|(i, node)| (i, LogicalOrOp::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Minus<T>(T);
#[allow(dead_code)]
impl<'a, N> Minus<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        Minus(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_minus(&self, cursor: usize) -> Result<Option<(usize, Minus<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::Minus))
            .map(|option| option.map(|(i, node)| (i, Minus::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mult<T>(T);
#[allow(dead_code)]
impl<'a, N> Mult<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        Mult(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_power(&self, cursor: usize) -> Result<Option<(usize, Mult<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Power))
            .map(|option| option.map(|(i, node)| (i, Mult::new(node))))
    }
    pub fn find_mult_list(&self, cursor: usize) -> Result<Option<(usize, Mult<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::MultList))
            .map(|option| option.map(|(i, node)| (i, Mult::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MultList<T>(T);
#[allow(dead_code)]
impl<'a, N> MultList<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        MultList(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_mult_list(&self, cursor: usize) -> Result<Option<(usize, MultList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::MultList))
            .map(|option| option.map(|(i, node)| (i, MultList::new(node))))
    }
    pub fn find_mult_op(&self, cursor: usize) -> Result<Option<(usize, MultList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::MultOp))
            .map(|option| option.map(|(i, node)| (i, MultList::new(node))))
    }
    pub fn find_power(&self, cursor: usize) -> Result<Option<(usize, MultList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Power))
            .map(|option| option.map(|(i, node)| (i, MultList::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MultOp<T>(T);
#[allow(dead_code)]
impl<'a, N> MultOp<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        MultOp(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_mult_op(&self, cursor: usize) -> Result<Option<(usize, MultOp<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::MultOp))
            .map(|option| option.map(|(i, node)| (i, MultOp::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Negate<T>(T);
#[allow(dead_code)]
impl<'a, N> Negate<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        Negate(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_minus(&self, cursor: usize) -> Result<Option<(usize, Negate<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Minus))
            .map(|option| option.map(|(i, node)| (i, Negate::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Number<T>(T);
#[allow(dead_code)]
impl<'a, N> Number<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        Number(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_number(&self, cursor: usize) -> Result<Option<(usize, Number<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::Number))
            .map(|option| option.map(|(i, node)| (i, Number::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plus<T>(T);
#[allow(dead_code)]
impl<'a, N> Plus<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        Plus(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_plus(&self, cursor: usize) -> Result<Option<(usize, Plus<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::Plus))
            .map(|option| option.map(|(i, node)| (i, Plus::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowOp<T>(T);
#[allow(dead_code)]
impl<'a, N> PowOp<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        PowOp(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_star_star(&self, cursor: usize) -> Result<Option<(usize, PowOp<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::StarStar))
            .map(|option| option.map(|(i, node)| (i, PowOp::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Power<T>(T);
#[allow(dead_code)]
impl<'a, N> Power<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        Power(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_factor(&self, cursor: usize) -> Result<Option<(usize, Power<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Factor))
            .map(|option| option.map(|(i, node)| (i, Power::new(node))))
    }
    pub fn find_power_list(&self, cursor: usize) -> Result<Option<(usize, Power<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::PowerList))
            .map(|option| option.map(|(i, node)| (i, Power::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerList<T>(T);
#[allow(dead_code)]
impl<'a, N> PowerList<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        PowerList(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_power_list(&self, cursor: usize) -> Result<Option<(usize, PowerList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::PowerList))
            .map(|option| option.map(|(i, node)| (i, PowerList::new(node))))
    }
    pub fn find_pow_op(&self, cursor: usize) -> Result<Option<(usize, PowerList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::PowOp))
            .map(|option| option.map(|(i, node)| (i, PowerList::new(node))))
    }
    pub fn find_factor(&self, cursor: usize) -> Result<Option<(usize, PowerList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Factor))
            .map(|option| option.map(|(i, node)| (i, PowerList::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Relational<T>(T);
#[allow(dead_code)]
impl<'a, N> Relational<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        Relational(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_bitwise_shift(&self, cursor: usize) -> Result<Option<(usize, Relational<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::BitwiseShift))
            .map(|option| option.map(|(i, node)| (i, Relational::new(node))))
    }
    pub fn find_relational_list(&self, cursor: usize) -> Result<Option<(usize, Relational<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::RelationalList),
            )
            .map(|option| option.map(|(i, node)| (i, Relational::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelationalList<T>(T);
#[allow(dead_code)]
impl<'a, N> RelationalList<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        RelationalList(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_relational_list(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, RelationalList<N>)>, N> {
        self.0
            .find_child(
                cursor,
                NodeKind::NonTerminal(NonTerminalKind::RelationalList),
            )
            .map(|option| option.map(|(i, node)| (i, RelationalList::new(node))))
    }
    pub fn find_relational_op(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, RelationalList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::RelationalOp))
            .map(|option| option.map(|(i, node)| (i, RelationalList::new(node))))
    }
    pub fn find_bitwise_shift(
        &self,
        cursor: usize,
    ) -> Result<Option<(usize, RelationalList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::BitwiseShift))
            .map(|option| option.map(|(i, node)| (i, RelationalList::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelationalOp<T>(T);
#[allow(dead_code)]
impl<'a, N> RelationalOp<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        RelationalOp(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_relational_op(&self, cursor: usize) -> Result<Option<(usize, RelationalOp<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::Terminal(TerminalKind::RelationalOp))
            .map(|option| option.map(|(i, node)| (i, RelationalOp::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summ<T>(T);
#[allow(dead_code)]
impl<'a, N> Summ<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        Summ(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_mult(&self, cursor: usize) -> Result<Option<(usize, Summ<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Mult))
            .map(|option| option.map(|(i, node)| (i, Summ::new(node))))
    }
    pub fn find_summ_list(&self, cursor: usize) -> Result<Option<(usize, Summ<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::SummList))
            .map(|option| option.map(|(i, node)| (i, Summ::new(node))))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SummList<T>(T);
#[allow(dead_code)]
impl<'a, N> SummList<N>
where
    N: Node<'a, TerminalKind, NonTerminalKind>,
{
    pub fn new(node: N) -> Self {
        SummList(node)
    }
    pub fn node(&self) -> &N {
        &self.0
    }
    pub fn node_mut(&mut self) -> &mut N {
        &mut self.0
    }
    pub fn find_summ_list(&self, cursor: usize) -> Result<Option<(usize, SummList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::SummList))
            .map(|option| option.map(|(i, node)| (i, SummList::new(node))))
    }
    pub fn find_add_op(&self, cursor: usize) -> Result<Option<(usize, SummList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::AddOp))
            .map(|option| option.map(|(i, node)| (i, SummList::new(node))))
    }
    pub fn find_mult(&self, cursor: usize) -> Result<Option<(usize, SummList<N>)>, N> {
        self.0
            .find_child(cursor, NodeKind::NonTerminal(NonTerminalKind::Mult))
            .map(|option| option.map(|(i, node)| (i, SummList::new(node))))
    }
}
