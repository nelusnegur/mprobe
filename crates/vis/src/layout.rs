mod iter;
mod series;
mod writer;

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use mprobe_diagnostics::DiagnosticData;

use crate::chart::Chart;
use crate::chart::Series;
use crate::id::Id;
use crate::layout::iter::ErrorHandlingIter;
use crate::layout::series::SeriesGen;
use crate::template::Context;
use crate::template::Template;

const MAIN_DIR_NAME: &str = "vis";
const SERIES_DIR_NAME: &str = "series";
const INDEX_FILE_NAME: &str = "index.html";

/// The data visualization directory is structured as follows:
///
/// ./vis/index.html
/// ./vis/series/series1.js
/// ./vis/series/series2.js
/// ./vis/series/...
/// ./vis/series/seriesN.js
///
/// The __index__ file represents the entry point into the visualization.
/// The __series__ directory contains the chart series.
pub struct VisLayout {
    root_path: PathBuf,
    index_file_path: PathBuf,
    series_path: PathBuf,
}

impl VisLayout {
    pub fn init(path: &Path) -> Result<VisLayout, std::io::Error> {
        let root_path = path.join(MAIN_DIR_NAME);
        let index_file_path = root_path.join(INDEX_FILE_NAME);
        let series_path = root_path.join(SERIES_DIR_NAME);

        fs::create_dir(&root_path)?;

        Ok(Self {
            root_path,
            series_path,
            index_file_path,
        })
    }

    pub fn generate_report(&self, diagnostic_data: DiagnosticData) -> Result<(), std::io::Error> {
        let mut series_gen = SeriesGen::new(&self.series_path);
        let iter = ErrorHandlingIter::new(diagnostic_data.into_iter());
        series_gen.write(iter)?;

        let template = Template::new(&self.index_file_path);
        let context = create_context();

        template.render(&context)
    }
}

// TODO: This is used temporarily for testing. Remove later
fn create_context() -> Context {
    let charts = vec![
        Chart::new(
            Id::next(),
            String::from("chart 1"),
            Series::new(String::from("xs1"), String::from("ys1")),
        ),
        Chart::new(
            Id::next(),
            String::from("chart 2"),
            Series::new(String::from("xs2"), String::from("ys2")),
        ),
        Chart::new(
            Id::next(),
            String::from("chart 3"),
            Series::new(String::from("xs3"), String::from("ys3")),
        ),
    ];

    Context::new(charts)
}
