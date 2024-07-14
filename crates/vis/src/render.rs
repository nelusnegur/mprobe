mod chart;
pub mod error;
pub mod output;
pub mod view;

use crate::render::error::RenderError;

pub trait OutputStream {
    fn write(&mut self, data: &str) -> Result<(), RenderError>;
}

pub trait Render {
    fn render<O>(&self, output: &mut O) -> Result<(), RenderError>
    where
        O: OutputStream;
}

pub trait DataWriter {
    fn start(&mut self) -> Result<(), std::io::Error>;
    fn write(&mut self, data: DataItem) -> Result<(), std::io::Error>;
    fn end(self) -> Result<(), std::io::Error>;
}

#[derive(Debug)]
pub struct DataItem {
    x: f64,
    y: f64,
}

impl DataItem {
    pub fn new(x: f64, y: f64) -> DataItem {
        Self { x, y }
    }
}
