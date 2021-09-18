error_chain! {
    links {
        RuntimeParserErr(parol_runtime::parser::errors::Error, parol_runtime::parser::errors::ErrorKind);
    }
}
