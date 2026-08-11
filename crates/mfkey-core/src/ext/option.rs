pub trait OptionExt<T> {
    fn or_err<E>(self, e: E) -> Result<T, E>;
}

impl<T> OptionExt<T> for Option<T> {
    fn or_err<E>(self, e: E) -> Result<T, E> {
        match self {
            Some(t) => Ok(t),
            None => Err(e),
        }
    }
}
