use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::chart::Chart;
use crate::chart::Series;
use crate::id::Id;

const MAIN_DIR_NAME: &str = "vis";
const SERIES_DIR_NAME: &str = "series";
const INDEX_FILE_NAME: &str = "index.html";

// The data visualization is structured as follows:
//
// ./vis/index.html
// ./vis/series/series1.js
// ./vis/series/series2.js
// ./vis/series/...
//
pub struct DataVis {
    root_path: PathBuf,
    index_file_path: PathBuf,
    series_path: PathBuf,
}

impl DataVis {
    pub fn init(path: &Path) -> Result<DataVis, std::io::Error> {
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

    pub fn generate_report(&self) -> Result<(), std::io::Error> {
        let mut template = TinyTemplate::new();
        template
            .add_template("index", include_str!("./html/index.html.tt"))
            .unwrap();

        let context = create_context();
        let text = template.render("index", &context).expect("Couldn't render");

        let mut file = File::create(&self.index_file_path)?;

        file.write_all(text.as_bytes()).unwrap();
        file.flush().unwrap();

        Ok(())
    }
}

#[derive(Serialize)]
struct Context {
    charts: Vec<Chart>,
}

fn create_context() -> Context {
    let charts = vec![
        Chart::new(
            Id::next(),
            Series::new(String::from("xs1"), String::from("ys1")),
        ),
        Chart::new(
            Id::next(),
            Series::new(String::from("xs2"), String::from("ys2")),
        ),
        Chart::new(
            Id::next(),
            Series::new(String::from("xs3"), String::from("ys3")),
        ),
    ];

    Context { charts }
}
