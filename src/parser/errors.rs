error_chain! {

    links {
        LexerErr(crate::lexer::errors::Error, crate::lexer::errors::ErrorKind);
    }

    foreign_links {
        IdTreeErr(id_tree::NodeIdError);
    }

}
