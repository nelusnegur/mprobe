use crate::layout::Chart;
use crate::layout::ElementKind;
use crate::layout::Section;
use crate::layout::View;
use crate::render::error::RenderError;
use crate::render::OutputStream;
use crate::render::Render;

impl Render for ElementKind {
    fn render<R>(&self, output: &mut R) -> Result<(), RenderError>
    where
        R: OutputStream,
    {
        match self {
            ElementKind::Section(s) => s.render(output),
            ElementKind::Chart(c) => c.render(output),
        }
    }
}

impl Render for View {
    fn render<R>(&self, output: &mut R) -> Result<(), RenderError>
    where
        R: OutputStream,
    {
        output.write("<html>")?;

        output.write("<head>")?;
        output.write("</head>")?;

        output.write("<body>")?;

        for element in &self.elements {
            element.render(output)?;
        }

        output.write("</body>")?;

        output.write("</html>")
    }
}

impl Render for Section {
    fn render<R>(&self, output: &mut R) -> Result<(), RenderError>
    where
        R: OutputStream,
    {
        output.write("<div>")?;

        for element in &self.elements {
            element.render(output)?;
        }

        output.write("</div>")
    }
}

impl Render for Chart {
    fn render<R>(&self, output: &mut R) -> Result<(), RenderError>
    where
        R: OutputStream,
    {
        // TODO: Render a chart
        output.write("a chart should be here")
    }
}
