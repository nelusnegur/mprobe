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

// Elements

pub trait Element {
    fn id(self) -> u64;
}

pub enum ElementKind {
    Section(Section),
    Chart(Chart),
}

impl Element for ElementKind {
    fn id(self) -> u64 {
        match self {
            ElementKind::Section(s) => s.id,
            ElementKind::Chart(c) => c.id,
        }
    }
}

pub struct Chart {
    id: u64,
    pub name: String,
}

impl Element for Chart {
    fn id(self) -> u64 {
        self.id
    }
}

pub struct Section {
    id: u64,
    pub elements: Vec<ElementKind>,
}

impl Element for Section {
    fn id(self) -> u64 {
        self.id
    }
}
