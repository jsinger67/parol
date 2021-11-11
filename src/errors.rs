error_chain! {
    foreign_links {
        IdTreeErr(id_tree::NodeIdError);
    }

}
