pub mod chart;
mod id;

use crate::layout::chart::Chart;
use crate::layout::id::Id;

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

#[derive(Debug)]
pub struct View {
    pub elements: Vec<ElementKind>,
}

impl View {
    pub fn new() -> View {
        View {
            elements: Vec::new(),
        }
    }

    pub fn add(mut self, element: ElementKind) -> View {
        self.elements.push(element);
        self
    }
}

#[derive(Debug)]
pub struct Section {
    id: Id,
    pub elements: Vec<ElementKind>,
}

impl Section {
    pub fn new() -> Section {
        Self {
            id: Id::next(),
            elements: Vec::new(),
        }
    }
}

impl Element for Section {
    fn id(&self) -> &Id {
        &self.id
    }
}
