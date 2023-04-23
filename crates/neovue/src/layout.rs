pub mod chart;

use crate::layout::chart::Chart;

pub trait Element {
    fn id(&self) -> &str;
}

#[derive(Debug)]
pub enum ElementKind {
    Section(Section),
    Chart(Chart),
}

impl Element for ElementKind {
    fn id(&self) -> &str {
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
    id: String,
    pub elements: Vec<ElementKind>,
}

impl Section {
    pub fn new() -> Section {
        Self {
            id: String::new(),
            elements: Vec::new(),
        }
    }
}

impl Element for Section {
    fn id(&self) -> &str {
        self.id.as_str()
    }
}
