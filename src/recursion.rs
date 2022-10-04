use lsp_types::Range;

use crate::parol_ls_grammar_trait::{
    Alternation, Alternations, Factor, Group, Optional, Production, Repeat, Symbol,
};

pub(crate) trait RecursionDetection {
    fn find_left_recursion(&self, productions: &[&Production]) -> Option<Vec<Range>>;
}

impl RecursionDetection for &Production {
    fn find_left_recursion(&self, productions: &[&Production]) -> Option<Vec<Range>> {
        let non_terminal = &self.production_l_h_s.identifier.identifier.symbol;
        let recursions = self.alternations.recurse_for(non_terminal, productions);
        if recursions.is_empty() {
            None
        } else {
            Some(recursions)
        }
    }
}

trait RecurseFor {
    fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range>;
}

// impl RecurseFor for ASTControl {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
impl RecurseFor for Alternation {
    fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
        for alt in &self.alternation_list {
            let recursions = alt.factor.recurse_for(non_terminal, productions);
            if !recursions.is_empty() {
                return recursions;
            }
        }
        return vec![];
    }
}
impl RecurseFor for Alternations {
    fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
        let recursions = self.alternation.recurse_for(non_terminal, productions);
        if !recursions.is_empty() {
            return recursions;
        }
        for alt in &self.alternations_list {
            let recursions = alt.alternation.recurse_for(non_terminal, productions);
            if !recursions.is_empty() {
                return recursions;
            }
        }
        return vec![];
    }
}
// impl RecurseFor for BlockComment {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for Comments {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for CommentsListGroup {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for CutOperator {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for Declaration {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for DoubleColon {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
impl RecurseFor for Factor {
    fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
        match self {
            Factor::Factor0(group) => group.group.recurse_for(non_terminal, productions),
            Factor::Factor1(repeat) => repeat.repeat.recurse_for(non_terminal, productions),
            Factor::Factor2(optional) => optional.optional.recurse_for(non_terminal, productions),
            Factor::Factor3(symbol) => symbol.symbol.recurse_for(non_terminal, productions),
        }
    }
}
// impl RecurseFor for GrammarDefinition {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
impl RecurseFor for Group {
    fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
        self.alternations.recurse_for(non_terminal, productions)
    }
}
// impl RecurseFor for Identifier {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for LineComment {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for NonTerminal {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
impl RecurseFor for Optional {
    fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
        todo!()
    }
}
// impl RecurseFor for ParolLs {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for Production {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for ProductionLHS {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for Prolog {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
impl RecurseFor for Repeat {
    fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
        todo!()
    }
}
// impl RecurseFor for ScannerDirectives {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for ScannerState {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for ScannerSwitch {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for SimpleToken {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for StartDeclaration {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for StateList {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for String {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
impl RecurseFor for Symbol {
    fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
        todo!()
    }
}
// impl RecurseFor for TokenWithStates {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for UserTypeDeclaration {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
// impl RecurseFor for UserTypeName {
//     fn recurse_for(&self, non_terminal: &str, productions: &[&Production]) -> Vec<Range> {
//         todo!()
//     }
// }
