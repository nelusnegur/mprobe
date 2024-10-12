use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::chart::Chart;
use crate::chart::Series;
use crate::id::Id;

const FILE_NAME: &str = "index.html";

pub fn create_html_file(path: &Path) -> Result<(), std::io::Error> {
    let mut template = TinyTemplate::new();
    template.add_template("index", include_str!("./html/index.html.tt")).unwrap();

    let context = create_context();
    let text = template.render("index", &context).expect("Couldn't render");

    let path = path.join(FILE_NAME);
    let mut file = File::create(path)?;

    file.write_all(text.as_bytes()).unwrap();
    file.flush().unwrap();

    Ok(())
}

#[derive(Serialize)]
struct Context {
    charts: Vec<Chart>,
}

fn create_context() -> Context {
    let charts = vec![
        Chart::new(Id::next(), Series::new(String::from("xs1"), String::from("ys1"))),
        Chart::new(Id::next(), Series::new(String::from("xs2"), String::from("ys2"))),
        Chart::new(Id::next(), Series::new(String::from("xs3"), String::from("ys3"))),
    ];

    Context { charts }
}
