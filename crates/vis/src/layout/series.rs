use std::fs;
use std::path::Path;

use mprobe_diagnostics::error::MetricsDecoderError;
use mprobe_diagnostics::metrics::MetricsChunk;

pub struct SeriesGenerator<'a> {
    path: &'a Path,
}

impl<'a> SeriesGenerator<'a> {
    pub fn new(path: &'a Path) -> SeriesGenerator {
        Self { path }
    }

    pub fn write<I>(&self, metrics: I) -> Result<(), std::io::Error>
    where
        I: Iterator<Item = Result<MetricsChunk, MetricsDecoderError>>,
    {
        if !self.path.exists() {
            fs::create_dir(self.path)?;
        }

        todo!()
    }
}
