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
}

impl<I: Iterator> IteratorExt for I {}

/// An fallible iterator that filters elements of `iter` with `predicate`.
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
