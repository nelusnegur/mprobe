mod iter;
mod data;
mod series;

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use mprobe_diagnostics::DiagnosticData;

use crate::error::Result;
use crate::layout::iter::ErrorHandlingIter;
use crate::layout::data::DataEngine;
use crate::template::TemplateEngine;

/// The data visualization directory is structured as follows:
///
/// ./vis/index.html
///
/// ./vis/views/view1.html
/// ./vis/views/view2.html
/// ./vis/views/...
/// ./vis/views/viewN.html
///
/// ./vis/data/data1.js
/// ./vis/data/data2.js
/// ./vis/data/...
/// ./vis/data/dataN.js
///
/// The __index__ file represents the entry point into the visualization.
/// The __data__ directory contains the chart data.
/// The __view__ directory contains the chart visualizations.
pub struct VisLayout {
    root_path: PathBuf,
    index_file_path: PathBuf,
    views_path: PathBuf,
    data_path: PathBuf,
}

impl VisLayout {
    const MAIN_DIR_NAME: &str = "vis";
    const DATA_DIR_NAME: &str = "data";
    const VIEWS_DIR_NAME: &str = "views";
    const INDEX_FILE_NAME: &str = "index.html";

    pub fn init(path: &Path) -> Result<VisLayout> {
        let root_path = path.join(Self::MAIN_DIR_NAME);
        let index_file_path = root_path.join(Self::INDEX_FILE_NAME);
        let data_path = root_path.join(Self::DATA_DIR_NAME);
        let views_path = root_path.join(Self::VIEWS_DIR_NAME);

        fs::create_dir(&root_path)?;

        Ok(Self {
            root_path,
            data_path,
            index_file_path,
            views_path,
        })
    }

    pub fn generate_report(&self, diagnostic_data: DiagnosticData) -> Result<()> {
        let mut data_engine = DataEngine::new(&self.data_path);
        let metrics = ErrorHandlingIter::new(diagnostic_data.into_iter());
        let charts = data_engine.render(metrics)?;

        let template = TemplateEngine::new(&self.index_file_path, &self.views_path);
        template.render(&charts)
    }
}
