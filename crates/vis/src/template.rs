use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::chart::Chart;
use crate::error::Result;

// TODO: Render the index template
static INDEX_TEMPLATE: TemplateInfo = TemplateInfo {
    name: "index",
    groups: &[],
    file_name: "index.html",
    text: include_str!("./template/index.html.tt"),
};

static TEMPLATES: [TemplateInfo; 5] = [
    TemplateInfo {
        name: "server-status",
        groups: &["serverStatus"],
        file_name: "server-status.html",
        text: include_str!("./template/server-status.html.tt"),
    },
    TemplateInfo {
        name: "replset-status",
        file_name: "replset-status.html",
        groups: &["replSetGetStatus"],
        text: include_str!("./template/replset-status.html.tt"),
    },
    TemplateInfo {
        name: "wiredtiger",
        file_name: "wiredtiger.html",
        groups: &["serverStatus", "wiredTiger"],
        text: include_str!("./template/wiredtiger.html.tt"),
    },
    TemplateInfo {
        name: "oplog",
        groups: &["local.oplog.rs.stats"],
        file_name: "oplog.html",
        text: include_str!("./template/oplog.html.tt"),
    },
    TemplateInfo {
        name: "system-metrics",
        groups: &["systemMetrics"],
        file_name: "system-metrics.html",
        text: include_str!("./template/system-metrics.html.tt"),
    },
];

struct TemplateInfo {
    name: &'static str,
    text: &'static str,
    groups: &'static [&'static str],
    file_name: &'static str,
}

pub struct Template<'a> {
    index_path: &'a Path,
    views_path: &'a Path,
}

impl<'a> Template<'a> {
    pub fn new(index_path: &'a Path, views_path: &'a Path) -> Template<'a> {
        Self {
            index_path,
            views_path,
        }
    }

    pub fn render(&self, charts: &[Chart]) -> Result<()> {
        let mut template = TinyTemplate::new();

        if !self.views_path.exists() {
            fs::create_dir(self.views_path)?;
        }

        for item in TEMPLATES.iter() {
            template.add_template(item.name, item.text)?;

            let charts: Vec<&Chart> = charts
                .iter()
                .filter(|c| {
                    !item.groups.is_empty()
                        && item
                            .groups
                            .iter()
                            .all(|g| c.groups.iter().any(|cg| cg == g))
                })
                .collect();

            let context = Context::new(charts);
            let text = template.render(item.name, &context)?;
            self.create_file(item.file_name, &text)?;
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
struct Context<'a> {
    charts: Vec<&'a Chart>,
}

impl<'a> Context<'a> {
    pub fn new(charts: Vec<&'a Chart>) -> Context<'a> {
        Self { charts }
    }
}
