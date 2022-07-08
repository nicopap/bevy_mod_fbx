//! Iterator of results.
// Derived from https://github.com/lo48576/fbx-viewer/blob/develop/src/util/iter/result.rs

/// Iterator of results.
pub trait ResultIteratorExt: Iterator {
    /// Call `Result::map` for the elements.
    fn result_map<F, T, U, E>(self, f: F) -> Map<Self, F>
    where
        Self: Sized + Iterator<Item = Result<T, E>>,
        F: FnMut(T) -> U,
    {
        Map { iter: self, f }
    }

    /// Call `Result::and_then` for the elements.
    fn and_then<F, T, U, E>(self, f: F) -> AndThen<Self, F>
    where
        Self: Sized + Iterator<Item = Result<T, E>>,
        F: FnMut(T) -> U,
    {
        AndThen { iter: self, f }
    }
}

impl<I, T, E> ResultIteratorExt for I where I: Iterator<Item = Result<T, E>> {}

/// Iterator calling `Result::map` for the elements.
#[derive(Debug, Clone, Copy)]
pub struct Map<I, F> {
    /// Iterator.
    iter: I,
    /// Function.
    f: F,
}

impl<I, F, T, U, E> Iterator for Map<I, F>
where
    I: Iterator<Item = Result<T, E>>,
    F: FnMut(T) -> U,
{
    type Item = Result<U, E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| v.map(&mut self.f))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Iterator calling `Result::and_then` for the elements.
#[derive(Debug, Clone, Copy)]
pub struct AndThen<I, F> {
    /// Iterator.
    iter: I,
    /// Function.
    f: F,
}

impl<I, F, T, U, E> Iterator for AndThen<I, F>
where
    I: Iterator<Item = Result<T, E>>,
    F: FnMut(T) -> Result<U, E>,
{
    type Item = Result<U, E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| v.and_then(&mut self.f))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
