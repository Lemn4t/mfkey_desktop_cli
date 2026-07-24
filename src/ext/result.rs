use crate::ext::printing::PrintExt;
use std::error::Error;
use std::fmt::Display;

pub type Rslt<T> = Result<T, Box<dyn Error>>;

pub trait ResultExt<T, E> {
    fn boxed(self) -> Result<T, Box<dyn Error>>
    where
        E: Error + Send + Sync + 'static;
    fn print_err(self)
    where
        E: Display;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn boxed(self) -> Result<T, Box<dyn Error>>
    where
        E: Error + Send + Sync + 'static,
    {
        self.map_err(Into::into)
    }

    fn print_err(self)
    where
        E: Display,
    {
        if let Err(e) = self {
            format!("✗ {e}").eprintln()
        }
    }
}
