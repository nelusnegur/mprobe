use serde::Serialize;

use crate::layout::id::Id;
use crate::layout::Element;

#[derive(Serialize, Debug)]
pub struct Chart {
    id: Id,
    pub name: String,
    pub spec: ChartSpec,
}

impl Chart {
    pub fn new() -> Chart {
        Self {
            id: Id::next(),
            name: String::new(),
            spec: ChartSpec::default(),
        }
    }
}

impl Element for Chart {
    fn id(&self) -> &Id {
        &self.id
    }
}

#[derive(Serialize, Debug, Default)]
pub struct ChartSpec {}
