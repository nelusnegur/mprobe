use serde::Serialize;

use crate::id::Id;


#[derive(Serialize, Debug)]
pub struct Chart {
    id: Id,
    xs: String,
    ys: String,
}

impl Chart {
   pub fn new(id: Id, xs: String, ys: String) -> Chart {
       Self { id, xs, ys }
   }
}
