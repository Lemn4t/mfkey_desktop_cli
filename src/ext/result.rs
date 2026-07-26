use std::error::Error;

pub type Rslt<T> = Result<T, Box<dyn Error>>;

pub trait ResultExt<T, E> {
    fn boxed(self) -> Result<T, Box<dyn Error>>
    where
        E: Error + Send + Sync + 'static;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn boxed(self) -> Result<T, Box<dyn Error>>
    where
        E: Error + Send + Sync + 'static,
    {
        self.map_err(Into::into)
    }
}
