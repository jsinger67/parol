use std::{
    fmt::{Debug, Display, Error, Formatter},
    ops::Index,
};

use parol_runtime::NonTerminalIndex;

#[derive(Debug, Default, Clone)]
pub struct StrVec {
    vec: Vec<String>,
    indent: String,
    indent_first_line: bool,
}

impl Display for StrVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for (i, s) in self.vec.iter().enumerate() {
            let indent = if i == 0 && !self.indent_first_line {
                ""
            } else {
                &self.indent
            };
            writeln!(f, "{indent}{s}")?
        }
        Ok(())
    }
}

#[allow(dead_code)]
impl StrVec {
    pub fn new(indent: usize) -> Self {
        Self {
            vec: Vec::new(),
            indent: Self::build_indent(indent),
            indent_first_line: true,
        }
    }
    pub fn first_line_no_indent(mut self) -> Self {
        self.indent_first_line = false;
        self
    }
    pub fn push(&mut self, s: String) {
        self.vec.push(s);
    }
    pub fn max_content_len(&self) -> usize {
        self.vec
            .iter()
            .fold(0usize, |acc, e| std::cmp::max(acc, e.len()))
    }
    pub fn len(&self) -> usize {
        self.vec.len()
    }
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
    pub fn iter(&self) -> StrVecIterator<'_> {
        StrVecIterator { vec: self, pos: 0 }
    }
    fn build_indent(amount: usize) -> String {
        let space = " ".to_string();
        space.repeat(amount)
    }

    pub fn pop(&mut self) -> Option<String> {
        self.vec.pop()
    }

    pub(crate) fn join(&self, arg: &str) -> String {
        self.vec.join(arg)
    }
}

pub struct StrVecIterator<'a> {
    vec: &'a StrVec,
    pos: usize,
}

impl<'a> Iterator for StrVecIterator<'a> {
    type Item = &'a String;
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {
        let current_pos = self.pos;
        if self.vec.len() > current_pos {
            self.pos += 1;
            Some(&self.vec.vec[current_pos])
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a StrVec {
    type Item = &'a String;
    type IntoIter = StrVecIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Index<NonTerminalIndex> for StrVec {
    type Output = String;
    fn index(&self, index: NonTerminalIndex) -> &Self::Output {
        &self.vec[index]
    }
}

impl FromIterator<String> for StrVec {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self {
            vec: iter.into_iter().collect(),
            indent: Self::build_indent(0),
            indent_first_line: true,
        }
    }
}
