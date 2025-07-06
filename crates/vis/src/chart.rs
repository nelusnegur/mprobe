pub(crate) mod axes;

use std::path::Path;
use std::sync::Arc;

use serde::Serialize;

use crate::chart::axes::AxisType;
use crate::id::Id;

#[derive(Serialize, Debug)]
pub struct Chart {
    id: Id,
    name: Arc<str>,
    pub(crate) groups: Vec<String>,
    yaxis_type: AxisType,
    series: Arc<Series>,
    series_path: Arc<Path>,
}

impl Chart {
    pub fn new(
        id: Id,
        name: Arc<str>,
        groups: Vec<String>,
        yaxis_type: AxisType,
        series: Arc<Series>,
        series_path: Arc<Path>,
    ) -> Chart {
        Self {
            id,
            name,
            groups,
            yaxis_type,
            series,
            series_path,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Series {
    pub(crate) xs: String,
    pub(crate) ys: String,
}

impl Series {
    pub fn new(xs: String, ys: String) -> Series {
        Self { xs, ys }
    }

    pub fn from(id: Id) -> Series {
        let xs = format!("xs_{id}");
        let ys = format!("ys_{id}");

        Self::new(xs, ys)
    }
}
