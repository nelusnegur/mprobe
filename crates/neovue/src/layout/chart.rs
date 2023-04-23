use serde::Serialize;

use crate::layout::Element;

#[derive(Serialize, Debug)]
pub struct Chart {
    id: String,
    pub name: String,
    pub spec: ChartSpec,
}

impl Chart {
    pub fn new() -> Chart {
        Self {
            id: String::from("chart"),
            name: String::new(),
            spec: ChartSpec::default(),
        }
    }
}

impl Element for Chart {
    fn id(&self) -> &str {
        self.id.as_str()
    }
}

#[derive(Serialize, Debug, Default)]
pub struct ChartSpec {}
