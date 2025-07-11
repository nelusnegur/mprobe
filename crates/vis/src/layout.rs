//! Defines the structure and the layout of the data used for
//! visualizing diagnostic metrics.

mod data;
mod iter;
mod series;

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use mprobe_diagnostics::DiagnosticData;

use crate::error::Result;
use crate::layout::data::DataEngine;
use crate::layout::iter::ErrorHandlingIter;
use crate::template::TemplateEngine;

/// Coordinates the creation of the data visualization.
///
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
/// The __index__ file represents the entry point into the data visualization.
/// The __data__ directory contains the chart data.
/// The __view__ directory contains the chart visualizations.
pub struct VisLayout {
    index_file_path: PathBuf,
    views_path: PathBuf,
    data_path: PathBuf,
}

impl VisLayout {
    const MAIN_DIR_NAME: &str = "vis";
    const DATA_DIR_NAME: &str = "data";
    const VIEWS_DIR_NAME: &str = "views";
    const INDEX_FILE_NAME: &str = "index.html";

    /// Initializes a directory for the data visualization and
    /// creates a new instance of [VisLayout].
    pub fn init(path: &Path) -> Result<VisLayout> {
        let root_path = path.join(Self::MAIN_DIR_NAME);
        let index_file_path = root_path.join(Self::INDEX_FILE_NAME);
        let data_path = root_path.join(Self::DATA_DIR_NAME);
        let views_path = root_path.join(Self::VIEWS_DIR_NAME);

        if root_path.exists() {
            fs::remove_dir_all(&root_path)?;
        }
        fs::create_dir(&root_path)?;

        Ok(Self {
            data_path,
            index_file_path,
            views_path,
        })
    }

    /// Generates a visualization report based on the provided diagnostic data.
    pub fn generate_report(&self, diagnostic_data: DiagnosticData) -> Result<()> {
        let mut data_engine = DataEngine::new(&self.data_path);
        let metrics = ErrorHandlingIter::new(diagnostic_data.into_iter());
        let charts = data_engine.render(metrics)?;

        let template = TemplateEngine::new(&self.index_file_path, &self.views_path);
        template.render(&charts)
    }
}
