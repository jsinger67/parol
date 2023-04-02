use crate::{CompiledTerminal, KTuple, MAX_K};

use super::{compiled_la_dfa::TerminalIndex, compiled_terminal::INVALID, k_tuple::Terminals};

#[derive(Debug, Clone)]
pub(crate) struct Node {
    t: TerminalIndex,
    c: Vec<Node>,
}

impl Node {
    /// Creates a new [`Node`].
    pub(crate) fn new(t: TerminalIndex) -> Self {
        Self {
            t,
            c: Default::default(),
        }
    }

    pub(crate) fn with_children(mut self, children: &[TerminalIndex]) -> Self {
        children.iter().for_each(|t| {
            let _ = self.add_child(*t);
        });
        self
    }

    /// Returns the terminal of this [`Node`].
    pub(crate) fn terminal(&self) -> TerminalIndex {
        self.t
    }

    /// Returns a reference to the children of this [`Node`].
    pub(crate) fn children(&self) -> &[Node] {
        &self.c
    }

    /// Returns a mutable reference to the children of this [`Node`].
    pub(crate) fn children_mut(&mut self) -> &mut [Node] {
        &mut self.c
    }

    /// Checks if the given terminal is in the node's list of children
    pub(crate) fn is_child(&self, t: TerminalIndex) -> bool {
        self.c.iter().find(|n| n.t == t).is_some()
    }

    /// Checks if self's terminal is valid
    pub(crate) fn is_valid(&self) -> bool {
        self.t != INVALID
    }

    /// Returns the index of the given terminal is in the node's list of children if it exists
    pub(crate) fn child_index(&self, t: TerminalIndex) -> Option<usize> {
        self.c.iter().position(|n| n.t == t)
    }

    /// Adds a child node if it not already exists and returns the child index of it
    pub(crate) fn add_child(&mut self, t: TerminalIndex) -> usize {
        if let Some(index) = self.child_index(t) {
            index
        } else {
            let idx = self.c.len();
            self.c.push(Node::new(t));
            idx
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            t: INVALID,
            c: Default::default(),
        }
    }
}

// The nodes identity is determined solely by its terminal value
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Trie {
    /// The root node's terminal index is always INVALID!
    root: Node,
}

impl Trie {
    /// Creates a new [`Trie`].
    pub(crate) fn new() -> Self {
        Trie::default()
    }

    /// Returns a reference to the root of this [`Trie`].
    pub(crate) fn root(&self) -> &Node {
        &self.root
    }

    /// Inserts a KTuple
    pub(crate) fn insert(&mut self, tuple: &KTuple) {
        let Terminals { t, i } = tuple.terminals.inner();
        if t.is_empty() {
            return;
        }
        let mut node = &mut self.root;
        for t in &t[0..*i] {
            let child_index = node.add_child(t.0);
            node = &mut node.children_mut()[child_index];
        }
    }

    /// Appends another Trie item to self
    pub fn append(&mut self, mut other: Self) -> bool {
        todo!()
    }

    pub(crate) fn terminals(&self) -> TerminalsIter<'_> {
        TerminalsIter::new(self)
    }
}

#[derive(Debug)]
pub(crate) struct TerminalsIter<'a> {
    // Tuple of the traversed node and the child index
    v: Vec<(&'a Node, usize)>,
    // Current node in DFS order
    n: &'a Node,
    // Next index in node's children
    i: usize,
    // Reference to the trie
    t: &'a Trie,
}

impl<'a> TerminalsIter<'a> {
    pub(crate) fn new(t: &'a Trie) -> Self {
        let mut this = Self {
            v: Vec::with_capacity(MAX_K),
            n: t.root(),
            i: 0,
            t,
        };
        this.expand(0);
        this
    }

    fn expand(&mut self, mut i: usize) {
        let mut node = self.n;
        loop {
            if node.children().len() <= i {
                break;
            }
            println!("push {} c[{}]", node.terminal(), i);
            self.v.push((node, i));
            node = &node.children()[i];
            i = 0;
        }
    }

    fn advance(&mut self) {
        self.v.pop().map(|(n, i)| {
            println!("pop  {} c[{}]", n.terminal(), i);
            if n.children().len() > i {
                self.n = n;
                self.expand(i + 1);
            } else {
                self.advance();
            }
        });
    }
}

impl Iterator for TerminalsIter<'_> {
    type Item = Terminals;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.v.is_empty() {
            None
        } else {
            Some(Terminals::from_slice_with(
                &self.v,
                self.v.len(),
                |(n, _)| CompiledTerminal(n.terminal()),
            ))
        };
        self.advance();
        result
    }
}

#[cfg(test)]
mod test {
    use crate::{
        analysis::{
            compiled_terminal::INVALID,
            terminals_trie::{Node, Trie},
        },
        KTuple,
    };

    #[test]
    fn node_new() {
        let n = Node::new(42);
        assert_eq!(n.t, 42);
        assert!(n.c.is_empty());
        assert!(n.is_valid());
    }

    #[test]
    fn node_default() {
        let n = Node::default();
        assert_eq!(n.t, INVALID);
        assert!(!n.is_valid());
    }

    #[test]
    fn node_terminal() {
        let n = Node::new(42);
        assert_eq!(n.terminal(), 42);
    }

    #[test]
    fn node_children() {
        let mut n = Node::new(42);
        n.add_child(7);
        assert_eq!(n.child_index(7), Some(0));
    }

    #[test]
    fn node_is_child() {
        let mut n = Node::new(42);
        n.add_child(7);
        assert!(n.is_child(7));
    }

    #[test]
    fn trie_new() {
        let t = Trie::new();
        assert_eq!(t.root(), &Node::default());
    }

    #[test]
    fn trie_insert() {
        let mut t = Trie::new();
        let tuple1 = KTuple::new(5).with_terminal_indices(&[1, 2, 3]);
        t.insert(&tuple1);
        assert_eq!(t.root().t, INVALID);
        assert!(t.root().is_child(1));
        assert_eq!(t.root().children().len(), 1);

        assert!(t.root().children()[0].is_child(2));
        assert_eq!(t.root().children()[0].children().len(), 1);
        assert_eq!(t.root().children()[0].children()[0].t, 2);

        assert!(t.root().children()[0].children()[0].is_child(3));
        assert_eq!(t.root().children()[0].children()[0].children().len(), 1);
        assert_eq!(t.root().children()[0].children()[0].children()[0].t, 3);
    }

    #[test]
    fn trie_multiple_inserts_no_change() {
        let mut t = Trie::new();
        let tuple1 = KTuple::new(5).with_terminal_indices(&[1, 2, 3]);
        let tuple2 = KTuple::new(5).with_terminal_indices(&[1, 2, 3]);
        //     1
        //     |
        //     2
        //     |
        //     3
        t.insert(&tuple1);
        t.insert(&tuple2);
        assert_eq!(t.root().t, INVALID);
        assert!(t.root().is_child(1));
        assert_eq!(t.root().children().len(), 1);

        assert!(t.root().children()[0].is_child(2));
        assert_eq!(t.root().children()[0].children().len(), 1);
        assert_eq!(t.root().children()[0].children()[0].t, 2);

        assert!(t.root().children()[0].children()[0].is_child(3));
        assert_eq!(t.root().children()[0].children()[0].children().len(), 1);
        assert_eq!(t.root().children()[0].children()[0].children()[0].t, 3);
    }

    #[test]
    fn trie_multiple_inserts_single_root() {
        let mut t = Trie::new();
        let tuple1 = KTuple::new(6).with_terminal_indices(&[1, 2, 3]);
        let tuple2 = KTuple::new(6).with_terminal_indices(&[1, 5, 6]);
        //      1
        //     / \
        //     2  5
        //     |  |
        //     3  6
        t.insert(&tuple1);
        t.insert(&tuple2);
        assert_eq!(t.root().t, INVALID);
        assert!(t.root().is_child(1));
        assert_eq!(t.root().children().len(), 1);

        assert!(t.root().children()[0].is_child(2));
        assert!(t.root().children()[0].is_child(5));
        assert_eq!(t.root().children()[0].children().len(), 2);
        assert_eq!(t.root().children()[0].children()[0].t, 2);
        assert_eq!(t.root().children()[0].children()[1].t, 5);

        assert!(t.root().children()[0].children()[0].is_child(3));
        assert!(t.root().children()[0].children()[1].is_child(6));
        assert_eq!(t.root().children()[0].children()[0].children().len(), 1);
        assert_eq!(t.root().children()[0].children()[1].children().len(), 1);
        assert_eq!(t.root().children()[0].children()[0].children()[0].t, 3);
        assert_eq!(t.root().children()[0].children()[1].children()[0].t, 6);
    }

    #[test]
    fn trie_multiple_inserts_two_roots() {
        let mut t = Trie::new();
        let tuple1 = KTuple::new(6).with_terminal_indices(&[1, 2, 3]);
        let tuple2 = KTuple::new(6).with_terminal_indices(&[4, 5, 6]);
        //     1  4
        //     |  |
        //     2  5
        //     |  |
        //     3  6
        t.insert(&tuple1);
        t.insert(&tuple2);
        assert_eq!(t.root().t, INVALID);
        assert!(t.root().is_child(1));
        assert!(t.root().is_child(4));
        assert_eq!(t.root().children().len(), 2);

        assert!(t.root().children()[0].is_child(2));
        assert!(t.root().children()[1].is_child(5));
        assert_eq!(t.root().children()[0].children().len(), 1);
        assert_eq!(t.root().children()[0].children()[0].t, 2);
        assert_eq!(t.root().children()[1].children()[0].t, 5);

        assert!(t.root().children()[0].children()[0].is_child(3));
        assert!(t.root().children()[1].children()[0].is_child(6));
        assert_eq!(t.root().children()[0].children()[0].children().len(), 1);
        assert_eq!(t.root().children()[1].children()[0].children().len(), 1);
        assert_eq!(t.root().children()[0].children()[0].children()[0].t, 3);
        assert_eq!(t.root().children()[1].children()[0].children()[0].t, 6);
    }

    #[test]
    fn trie_terminals() {
        let mut t = Trie::new();
        let tuple1 = KTuple::new(6).with_terminal_indices(&[1, 2, 3]);
        let tuple2 = KTuple::new(6).with_terminal_indices(&[1, 2, 4]);
        let tuple3 = KTuple::new(6).with_terminal_indices(&[5, 6, 7]);
        let tuple4 = KTuple::new(6).with_terminal_indices(&[5, 8]);
        //     1     5
        //     |     | \
        //     2     6  8
        //     | \   |
        //     3  4  7
        t.insert(&tuple1);
        t.insert(&tuple2);
        t.insert(&tuple3);
        t.insert(&tuple4);

        for t in t.terminals() {
            println!(
                "[{}]",
                t.t.iter()
                    .skip(1)
                    .take_while(|t| t.0 != INVALID)
                    .map(|e| format!("{}", e))
                    .collect::<Vec<String>>()
                    .join(", "),
            );
        }
    }
}
