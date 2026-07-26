use std::error::Error;
use std::fmt;

pub type Rslt<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct ContextError {
    context: String,
    source: Box<dyn Error + Send + Sync>,
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.context, self.source)
    }
}

impl Error for ContextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

pub trait ResultExt<T, E> {
    fn boxed(self) -> Rslt<T>
    where
        E: Error + Send + Sync + 'static;

    fn context(self, msg: impl Into<String>) -> Rslt<T>
    where
        E: Error + Send + Sync + 'static;

    fn with_context<F>(self, f: F) -> Rslt<T>
    where
        E: Error + Send + Sync + 'static,
        F: FnOnce() -> String;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn boxed(self) -> Rslt<T>
    where
        E: Error + Send + Sync + 'static,
    {
        self.map_err(Into::into)
    }

    fn context(self, msg: impl Into<String>) -> Rslt<T>
    where
        E: Error + Send + Sync + 'static,
    {
        self.map_err(|e| {
            Box::new(ContextError {
                context: msg.into(),
                source: Box::new(e),
            }) as Box<dyn Error>
        })
    }

    fn with_context<F>(self, f: F) -> Rslt<T>
    where
        E: Error + Send + Sync + 'static,
        F: FnOnce() -> String,
    {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Box::new(ContextError {
                context: f(),
                source: Box::new(e),
            })),
        }
    }
}
