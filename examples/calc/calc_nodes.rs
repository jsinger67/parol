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
    LogicalOrOp,
    LogicalAndOp,
    BitwiseOrOp,
    BitwiseAndOp,
    BitwiseShiftOp,
    RelationalOp,
    Plus,
    Minus,
    PowOp,
    MultOp,
    LParen,
    RParen,
    Number,
    Id,
}
impl TerminalKind {
    pub fn from_terminal_index(index: u16) -> Self {
        match index {
            1 => Self::NewLine,
            2 => Self::Whitespace,
            3 => Self::LineComment,
            4 => Self::BlockComment,
            5 => Self::Semicolon,
            6 => Self::EqualityOp,
            7 => Self::AssignOp,
            8 => Self::LogicalOrOp,
            9 => Self::LogicalAndOp,
            10 => Self::BitwiseOrOp,
            11 => Self::BitwiseAndOp,
            12 => Self::BitwiseShiftOp,
            13 => Self::RelationalOp,
            14 => Self::Plus,
            15 => Self::Minus,
            16 => Self::PowOp,
            17 => Self::MultOp,
            18 => Self::LParen,
            19 => Self::RParen,
            20 => Self::Number,
            21 => Self::Id,
            _ => panic!("Invalid terminal index: {}", index),
        }
    }
    pub fn is_builtin_terminal(&self) -> bool {
        matches!(
            self,
            TerminalKind::NewLine
                | TerminalKind::Whitespace
                | TerminalKind::LineComment
                | TerminalKind::BlockComment
        )
    }
    pub fn is_builtin_new_line(&self) -> bool {
        matches!(self, TerminalKind::NewLine)
    }
    pub fn is_builtin_whitespace(&self) -> bool {
        matches!(self, TerminalKind::Whitespace)
    }
    pub fn is_builtin_line_comment(&self) -> bool {
        matches!(self, TerminalKind::LineComment)
    }
    pub fn is_builtin_block_comment(&self) -> bool {
        matches!(self, TerminalKind::BlockComment)
    }
}

impl NonTerminalKind {
    pub fn from_non_terminal_name(name: &str) -> Self {
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
            Self::LogicalOrOp => write!(f, stringify!(LogicalOrOp)),
            Self::LogicalAndOp => write!(f, stringify!(LogicalAndOp)),
            Self::BitwiseOrOp => write!(f, stringify!(BitwiseOrOp)),
            Self::BitwiseAndOp => write!(f, stringify!(BitwiseAndOp)),
            Self::BitwiseShiftOp => write!(f, stringify!(BitwiseShiftOp)),
            Self::RelationalOp => write!(f, stringify!(RelationalOp)),
            Self::Plus => write!(f, stringify!(Plus)),
            Self::Minus => write!(f, stringify!(Minus)),
            Self::PowOp => write!(f, stringify!(PowOp)),
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
            Self::Root => write!(f, stringify!(Root)),
        }
    }
}
