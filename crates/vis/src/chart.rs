use serde::Serialize;

use crate::id::Id;

#[derive(Serialize, Debug)]
pub struct Chart {
    id: Id,
    name: String,
    series: Series,
}

impl Chart {
    pub fn new(id: Id, name: String, series: Series) -> Chart {
        Self { id, name, series }
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
        let xs = format!("xs_{}", id);
        let ys = format!("ys_{}", id);

        Self::new(xs, ys)
    }
}
