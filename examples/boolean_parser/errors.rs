error_chain! {
    links {
        RuntimeParserErr(parol_runtime::errors::Error, parol_runtime::errors::ErrorKind);
    }
}
