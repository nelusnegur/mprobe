pub mod chart;
pub mod section;
pub mod view;

mod id;

use crate::layout::chart::Chart;
use crate::layout::id::Id;
use crate::layout::section::Section;

pub trait Element {
    fn id(&self) -> &Id;
}

#[derive(Debug)]
pub enum ElementKind {
    Section(Section),
    Chart(Chart),
}

impl Element for ElementKind {
    fn id(&self) -> &Id {
        match self {
            ElementKind::Section(s) => s.id(),
            ElementKind::Chart(c) => c.id(),
        }
    }
}
