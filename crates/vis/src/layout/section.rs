use crate::layout::id::Id;
use crate::layout::Element;
use crate::layout::ElementKind;

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

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Section {
    fn id(&self) -> &Id {
        &self.id
    }
}
