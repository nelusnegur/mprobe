use serde_json;

use crate::layout::chart::Chart;
use crate::layout::section::Section;
use crate::layout::view::View;
use crate::layout::Element;
use crate::layout::ElementKind;
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
        output.write("<!DOCTYPE html>")?;
        output.write(r#"<html lang="en">"#)?;
        output.write(
            r#"<head>
                 <meta charset="utf-8" />
                 <script src="https://cdn.plot.ly/plotly-2.20.0.min.js" charset="utf-8"></script>
               </head>
            "#,
        )?;
        output.write("<body>")?;

        for element in &self.elements {
            element.render(output)?;
        }

        // TODO: Write all chart files here

        output.write("</body>")?;
        output.write("</html>")
    }
}

impl Render for Section {
    fn render<R>(&self, output: &mut R) -> Result<(), RenderError>
    where
        R: OutputStream,
    {
        let id = self.id();
        output.write(&format!(r#"<div id="{id}">"#))?;

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
        let id = self.id();
        let trace = serde_json::to_string(&self.spec.trace)?;
        let layout = serde_json::to_string(&self.spec.layout)?;

        output.write(&format!(
            r#"
                <div>
                    <div id="{id}"></div>
                    <script>
                        let trace = {trace};
                        trace.x = xs{id};
                        trace.y = ys{id};

                        const chart = document.getElementById("{id}");
                        Plotly.newPlot(chart, [trace], {layout});
                    </script>
                </div>
            "#,
        ))
    }
}
