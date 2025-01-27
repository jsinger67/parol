use std::{
    cell::RefCell,
    fmt::{Display, Error, Formatter},
};

pub(crate) struct StrIter<T: Iterator<Item = U>, U> {
    iter: RefCell<Option<T>>,
}

impl<T, U> StrIter<T, U>
where
    T: Iterator<Item = U>,
    U: Display,
{
    pub fn new(iter: T) -> Self {
        Self {
            iter: RefCell::new(Some(iter)),
        }
    }
}

pub(crate) trait IteratorExt<U: Display>: Iterator<Item = U> + Sized {
    fn into_str_iter(self) -> StrIter<Self, U>;
}

impl<T: Iterator<Item = U>, U: Display> IteratorExt<U> for T {
    fn into_str_iter(self) -> StrIter<Self, U> {
        StrIter::new(self)
    }
}

impl<T, U> Display for StrIter<T, U>
where
    T: Iterator<Item = U>,
    U: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let iter = self
            .iter
            .borrow_mut()
            .take()
            .expect("StrIter cannot be displayed more than once");
        for item in iter {
            f.write_fmt(format_args!("{}", item))?;
        }
        Ok(())
    }
}
