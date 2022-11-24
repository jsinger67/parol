use crate::parol_ls_grammar_trait::*;
use crate::rng::Rng;

impl From<&ASTControl> for Rng {
    fn from(val: &ASTControl) -> Self {
        match val {
            ASTControl::ASTControl0(cut) => Self::from(&*cut.cut_operator),
            ASTControl::ASTControl1(ut) => Self::from(&*ut.user_type_declaration),
        }
    }
}

impl From<&Alternation> for Rng {
    fn from(val: &Alternation) -> Self {
        Self::from_slice(&val.alternation_list)
    }
}

impl From<&AlternationList> for Rng {
    fn from(val: &AlternationList) -> Self {
        Self::from(&*val.factor).extend(Self::from(&*val.comments))
    }
}

impl From<&Alternations> for Rng {
    fn from(val: &Alternations) -> Self {
        Self::from(&*val.alternation).extend(Self::from_slice(&val.alternations_list))
    }
}

impl From<&AlternationsList> for Rng {
    fn from(val: &AlternationsList) -> Self {
        Self::from(&val.or).extend(Self::from(&*val.alternation))
    }
}

impl From<&BlockComment> for Rng {
    fn from(val: &BlockComment) -> Self {
        Self::from(&val.block_comment)
    }
}

impl From<&Comments> for Rng {
    fn from(val: &Comments) -> Self {
        Self::from_slice(&val.comments_list)
    }
}

impl From<&CommentsList> for Rng {
    fn from(val: &CommentsList) -> Self {
        Self::from(&*val.comments_list_group)
    }
}

impl From<&CommentsListGroup> for Rng {
    fn from(val: &CommentsListGroup) -> Self {
        match val {
            CommentsListGroup::CommentsListGroup0(lc) => Self::from(&*lc.line_comment),
            CommentsListGroup::CommentsListGroup1(bc) => Self::from(&*bc.block_comment),
        }
    }
}

impl From<&CutOperator> for Rng {
    fn from(val: &CutOperator) -> Self {
        Self::from(&val.cut_operator)
    }
}

impl From<&Declaration> for Rng {
    fn from(val: &Declaration) -> Self {
        match val {
            Declaration::Declaration0(title) => Self::from(&title.percent_title)
                .extend(Self::from(&*title.string))
                .extend(Self::from(&*title.comments)),
            Declaration::Declaration1(comment) => Self::from(&comment.percent_comment)
                .extend(Self::from(&*comment.string))
                .extend(Self::from(&*comment.comments)),
            Declaration::Declaration2(user_type) => {
                Self::from(&user_type.percent_user_underscore_type)
                    .extend(Self::from(&*user_type.user_type_name))
                    .extend(Self::from(&*user_type.comments))
            }
            Declaration::Declaration3(scanner) => Self::from(&*scanner.scanner_directives),
        }
    }
}

impl From<&DoubleColon> for Rng {
    fn from(val: &DoubleColon) -> Self {
        Self::from(&val.double_colon)
    }
}

impl From<&Factor> for Rng {
    fn from(val: &Factor) -> Self {
        match val {
            Factor::Factor0(grp) => Self::from(&*grp.group),
            Factor::Factor1(rpt) => Self::from(&*rpt.repeat),
            Factor::Factor2(opt) => Self::from(&*opt.optional),
            Factor::Factor3(sym) => Self::from(&*sym.symbol),
        }
    }
}

impl From<&GrammarDefinition> for Rng {
    fn from(val: &GrammarDefinition) -> Self {
        Self::from(&val.percent_percent)
            .extend(Self::from(&*val.production))
            .extend(Self::from_slice(&val.grammar_definition_list))
    }
}

impl From<&GrammarDefinitionList> for Rng {
    fn from(val: &GrammarDefinitionList) -> Self {
        Self::from(&*val.production)
    }
}

impl From<&Group> for Rng {
    fn from(val: &Group) -> Self {
        Self::from(&val.l_paren).extend(Self::from(&val.r_paren))
    }
}

impl From<&Identifier> for Rng {
    fn from(val: &Identifier) -> Self {
        Self::from(&val.identifier)
    }
}

impl From<&LineComment> for Rng {
    fn from(val: &LineComment) -> Self {
        Self::from(&val.line_comment)
    }
}

impl From<&NonTerminal> for Rng {
    fn from(val: &NonTerminal) -> Self {
        let rng = Self::from(&val.identifier.identifier);
        val.non_terminal_opt
            .as_ref()
            .map_or(rng, |non_terminal_opt| {
                rng.extend(Self::from(&**non_terminal_opt))
            })
    }
}

impl From<&NonTerminalOpt> for Rng {
    fn from(val: &NonTerminalOpt) -> Self {
        Self::from(&*val.a_s_t_control)
    }
}

impl From<&Optional> for Rng {
    fn from(val: &Optional) -> Self {
        Self::from(&val.l_bracket).extend(Self::from(&val.r_bracket))
    }
}

impl From<&ParolLs> for Rng {
    fn from(val: &ParolLs) -> Self {
        Self::from(&*val.prolog).extend(Self::from(&*val.grammar_definition))
    }
}

impl From<&Production> for Rng {
    fn from(val: &Production) -> Self {
        Self::from(&*val.production_l_h_s).extend(Self::from(&val.semicolon))
    }
}

impl From<&ProductionLHS> for Rng {
    fn from(val: &ProductionLHS) -> Self {
        Self::from(&*val.comments)
            .extend(Self::from(&val.identifier.identifier))
            .extend(Self::from(&val.colon))
    }
}

impl From<&Prolog> for Rng {
    fn from(val: &Prolog) -> Self {
        Self::from(&*val.start_declaration)
            .extend(Self::from_slice(&val.prolog_list))
            .extend(Self::from_slice(&val.prolog_list0))
    }
}

impl From<&PrologList> for Rng {
    fn from(val: &PrologList) -> Self {
        Self::from(&*val.declaration)
    }
}

impl From<&PrologList0> for Rng {
    fn from(val: &PrologList0) -> Self {
        Self::from(&*val.scanner_state)
    }
}

impl From<&Repeat> for Rng {
    fn from(val: &Repeat) -> Self {
        Self::from(&val.l_brace).extend(Self::from(&val.r_brace))
    }
}

impl From<&ScannerDirectives> for Rng {
    fn from(val: &ScannerDirectives) -> Self {
        match val {
            ScannerDirectives::ScannerDirectives0(lc) => {
                Self::from(&lc.percent_line_underscore_comment)
                    .extend(Self::from(&*lc.token_literal))
                    .extend(Self::from(&*lc.comments))
            }
            ScannerDirectives::ScannerDirectives1(bc) => {
                Self::from(&bc.percent_block_underscore_comment)
                    .extend(Self::from(&*bc.token_literal0))
                    .extend(Self::from(&*bc.comments))
            }
            ScannerDirectives::ScannerDirectives2(auto_nl) => {
                Self::from(&auto_nl.percent_auto_underscore_newline_underscore_off)
                    .extend(Self::from(&*auto_nl.comments))
            }
            ScannerDirectives::ScannerDirectives3(auto_ws) => {
                Self::from(&auto_ws.percent_auto_underscore_ws_underscore_off)
                    .extend(Self::from(&*auto_ws.comments))
            }
        }
    }
}

impl From<&ScannerState> for Rng {
    fn from(val: &ScannerState) -> Self {
        Self::from(&val.percent_scanner).extend(Self::from(&val.r_brace))
    }
}

impl From<&ScannerStateList> for Rng {
    fn from(val: &ScannerStateList) -> Self {
        Self::from(&*val.scanner_directives)
    }
}

impl From<&ScannerSwitch> for Rng {
    fn from(val: &ScannerSwitch) -> Self {
        match val {
            ScannerSwitch::ScannerSwitch0(sc) => {
                Self::from(&sc.percent_sc).extend(Self::from(&sc.r_paren))
            }
            ScannerSwitch::ScannerSwitch1(push) => {
                Self::from(&push.percent_push).extend(Self::from(&push.r_paren))
            }
            ScannerSwitch::ScannerSwitch2(pop) => {
                Self::from(&pop.percent_pop).extend(Self::from(&pop.r_paren))
            }
        }
    }
}

impl From<&ScannerSwitchOpt> for Rng {
    fn from(val: &ScannerSwitchOpt) -> Self {
        Self::from(&val.identifier.identifier)
    }
}

impl From<&SimpleToken> for Rng {
    fn from(val: &SimpleToken) -> Self {
        let rng = Self::from(&*val.token_literal);
        val.simple_token_opt
            .as_ref()
            .map_or(rng, |simple_token_opt| {
                rng.extend(Self::from(&**simple_token_opt))
            })
    }
}

impl From<&SimpleTokenOpt> for Rng {
    fn from(val: &SimpleTokenOpt) -> Self {
        Self::from(&*val.a_s_t_control)
    }
}

impl From<&StartDeclaration> for Rng {
    fn from(val: &StartDeclaration) -> Self {
        Self::from(&*val.comments)
            .extend(Self::from(&val.percent_start))
            .extend(Self::from(&val.identifier.identifier))
            .extend(Self::from(&*val.comments0))
    }
}

impl From<&StateList> for Rng {
    fn from(val: &StateList) -> Self {
        let rng = Self::from(&val.identifier.identifier);
        val.state_list_list.last().map_or(rng, |state_list_list| {
            rng.extend(Self::from(state_list_list))
        })
    }
}

impl From<&StateListList> for Rng {
    fn from(val: &StateListList) -> Self {
        Self::from(&val.comma).extend(Self::from(&val.identifier.identifier))
    }
}

impl From<&String> for Rng {
    fn from(val: &String) -> Self {
        (&val.string).into()
    }
}

impl From<&Symbol> for Rng {
    fn from(val: &Symbol) -> Self {
        match val {
            Symbol::Symbol0(nt) => Self::from(&nt.non_terminal.identifier.identifier),
            Symbol::Symbol1(to) => Self::from(&*to.simple_token),
            Symbol::Symbol2(ts) => Self::from(&*ts.token_with_states),
            Symbol::Symbol3(sw) => Self::from(&*sw.scanner_switch),
        }
    }
}

impl From<&TokenLiteral> for Rng {
    fn from(val: &TokenLiteral) -> Self {
        match val {
            TokenLiteral::TokenLiteral0(s) => Self::from(&s.string.string),
            TokenLiteral::TokenLiteral1(l) => Self::from(&l.literal_string.literal_string),
            TokenLiteral::TokenLiteral2(r) => Self::from(&r.regex.regex),
        }
    }
}

impl From<&TokenWithStates> for Rng {
    fn from(val: &TokenWithStates) -> Self {
        let rng = Self::from(&val.l_t);
        val.token_with_states_opt.as_ref().map_or(
            rng.extend(Self::from(&*val.token_literal)),
            |token_with_states| rng.extend(Self::from(&**token_with_states)),
        )
    }
}

impl From<&TokenWithStatesOpt> for Rng {
    fn from(val: &TokenWithStatesOpt) -> Self {
        Self::from(&*val.a_s_t_control)
    }
}

impl From<&UserTypeDeclaration> for Rng {
    fn from(val: &UserTypeDeclaration) -> Self {
        Self::from(&val.colon).extend(Self::from(&*val.user_type_name))
    }
}

impl From<&UserTypeName> for Rng {
    fn from(val: &UserTypeName) -> Self {
        let rng = Self::from(&val.identifier.identifier);
        val.user_type_name_list
            .last()
            .map_or(rng, |u| rng.extend(Self::from(u)))
    }
}

impl From<&UserTypeNameList> for Rng {
    fn from(val: &UserTypeNameList) -> Self {
        Self::from(&val.double_colon.double_colon).extend(Self::from(&val.identifier.identifier))
    }
}
