use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::render::error::RenderError;
use crate::render::OutputStream;

const FILE_NAME: &str = "index.html";

pub struct OutputFile {
    file: File,
}

impl OutputFile {
    pub fn new(path: &Path) -> Result<OutputFile, RenderError> {
        let path = path.join(FILE_NAME);
        let file = File::create(path)?;
        Ok(Self { file })
    }
}

impl OutputStream for OutputFile {
    fn write(&mut self, data: &str) -> Result<(), RenderError> {
        self.file.write_all(data.as_bytes())?;
        Ok(())
    }
}
