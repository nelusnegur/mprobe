use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::chart::Chart;
use crate::error::Result;

struct Template {
    name: &'static str,
    text: &'static str,
}

// TODO: Render the index template
static INDEX_TEMPLATE: Template = Template {
    name: "index",
    text: include_str!("./template/index.html.tt"),
};

static CHARTS_TEMPLATE: Template = Template {
    name: "charts",
    text: include_str!("./template/charts.html.tt"),
};

static VIEW_TEMPLATE: Template = Template {
    name: "view",
    text: include_str!("./template/view.html.tt"),
};

struct View {
    groups: &'static [&'static str],
    file_name: &'static str,
}

static VIEWS: [View; 5] = [
    View {
        groups: &["serverStatus"],
        file_name: "server-status.html",
    },
    View {
        groups: &["replSetGetStatus"],
        file_name: "replset-status.html",
    },
    View {
        groups: &["serverStatus", "wiredTiger"],
        file_name: "wiredtiger.html",
    },
    View {
        groups: &["local.oplog.rs.stats"],
        file_name: "oplog.html",
    },
    View {
        groups: &["systemMetrics"],
        file_name: "system-metrics.html",
    },
];

impl View {
    pub fn select<'c, I>(&self, charts: I) -> impl Iterator<Item = &'c Chart>
    where
        I: Iterator<Item = &'c Chart>,
    {
        charts.filter(|c| {
            !self.groups.is_empty()
                && self
                    .groups
                    .iter()
                    .all(|g| c.groups.iter().any(|cg| cg == g))
        })
    }
}

pub struct TemplateEngine<'a> {
    index_path: &'a Path,
    views_path: &'a Path,
    templates: TinyTemplate<'static>,
}

impl<'a> TemplateEngine<'a> {
    pub fn new(index_path: &'a Path, views_path: &'a Path) -> TemplateEngine<'a> {
        let mut templates = TinyTemplate::new();

        templates
            .add_template(CHARTS_TEMPLATE.name, CHARTS_TEMPLATE.text)
            .expect("parse and compile the charts template");

        templates
            .add_template(VIEW_TEMPLATE.name, VIEW_TEMPLATE.text)
            .expect("parse and compile the view template");

        templates
            .add_template(INDEX_TEMPLATE.name, INDEX_TEMPLATE.text)
            .expect("parse and compile the index template");

        Self {
            index_path,
            views_path,
            templates,
        }
    }

    pub fn render(&self, charts: &[Chart]) -> Result<()> {
        if !self.views_path.exists() {
            fs::create_dir(self.views_path)?;
        }

        for view in VIEWS.iter() {
            let charts: Vec<&Chart> = view.select(charts.iter()).collect();
            let chart_context = ChartContext::new(charts);
            let charts = self
                .templates
                .render(CHARTS_TEMPLATE.name, &chart_context)?;

            let view_context = ViewContext::new(charts);
            let text = self.templates.render(VIEW_TEMPLATE.name, &view_context)?;
            self.create_file(view.file_name, &text)?;
        }

        Ok(())
    }

    fn create_file(&self, name: &str, text: &str) -> Result<()> {
        let path = self.views_path.join(name);
        let mut file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        file.write_all(text.as_bytes())?;
        file.flush()?;

        Ok(())
    }
}

#[derive(Serialize)]
struct ViewContext {
    view: String,
}

impl ViewContext {
    pub fn new(view: String) -> ViewContext {
        Self { view }
    }
}

#[derive(Serialize)]
struct ChartContext<'a> {
    charts: Vec<&'a Chart>,
}

impl<'a> ChartContext<'a> {
    pub fn new(charts: Vec<&'a Chart>) -> ChartContext<'a> {
        Self { charts }
    }
}
