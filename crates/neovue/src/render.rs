pub mod error;
pub mod output;
pub mod page;

use crate::render::error::RenderError;

pub trait OutputStream {
    fn write(&mut self, data: &str) -> Result<(), RenderError>;
}

pub trait Render {
    fn render<O>(&self, output: &mut O) -> Result<(), RenderError>
    where
        O: OutputStream;
}
