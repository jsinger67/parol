use bitflags::bitflags;
use std::{
    fmt::{Display, Formatter},
    ops::{Index, IndexMut},
};

use crate::{CompiledTerminal, KTuple, MAX_K};

use super::{compiled_la_dfa::TerminalIndex, compiled_terminal::INVALID, k_tuple::Terminals};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Node {
    // Node data
    t: TerminalIndex,
    // Children
    c: Vec<Node>,
    // End node
    e: bool,
}

impl Node {
    /// Creates a new [`Node`].
    pub(crate) fn new(t: TerminalIndex) -> Self {
        Self {
            t,
            ..Default::default()
        }
    }

    /// Returns the terminal of this [`Node`].
    #[inline]
    pub(crate) fn terminal(&self) -> TerminalIndex {
        self.t
    }

    /// Returns the is inner end node property of this [`Node`].
    /// It is true if node is an end node and if it has children!
    #[inline]
    pub(crate) fn is_inner_end_node(&self) -> bool {
        self.e && !self.c.is_empty()
    }

    /// Sets the end property of this [`Node`].
    #[inline]
    fn set_end(&mut self) {
        self.e = true
    }

    /// Returns a reference to the children of this [`Node`].
    #[inline]
    fn children(&self) -> &[Node] {
        &self.c
    }

    /// Returns a mutable reference to the children of this [`Node`].
    #[inline]
    fn children_mut(&mut self) -> &mut [Node] {
        &mut self.c
    }

    /// Returns the index of the given terminal is in the node's list of children if it exists
    fn child_index(&self, t: TerminalIndex) -> Option<usize> {
        self.c.binary_search(&Node::new(t)).ok()
    }

    /// Adds a child node if it not already exists and returns the child index of it
    /// The boolean in the return value ist true on insertion (collection changed)
    fn add_child(&mut self, t: TerminalIndex) -> (usize, bool) {
        if let Some(index) = self.child_index(t) {
            (index, false)
        } else {
            let idx = self.c.partition_point(|n| n.t < t);
            // insert in sort order
            self.c.insert(idx, Node::new(t));
            (idx, true)
        }
    }
}

impl Index<usize> for Node {
    type Output = Node;

    fn index(&self, index: usize) -> &Self::Output {
        &self.c[index]
    }
}

impl IndexMut<usize> for Node {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.c[index]
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.t.cmp(&other.t)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.t.partial_cmp(&other.t)
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            t: INVALID,
            c: Default::default(),
            e: Default::default(),
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let t = if self.t == INVALID {
            "INVALID".to_string()
        } else {
            self.t.to_string()
        };
        write!(f, "{t}")
    }
}

#[derive(Debug, Clone, Default, Eq)]
pub(crate) struct Trie {
    /// The root node's terminal index is always INVALID!
    root: Node,
    /// The length counter
    len: usize,
}

impl Trie {
    /// Creates a new [`Trie`].
    pub(crate) fn new() -> Self {
        Trie::default()
    }

    /// Returns the number of tuples in this [`Trie`].
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Checks if the collection is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Inserts a KTuple
    pub(crate) fn insert(&mut self, tuple: &KTuple) {
        self.add(tuple.terminals.inner());
    }

    /// Inserts a Terminals instance
    pub(crate) fn add(&mut self, terminals: &Terminals) {
        if terminals.is_empty() {
            return;
        }
        let Terminals { t, i } = terminals;
        let (start_root, mut changed) = self.add_child(t[0].0);
        let mut node = &mut self.root[start_root];
        for t in &t[1..*i] {
            let (child_index, inserted) = node.add_child(t.0);
            node = &mut node.children_mut()[child_index];
            changed |= inserted;
        }
        node.set_end();
        if changed {
            self.len += 1;
        }
    }

    /// Appends another Trie item to self
    /// Returns true if the insertion actually changed the trie
    pub fn append(&mut self, other: &Self) -> bool {
        let len = self.len();
        other.iter().for_each(|t| self.add(&t));
        len != self.len()
    }

    /// Creates a union with another KTuples and self
    pub fn union(&self, other: &Self) -> (Self, bool) {
        let mut trie = self.clone();
        let changed = trie.append(other);
        (trie, changed)
    }

    /// Creates a intersection with another Trie and self
    pub fn intersection(&self, other: &Self) -> Self {
        let s1 = self.iter().collect::<Vec<_>>();
        other
            .iter()
            .filter(|t2| s1.iter().any(|t1| t1 == t2))
            .fold(Trie::new(), |mut acc, t| {
                acc.add(&t);
                acc
            })
    }

    /// Checks if self and other are disjoint
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.intersection(other).is_empty()
    }

    /// Returns an iterator over the elements of this [`Trie`].
    pub(crate) fn iter(&self) -> TerminalsIter<'_> {
        TerminalsIter::new(self)
    }

    /// Creates an epsilon item, i.e. a set with exactly one epsilon k-tuple
    pub fn eps() -> Self {
        let mut trie = Trie::new();
        trie.add(&Terminals::eps());
        trie
    }

    /// Creates an end-of-input item, i.e. a set with exactly one end-of-input k-tuple
    pub fn end() -> Self {
        let mut trie = Trie::new();
        trie.add(&Terminals::end());
        trie
    }

    /// Returns the index of the given terminal is in the node's list of children if it exists
    fn child_index(&self, t: TerminalIndex) -> Option<usize> {
        self.root.c.binary_search(&Node::new(t)).ok()
    }

    /// Adds a child node if it not already exists and returns the child index of it.
    /// The boolean in the return value ist true on insertion, i.e. when the collection has changed.
    fn add_child(&mut self, t: TerminalIndex) -> (usize, bool) {
        if let Some(index) = self.child_index(t) {
            (index, false)
        } else {
            let idx = self.root.c.partition_point(|n| n.t < t);
            // insert in sort order
            self.root.c.insert(idx, Node::new(t));
            (idx, true)
        }
    }
}

impl Index<usize> for Trie {
    type Output = Node;

    fn index(&self, index: usize) -> &Self::Output {
        &self.root.c[index]
    }
}

impl Extend<Terminals> for Trie {
    fn extend<T: IntoIterator<Item = Terminals>>(&mut self, iter: T) {
        iter.into_iter().for_each(|t| self.add(&t))
    }
}

impl PartialEq for Trie {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.union(other).0.len() == self.len()
    }
}

impl Display for Trie {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for t in self.iter() {
            writeln!(f, "{t}")?
        }
        Ok(())
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Flags: u32 {
        const Default = 0;
        const EndNode = 0b1;
        const Iterated = 0b10;
    }
}

#[derive(Debug)]
pub(crate) struct TerminalsIter<'a> {
    // Stack with triples of traversed node, child index and node flags
    v: Vec<(&'a Node, usize, Flags)>,
}

impl<'a> TerminalsIter<'a> {
    pub(crate) fn new(t: &'a Trie) -> Self {
        let mut this = Self {
            v: Vec::with_capacity(MAX_K), // Depth of Tie can't exceed MAX_K
        };
        if !t.is_empty() {
            let flags = if t.root[0].e {
                Flags::EndNode
            } else {
                Flags::Default
            };
            this.v.push((&t.root, 0, flags));
            this.expand(&t.root, 0, flags);
        }
        this
    }

    // From the given node take child with index i and traverse in depth first order.
    // Push all nodes and their indices on the node stack.
    fn expand(&mut self, node: &'a Node, mut i: usize, flags: Flags) {
        let mut node = node;
        loop {
            if node.is_inner_end_node() && flags & Flags::Iterated == Flags::Default {
                // Stop on inner end nodes once
                break;
            }
            if node.children().len() <= i {
                break;
            }
            node = &node.children()[i];
            let flags = if node.e {
                Flags::EndNode
            } else {
                Flags::Default
            };
            self.v.push((node, 0, flags));
            i = 0;
        }
    }

    // Try to advance horizontally
    fn advance(&mut self) {
        if let Some((n, mut i, flags)) = self.v.pop() {
            i += 1;
            if n.children().len() > i {
                self.v.push((n, i, flags));
                self.expand(n, i, flags);
            } else {
                self.advance();
            }
        };
    }

    fn last_is_inner_node(&self) -> bool {
        if self.v.is_empty() {
            return false;
        }
        let (node, _, flags) = self.v.last().unwrap();
        *flags & (Flags::EndNode | Flags::Iterated) == Flags::EndNode && !node.c.is_empty()
    }
}

impl Iterator for TerminalsIter<'_> {
    type Item = Terminals;

    fn next(&mut self) -> Option<Self::Item> {
        if self.v.is_empty() {
            return None;
        }
        let result = Some(Terminals::from_slice_with(
            &self.v[1..],
            self.v.len(),
            |(n, _, _)| CompiledTerminal(n.terminal()),
        ));
        if self.last_is_inner_node() {
            // Set the iterated flag
            let (node, i, flags) = self.v.pop().unwrap();
            let flags = flags | Flags::Iterated;
            self.v.push((node, i, flags));
            self.expand(node, i, flags);
        } else {
            self.advance();
        }
        result
    }
}

impl Display for TerminalsIter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Stack: [{}]",
            self.v
                .iter()
                .map(|e| format!("({}, i{})", e.0, e.1))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod test {
    use parol_runtime::lexer::EOI;

    use crate::{
        analysis::{
            compiled_terminal::{EPS, INVALID},
            k_tuple::Terminals,
            terminals_trie::{Node, Trie},
        },
        CompiledTerminal, KTuple,
    };

    #[test]
    fn node_new() {
        let n = Node::new(42);
        assert_eq!(n.t, 42);
        assert!(n.c.is_empty());
        assert!(n.terminal() != INVALID);
    }

    #[test]
    fn node_default() {
        let n = Node::default();
        assert_eq!(n.t, INVALID);
        assert!(n.terminal() == INVALID);
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
        assert!(n.children().iter().find(|n| n.t == 7).is_some());
    }

    #[test]
    fn trie_new() {
        let t = Trie::new();
        assert!(t.root.children().is_empty());
        assert!(t.is_empty());
    }

    #[test]
    fn trie_eps() {
        let t = Trie::eps();
        assert_eq!(1, t.len());
        assert_eq!(t[0].t, EPS);
    }

    #[test]
    fn trie_end() {
        let t = Trie::end();
        assert_eq!(1, t.len());
        assert_eq!(t[0].t, EOI);
    }

    fn end_node_count(trie: &Trie) -> usize {
        fn recurse_for_cnt(node: &Node) -> usize {
            let cnt = if node.e { 1 } else { 0 };
            node.c.iter().fold(cnt, |mut acc, node| {
                acc += recurse_for_cnt(node);
                acc
            })
        }
        trie.root.c.iter().fold(0, |mut acc, node| {
            acc += recurse_for_cnt(node);
            acc
        })
    }

    #[test]
    fn trie_insert() {
        let mut t = Trie::new();
        let tuple1 = KTuple::new(5).with_terminal_indices(&[1, 2, 3]);
        t.insert(&tuple1);

        assert_eq!(1, t.len());
        assert_eq!(1, end_node_count(&t));

        assert!(!t.root.children().is_empty());
        assert!(t.root.children().iter().find(|n| n.t == 1).is_some());
        assert_eq!(t.root.children().len(), 1);

        assert!(t[0].children().iter().find(|n| n.t == 2).is_some());
        assert_eq!(t[0].children().len(), 1);
        assert_eq!(t[0][0].t, 2);

        assert!(t[0][0].children().iter().find(|n| n.t == 3).is_some());
        assert_eq!(t[0][0].children().len(), 1);
        assert_eq!(t[0][0][0].t, 3);
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

        assert_eq!(1, t.len());
        assert_eq!(1, end_node_count(&t));

        assert!(!t.root.children().is_empty());
        assert!(t.root.children().iter().find(|n| n.t == 1).is_some());
        assert_eq!(t.root.children().len(), 1);

        assert!(t[0].children().iter().find(|n| n.t == 2).is_some());
        assert_eq!(t[0].children().len(), 1);
        assert_eq!(t[0][0].t, 2);

        assert!(t[0][0].children().iter().find(|n| n.t == 3).is_some());
        assert_eq!(t[0][0].children().len(), 1);
        assert_eq!(t[0][0][0].t, 3);
    }

    #[test]
    fn trie_multiple_inserts_single_root() {
        let mut t = Trie::new();
        let tuple1 = KTuple::new(6).with_terminal_indices(&[1, 2, 3]);
        let tuple2 = KTuple::new(6).with_terminal_indices(&[1, 5, 6]);
        //      1
        //     / \
        //    2   5
        //    |   |
        //    3   6
        t.insert(&tuple1);
        t.insert(&tuple2);

        assert_eq!(2, t.len());
        assert_eq!(2, end_node_count(&t));

        assert!(!t.root.children().is_empty());
        assert!(t.root.children().iter().find(|n| n.t == 1).is_some());
        assert_eq!(t.root.children().len(), 1);

        assert!(t[0].children().iter().find(|n| n.t == 2).is_some());
        assert!(t[0].children().iter().find(|n| n.t == 5).is_some());
        assert_eq!(t[0].children().len(), 2);
        assert_eq!(t[0][0].t, 2);
        assert_eq!(t[0][1].t, 5);

        assert!(t[0][0].children().iter().find(|n| n.t == 3).is_some());
        assert!(t[0][1].children().iter().find(|n| n.t == 6).is_some());
        assert_eq!(t[0][0].children().len(), 1);
        assert_eq!(t[0][1].children().len(), 1);
        assert_eq!(t[0][0][0].t, 3);
        assert_eq!(t[0][1][0].t, 6);
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

        assert_eq!(2, t.len());
        assert_eq!(2, end_node_count(&t));

        assert!(!t.root.children().is_empty());
        assert!(t.root.children().iter().find(|n| n.t == 1).is_some());
        assert!(t.root.children().iter().find(|n| n.t == 4).is_some());
        assert_eq!(t.root.children().len(), 2);

        assert!(t[0].children().iter().find(|n| n.t == 2).is_some());
        assert!(t[1].children().iter().find(|n| n.t == 5).is_some());
        assert_eq!(t[0].children().len(), 1);
        assert_eq!(t[0][0].t, 2);
        assert_eq!(t[1][0].t, 5);

        assert!(t[0][0].children().iter().find(|n| n.t == 3).is_some());
        assert!(t[1][0].children().iter().find(|n| n.t == 6).is_some());
        assert_eq!(t[0][0].children().len(), 1);
        assert_eq!(t[1][0].children().len(), 1);
        assert_eq!(t[0][0][0].t, 3);
        assert_eq!(t[1][0][0].t, 6);
    }

    #[test]
    fn trie_empty_iter() {
        let t = Trie::new();
        assert_eq!(0, t.len());
        assert_eq!(0, end_node_count(&t));
        assert_eq!(0, t.iter().count());
        let expected = Vec::<Terminals>::new();
        assert_eq!(expected, t.iter().collect::<Vec<_>>());
    }

    #[test]
    fn trie_iter_single() {
        let mut t = Trie::new();
        t.insert(&KTuple::new(6).with_terminal_indices(&[1]));
        assert_eq!(1, t.len());
        assert_eq!(1, end_node_count(&t));

        let expected = vec![vec![1]]
            .iter()
            .map(|v| Terminals::from_slice_with(v, 6, |t| CompiledTerminal(*t)))
            .collect::<Vec<Terminals>>();

        assert_eq!(expected, t.iter().collect::<Vec<_>>());
    }

    #[test]
    fn trie_iter_single_plus() {
        let mut t = Trie::new();
        t.insert(&KTuple::new(6).with_terminal_indices(&[1]));
        t.insert(&KTuple::new(6).with_terminal_indices(&[1, 2]));
        assert_eq!(2, t.len());
        assert_eq!(2, end_node_count(&t));

        let expected = vec![vec![1], vec![1, 2]]
            .iter()
            .map(|v| Terminals::from_slice_with(v, 6, |t| CompiledTerminal(*t)))
            .collect::<Vec<Terminals>>();

        assert_eq!(expected, t.iter().collect::<Vec<_>>());
    }

    #[test]
    fn trie_iter() {
        let mut t = Trie::new();
        let tuple0 = KTuple::new(6).with_terminal_indices(&[1, 2]);
        let tuple1 = KTuple::new(6).with_terminal_indices(&[1, 2, 3]);
        let tuple2 = KTuple::new(6).with_terminal_indices(&[1, 2, 4]);
        let tuple3 = KTuple::new(6).with_terminal_indices(&[5, 6, 7]);
        let tuple4 = KTuple::new(6).with_terminal_indices(&[5, 8]);
        //     1     5
        //     |     | \
        //    (2)    6 (8)
        //     | \   |
        //    (3)(4)(7)
        t.insert(&tuple0);
        t.insert(&tuple1);
        t.insert(&tuple2);
        t.insert(&tuple3);
        t.insert(&tuple4);

        assert_eq!(t[0].t, 1);
        assert_eq!(t[0][0].t, 2);
        assert_eq!(t[0][0][0].t, 3);
        assert_eq!(t[0][0][1].t, 4);
        assert_eq!(t[1].t, 5);
        assert_eq!(t[1][0].t, 6);
        assert_eq!(t[1][0][0].t, 7);
        assert_eq!(t[1][1].t, 8);

        assert_eq!(5, t.len());
        assert_eq!(5, end_node_count(&t));

        let expected = vec![
            vec![1, 2],
            vec![1, 2, 3],
            vec![1, 2, 4],
            vec![5, 6, 7],
            vec![5, 8],
        ]
        .iter()
        .map(|v| Terminals::from_slice_with(v, 6, |t| CompiledTerminal(*t)))
        .collect::<Vec<Terminals>>();

        assert_eq!(expected, t.iter().collect::<Vec<_>>());
    }

    #[test]
    fn trie_intersection() {
        let mut t1 = Trie::new();
        let tuple1 = KTuple::new(6).with_terminal_indices(&[1, 2, 3]);
        let tuple2 = KTuple::new(6).with_terminal_indices(&[1, 2, 4]);
        let tuple3 = KTuple::new(6).with_terminal_indices(&[5, 6, 7]);
        let tuple4 = KTuple::new(6).with_terminal_indices(&[5, 8]);
        //     1     5
        //     |     | \
        //     2     6  8
        //     | \   |
        //     3  4  7
        t1.insert(&tuple1);
        t1.insert(&tuple2);
        t1.insert(&tuple3);
        t1.insert(&tuple4);

        let mut t2 = Trie::new();
        let tuple1 = KTuple::new(6).with_terminal_indices(&[1, 2, 3]);
        //     1
        //     |
        //     2
        //     |
        //     3
        t2.insert(&tuple1);

        let expected = vec![vec![1, 2, 3]]
            .iter()
            .map(|v| Terminals::from_slice_with(v, 6, |t| CompiledTerminal(*t)))
            .collect::<Vec<Terminals>>();

        assert_eq!(expected, t1.intersection(&t2).iter().collect::<Vec<_>>());
    }

    #[test]
    fn trie_is_disjoint_positive() {
        let mut t1 = Trie::new();
        let tuple1 = KTuple::new(6).with_terminal_indices(&[1, 2, 3]);
        let tuple2 = KTuple::new(6).with_terminal_indices(&[1, 2, 4]);
        t1.insert(&tuple1);
        t1.insert(&tuple2);
        let mut t2 = Trie::new();
        let tuple3 = KTuple::new(6).with_terminal_indices(&[5, 6, 7]);
        let tuple4 = KTuple::new(6).with_terminal_indices(&[5, 8]);
        t2.insert(&tuple3);
        t2.insert(&tuple4);
        //     t1    t2
        // ---------------
        //     1     5
        //     |     | \
        //     2     6  8
        //     | \   |
        //     3  4  7

        assert!(t1.is_disjoint(&t2));
        assert!(t2.is_disjoint(&t1));
    }

    #[test]
    fn trie_is_disjoint_negative() {
        let mut t1 = Trie::new();
        let tuple1 = KTuple::new(6).with_terminal_indices(&[1, 2, 3]);
        let tuple2 = KTuple::new(6).with_terminal_indices(&[1, 2, 4]);
        t1.insert(&tuple1);
        t1.insert(&tuple2);
        let mut t2 = Trie::new();
        let tuple3 = KTuple::new(6).with_terminal_indices(&[1, 2, 4]);
        let tuple4 = KTuple::new(6).with_terminal_indices(&[5, 8]);
        t2.insert(&tuple3);
        t2.insert(&tuple4);
        //     t1    t2
        // ---------------
        //     1     1  5
        //     |     |  |
        //     2     2  8
        //     | \   |
        //     3  4  4

        assert!(!t1.is_disjoint(&t2));
        assert!(!t2.is_disjoint(&t1));
    }

    #[test]
    fn trie_extend() {
        let mut t1 = Trie::new();
        let tuple1 = KTuple::new(6).with_terminal_indices(&[1, 2, 3]);
        let tuple2 = KTuple::new(6).with_terminal_indices(&[1, 2, 4]);
        t1.insert(&tuple1);
        t1.insert(&tuple2);
        let mut t2 = Trie::new();
        let tuple3 = KTuple::new(6).with_terminal_indices(&[5, 6, 7]);
        let tuple4 = KTuple::new(6).with_terminal_indices(&[5, 8]);
        t2.insert(&tuple3);
        t2.insert(&tuple4);
        //     t1    t2
        // ---------------
        //     1     5
        //     |     | \
        //     2     6  8
        //     | \   |
        //     3  4  7

        t1.extend(t2.iter());
        //        t1
        // ----------------
        //     1     5
        //     |     | \
        //     2     6  8
        //     | \   |
        //     3  4  7
        assert_eq!(4, t1.len());
        assert_eq!(4, end_node_count(&t1));

        let expected = vec![vec![1, 2, 3], vec![1, 2, 4], vec![5, 6, 7], vec![5, 8]]
            .iter()
            .map(|v| Terminals::from_slice_with(v, 6, |t| CompiledTerminal(*t)))
            .collect::<Vec<Terminals>>();

        assert_eq!(expected, t1.iter().collect::<Vec<_>>());
    }

    #[test]
    fn trie_eq_positive() {
        let mut t1 = Trie::new();
        let tuple1 = KTuple::new(6).with_terminal_indices(&[1, 2, 3]);
        let tuple2 = KTuple::new(6).with_terminal_indices(&[1, 2, 4]);
        t1.insert(&tuple1);
        t1.insert(&tuple2);
        let mut t2 = Trie::new();
        let tuple3 = KTuple::new(6).with_terminal_indices(&[1, 2, 3]);
        let tuple4 = KTuple::new(6).with_terminal_indices(&[1, 2, 4]);
        t2.insert(&tuple3);
        t2.insert(&tuple4);
        //     t1    t2
        // ---------------
        //     1     1
        //     |     |
        //     2     2
        //     | \   | \
        //     3  4  3  4

        assert_eq!(t1, t2);
    }

    #[test]
    fn trie_eq_negative() {
        let mut t1 = Trie::new();
        let tuple1 = KTuple::new(6).with_terminal_indices(&[1, 2, 3]);
        let tuple2 = KTuple::new(6).with_terminal_indices(&[1, 2, 4]);
        t1.insert(&tuple1);
        t1.insert(&tuple2);
        let mut t2 = Trie::new();
        let tuple3 = KTuple::new(6).with_terminal_indices(&[5, 6, 7]);
        let tuple4 = KTuple::new(6).with_terminal_indices(&[5, 8]);
        t2.insert(&tuple3);
        t2.insert(&tuple4);
        //     t1    t2
        // ---------------
        //     1     5
        //     |     | \
        //     2     6  8
        //     | \   |
        //     3  4  7

        assert_ne!(t1, t2);
    }

    // Trie::insert is commutative regarding Eq
    #[quickcheck]
    fn trie_insert_is_commutative_regarding_eq(t1: Vec<usize>, t2: Vec<usize>, k: usize) -> bool {
        let tuple1 = KTuple::new(k).with_terminal_indices(&t1);
        let tuple2 = KTuple::new(k).with_terminal_indices(&t2);
        // Insertion order 1, 2
        let mut t1 = Trie::new();
        t1.insert(&tuple1);
        t1.insert(&tuple2);

        // Insertion order 2, 1
        let mut t2 = Trie::new();
        t2.insert(&tuple2);
        t2.insert(&tuple1);

        t1 == t2
    }

    // Number of elements should be eq to number of the sum of inner and outer end nodes
    #[quickcheck]
    fn trie_item_count_equals_end_node_count(t1: Vec<Vec<usize>>, k: usize) -> bool {
        let trie = t1.iter().fold(Trie::new(), |mut acc, e| {
            acc.insert(&KTuple::new(k).with_terminal_indices(e));
            acc
        });

        let item_count = trie.iter().count();
        let end_node_count = end_node_count(&trie);
        // eprintln!(
        //     "{:?} => item_count: {item_count}, end_node_count: {end_node_count}",
        //     t1
        // );
        item_count == end_node_count
    }
}
