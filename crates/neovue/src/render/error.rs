use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum RenderError {}

impl Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for RenderError {}
