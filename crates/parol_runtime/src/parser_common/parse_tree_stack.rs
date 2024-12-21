use std::fmt::{Display, Error, Formatter};

use log::trace;

/// A stack for the parse tree nodes.
#[derive(Debug, Default)]
pub struct ParseTreeStack<T: Display> {
    /// The actual stack.
    pub stack: Vec<T>,
}

impl<T: Display> ParseTreeStack<T> {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Pushes a node onto the stack.
    pub fn push(&mut self, node: T) {
        trace!("ParseTreeStack: Pushing node: {}", node);
        self.stack.push(node);
    }

    /// Pops a node from the stack.
    pub fn pop(&mut self) -> Option<T> {
        let node = self.stack.pop();
        node.inspect(|n| {
            trace!("LRParseTreeStack: Popping node: {}", n);
        })
    }

    /// Returns the node at the top of the stack.
    pub fn last(&self) -> Option<&T> {
        self.stack.last()
    }

    /// Pops nodes from the stack where 'at' is the index of the first node to pop.
    pub fn split_off(&mut self, at: usize) -> Vec<T> {
        self.stack.split_off(at)
    }

    /// Pop n nodes from the stack and calculate n by applying the given function to each node.
    /// The function returns true if the node should be counted.
    /// Actually, this function is used to pop n nodes from the stack that are part of the parse
    /// tree plus additional nodes that are not part of the parse tree, such as skip tokens.
    pub fn pop_n(&mut self, n: usize, f: impl Fn(&T) -> bool) -> Vec<T> {
        // len is the split_off index
        let mut len = 0;
        // i is the number of nodes to pop
        let mut i = 0;
        while i < n && len < self.stack.len() {
            // Call the function for each node starting from the end and increment the number of
            // nodes to pop if the function returns true
            if f(&self.stack[self.stack.len() - 1 - len]) {
                // Increment the number of counted nodes
                i += 1;
            }
            // Increment the split_off index
            len += 1;
        }
        // Ensure we do not pop more nodes than available
        if len > self.stack.len() {
            len = self.stack.len();
        }
        // Pop the nodes from the stack at the calculated index
        self.split_off(self.stack.len() - len)
    }

    /// Returns the length of the stack.
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Returns true if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Pops all nodes from the stack.
    pub(crate) fn pop_all(&mut self) -> Vec<T> {
        // use mem::swap to avoid clone
        let mut stack = Vec::new();
        std::mem::swap(&mut stack, &mut self.stack);
        stack
    }
}

impl<T: Display> Display for ParseTreeStack<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.stack
            .iter()
            .rev()
            .enumerate()
            .try_for_each(|(i, e)| writeln!(f, "{} - {}", i, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_push_pop() {
        init();
        let mut stack = ParseTreeStack::new();
        stack.push("a");
        stack.push("b");
        stack.push("c");
        assert_eq!(stack.pop(), Some("c"));
        assert_eq!(stack.pop(), Some("b"));
        assert_eq!(stack.pop(), Some("a"));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_split_off() {
        init();
        let mut stack = ParseTreeStack::new();
        stack.push("a");
        stack.push("b");
        stack.push("c");
        stack.push("d");
        stack.push("e");
        let split = stack.split_off(2);
        assert_eq!(split, vec!["c", "d", "e"]);
        assert_eq!(stack.stack, vec!["a", "b"]);
    }

    #[test]
    fn test_pop_3() {
        init();
        let mut stack = ParseTreeStack::new();
        stack.push("a");
        stack.push("b");
        stack.push("c");
        stack.push("d");
        stack.push("e");
        let split = stack.pop_n(3, |_| true);
        assert_eq!(split, vec!["c", "d", "e"]);
        assert_eq!(stack.stack, vec!["a", "b"]);
    }

    #[test]
    fn test_pop_5() {
        init();
        let mut stack = ParseTreeStack::new();
        stack.push("a");
        stack.push("b");
        stack.push("c");
        stack.push("d");
        stack.push("e");
        let split = stack.pop_n(3, |_| false);
        assert_eq!(split, vec!["a", "b", "c", "d", "e"]);
        assert_eq!(stack.stack, Vec::<&str>::new());
    }

    #[test]
    fn test_pop_n() {
        init();
        let mut stack = ParseTreeStack::new();
        stack.push("a");
        stack.push("b");
        stack.push("c");
        stack.push("d");
        stack.push("e");
        // As a test function, we use a function that returns true for all nodes except "d"
        let split = stack.pop_n(3, |n| *n != "d");
        // Because the function returns false for "d", we should pop actually 4 nodes
        assert_eq!(split, vec!["b", "c", "d", "e"]);
        assert_eq!(stack.stack, vec!["a"]);
    }
}
