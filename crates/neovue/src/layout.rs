pub trait Element {
    fn id(&self) -> u64;
}

#[derive(Debug)]
pub enum ElementKind {
    Section(Section),
    Chart(Chart),
}

impl Element for ElementKind {
    fn id(&self) -> u64 {
        match self {
            ElementKind::Section(s) => s.id,
            ElementKind::Chart(c) => c.id,
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
pub struct Chart {
    id: u64,
    pub name: String,
}

impl Chart {
    pub fn new() -> Chart {
        Self {
            id: 0,
            name: String::new(),
        }
    }
}

impl Element for Chart {
    fn id(&self) -> u64 {
        self.id
    }
}

#[derive(Debug)]
pub struct Section {
    id: u64,
    pub elements: Vec<ElementKind>,
}

impl Section {
    pub fn new() -> Section {
        Self {
            id: 0,
            elements: Vec::new(),
        }
    }
}

impl Element for Section {
    fn id(&self) -> u64 {
        self.id
    }
}
