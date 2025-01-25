use std::fmt::{Display, Error, Formatter};

pub(crate) struct StrIter<T: Iterator<Item = U>, U> {
    iter: T,
}

impl<T, U> StrIter<T, U>
where
    T: Iterator<Item = U>,
    U: Display,
{
    pub fn new(iter: T) -> Self {
        Self { iter }
    }
}

pub(crate) trait IteratorExt<U: Display>: Iterator<Item = U> + Sized {
    fn into_str_iter(self) -> StrIter<Self, U>;
}

impl<T: Iterator<Item = U> + Clone, U: Display> IteratorExt<U> for T {
    fn into_str_iter(self) -> StrIter<Self, U> {
        StrIter::new(self)
    }
}

impl<T, U> Display for StrIter<T, U>
where
    T: Iterator<Item = U> + Clone,
    U: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let iter = self.iter.clone();
        for item in iter {
            f.write_fmt(format_args!("{}", item))?;
        }
        Ok(())
    }
}
