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

    /// Returns the length of the stack.
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Returns true if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
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
