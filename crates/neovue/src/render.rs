pub(crate) mod error;

use crate::render::error::RenderError;

pub trait OutuptStream {
    fn write(&mut self, data: &str) -> Result<(), RenderError>;
}

pub trait Render {
    fn render<R>(&self, output: &mut R) -> Result<(), RenderError>
    where
        R: OutuptStream;
}
