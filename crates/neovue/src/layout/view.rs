use crate::layout::ElementKind;

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
