use std::{fmt, ops};

#[derive(Debug, PartialEq, Clone)]
pub struct DisplayVec<T>(pub Vec<T>);

impl<T> ops::Deref for DisplayVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> fmt::Display for DisplayVec<T>
where
    T: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((last, rest)) = self.split_last() {
            for elem in rest {
                write!(f, "{}, ", elem)?;
            }
            write!(f, "{}", last)?;
        }
        Ok(())
    }
}

impl<T> From<Vec<T>> for DisplayVec<T> {
    fn from(v: Vec<T>) -> Self {
        DisplayVec(v)
    }
}

