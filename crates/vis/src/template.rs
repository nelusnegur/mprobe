use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::chart::Chart;

pub struct Template<'a> {
    path: &'a Path,
}

impl<'a> Template<'a> {
    pub fn new(path: &'a Path) -> Template {
        Self { path }
    }

    pub fn render(&self, context: &Context) -> Result<(), std::io::Error> {
        let mut template = TinyTemplate::new();
        template
            .add_template("index", include_str!("./template/index.html.tt"))
            .unwrap();

        let text = template.render("index", &context).expect("Couldn't render");

        let mut file = File::create(self.path)?;

        file.write_all(text.as_bytes()).unwrap();
        file.flush().unwrap();

        Ok(())
    }
}

#[derive(Serialize)]
pub struct Context {
    charts: Vec<Chart>,
}

impl Context {
    pub fn new(charts: Vec<Chart>) -> Context {
        Self { charts }
    }
}
