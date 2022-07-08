//! Iterator of options.
// Derived from https://github.com/lo48576/fbx-viewer/blob/develop/src/util/iter/option.rs

/// Iterator of options.
pub trait OptionIteratorExt: Iterator {
    /// Call `Option::map` for the elements.
    fn option_map<F, T, U>(self, f: F) -> Map<Self, F>
    where
        Self: Sized + Iterator<Item = Option<T>>,
        F: FnMut(T) -> U,
    {
        Map { iter: self, f }
    }

    /// Call `Option::and_then` for the elements.
    fn and_then<F, T, U>(self, f: F) -> AndThen<Self, F>
    where
        Self: Sized + Iterator<Item = Option<T>>,
        F: FnMut(T) -> U,
    {
        AndThen { iter: self, f }
    }

    /// Call `Option::ok_or` for the elements.
    fn ok_or<T, E>(self, e: E) -> OkOr<Self, E>
    where
        Self: Sized + Iterator<Item = Option<T>>,
        E: Clone,
    {
        OkOr { iter: self, e }
    }

    /// Call `Option::ok_or_else` for the elements.
    fn ok_or_else<F, T, E>(self, f: F) -> OkOrElse<Self, F>
    where
        Self: Sized + Iterator<Item = Option<T>>,
        F: FnMut() -> E,
    {
        OkOrElse { iter: self, f }
    }
}

impl<I, T> OptionIteratorExt for I where I: Iterator<Item = Option<T>> {}

/// Iterator calling `Option::map` for the elements.
#[derive(Debug, Clone, Copy)]
pub struct Map<I, F> {
    /// Iterator.
    iter: I,
    /// Function.
    f: F,
}

impl<I, F, T, U> Iterator for Map<I, F>
where
    I: Iterator<Item = Option<T>>,
    F: FnMut(T) -> U,
{
    type Item = Option<U>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| v.map(&mut self.f))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Iterator calling `Option::and_then` for the elements.
#[derive(Debug, Clone, Copy)]
pub struct AndThen<I, F> {
    /// Iterator.
    iter: I,
    /// Function.
    f: F,
}

impl<I, F, T, U> Iterator for AndThen<I, F>
where
    I: Iterator<Item = Option<T>>,
    F: FnMut(T) -> Option<U>,
{
    type Item = Option<U>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| v.and_then(&mut self.f))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Iterator calling `Option::ok_or` for the elements.
#[derive(Debug, Clone, Copy)]
pub struct OkOr<I, E> {
    /// Iterator.
    iter: I,
    /// Error value.
    e: E,
}

impl<I, E, T> Iterator for OkOr<I, E>
where
    I: Iterator<Item = Option<T>>,
    E: Clone,
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| v.ok_or_else(|| self.e.clone()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Iterator calling `Option::ok_or_else` for the elements.
#[derive(Debug, Clone, Copy)]
pub struct OkOrElse<I, F> {
    /// Iterator.
    iter: I,
    /// Function.
    f: F,
}

impl<I, F, T, E> Iterator for OkOrElse<I, F>
where
    I: Iterator<Item = Option<T>>,
    F: FnMut() -> E,
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| v.ok_or_else(&mut self.f))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
