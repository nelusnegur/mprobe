/// Iterator adapters
pub(crate) trait IteratorExt {
    /// Creates a fallible iterator that uses a fallible clojure to determine
    /// if an elemnt should be yielded.
    #[inline]
    fn try_filter<T, E, P>(self, predicate: P) -> TryFilter<Self, P>
    where
        Self: Iterator<Item = Result<T, E>> + Sized,
        P: FnMut(&T) -> Result<bool, E>,
    {
        TryFilter::new(self, predicate)
    }

    /// Creates a fallible iterator that flattens nested structure
    /// propagating errors from both the inner and outer elements.
    #[inline]
    fn try_flatten<U, T, E>(self) -> TryFlatten<Self, U>
    where
        Self: Iterator<Item = Result<U, E>> + Sized,
        U: Iterator<Item = Result<T, E>>,
    {
        TryFlatten::new(self)
    }
}

impl<I: Iterator> IteratorExt for I {}

/// A fallible iterator that filters elements of `iter` with `predicate`.
///
/// This `struct` is created by the [`IteratorExt::try_filter`] method on [`IteratorExt`].
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
pub(crate) struct TryFilter<I, P> {
    iter: I,
    predicate: P,
}

impl<I, P> TryFilter<I, P> {
    pub(in crate::iter) fn new(iter: I, predicate: P) -> TryFilter<I, P> {
        TryFilter { iter, predicate }
    }
}

impl<I, P, T, E> Iterator for TryFilter<I, P>
where
    I: Iterator<Item = Result<T, E>>,
    P: FnMut(&T) -> Result<bool, E>,
{
    type Item = Result<T, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next()? {
                Ok(item) => match (self.predicate)(&item) {
                    Ok(true) => return Some(Ok(item)),
                    Ok(false) => continue,
                    Err(err) => return Some(Err(err)),
                },
                Err(err) => return Some(Err(err)),
            }
        }
    }
}

/// A fallible iterator that flattens one level of nesting of an iterator
/// propagating errors from both the inner and outer elements.
///
/// This `struct` is created by the [`IteratorExt::try_flatten`] method on [`IteratorExt`].
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
pub(crate) struct TryFlatten<I, U> {
    iter: I,
    inner_iter: Option<U>,
}

impl<I, U> TryFlatten<I, U> {
    pub(in crate::iter) fn new(iter: I) -> TryFlatten<I, U> {
        TryFlatten {
            iter,
            inner_iter: None,
        }
    }
}

impl<I, U, T, E> Iterator for TryFlatten<I, U>
where
    I: Iterator<Item = Result<U, E>>,
    U: Iterator<Item = Result<T, E>>,
{
    type Item = Result<T, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner_iter {
                Some(ref mut inner_iter) => return inner_iter.next(),
                None => match self.iter.next()? {
                    Ok(inner_iter) => self.inner_iter = Some(inner_iter),
                    Err(error) => return Some(Err(error)),
                },
            }
        }
    }
}
