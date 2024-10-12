use serde::Serialize;

use crate::id::Id;


#[derive(Serialize, Debug)]
pub struct Chart {
    id: Id,
    series: Series,
}

impl Chart {
   pub fn new(id: Id, series: Series) -> Chart {
       Self { id, series }
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
}
