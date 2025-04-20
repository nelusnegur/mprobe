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
    name: &'static str,
    ingroup: &'static [&'static [&'static str]],
    exgroup: &'static [&'static [&'static str]],
    file_name: &'static str,
}

static VIEWS: [View; 6] = [
    View {
        name: "Server status",
        ingroup: &[&["serverStatus"]],
        exgroup: &[
            &["serverStatus", "wiredTiger"],
            &["serverStatus", "metrics"],
        ],
        file_name: "server-status.html",
    },
    View {
        name: "ReplicaSet status",
        ingroup: &[&["replSetGetStatus"]],
        exgroup: &[],
        file_name: "replset-status.html",
    },
    View {
        name: "WiredTiger",
        ingroup: &[&["serverStatus", "wiredTiger"]],
        exgroup: &[],
        file_name: "wiredtiger.html",
    },
    View {
        name: "Oplog",
        ingroup: &[&["local.oplog.rs.stats"]],
        exgroup: &[],
        file_name: "oplog.html",
    },
    View {
        name: "System metrics",
        ingroup: &[&["systemMetrics"]],
        exgroup: &[],
        file_name: "system-metrics.html",
    },
    View {
        name: "Server metrics",
        ingroup: &[&["serverStatus", "metrics"]],
        exgroup: &[],
        file_name: "server-metrics.html",
    },
];

impl View {
    pub fn select<'c, I>(&self, charts: I) -> impl Iterator<Item = &'c Chart>
    where
        I: Iterator<Item = &'c Chart>,
    {
        charts
            .filter(|c| self.ingroup.includes(&c.groups))
            .filter(|c| self.exgroup.excludes(&c.groups))
    }
}

trait Group {
    fn includes(&self, group: &[String]) -> bool;

    fn excludes(&self, group: &[String]) -> bool {
        !self.includes(group)
    }
}

impl Group for &[&[&str]] {
    fn includes(&self, group: &[String]) -> bool {
        !self.is_empty()
            && self
                .iter()
                .any(|g| !g.is_empty() && g.iter().all(|gg| group.iter().any(|xg| xg == gg)))
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

        self.render_index()?;

        for view in VIEWS.iter() {
            let charts: Vec<&Chart> = view.select(charts.iter()).collect();
            let chart_context = ChartContext { charts };
            let charts = self
                .templates
                .render(CHARTS_TEMPLATE.name, &chart_context)?;

            let view_context = ViewContext { view: charts };
            let text = self.templates.render(VIEW_TEMPLATE.name, &view_context)?;
            self.create_view(view.file_name, &text)?;
        }

        Ok(())
    }

    fn render_index(&self) -> Result<()> {
        let views: Vec<ViewItem> = VIEWS
            .iter()
            .map(|v| ViewItem {
                name: v.name,
                file_name: v.file_name,
            })
            .collect();

        let context = IndexContext { views };
        let text = self.templates.render(INDEX_TEMPLATE.name, &context)?;
        self.create_file(self.index_path, &text)?;

        Ok(())
    }

    fn create_view(&self, name: &str, text: &str) -> Result<()> {
        let path = self.views_path.join(name);
        self.create_file(&path, text)
    }

    fn create_file(&self, path: &Path, text: &str) -> Result<()> {
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

#[derive(Serialize)]
struct ChartContext<'a> {
    charts: Vec<&'a Chart>,
}

#[derive(Serialize)]
struct IndexContext {
    views: Vec<ViewItem>,
}

#[derive(Serialize)]
struct ViewItem {
    name: &'static str,
    file_name: &'static str,
}
