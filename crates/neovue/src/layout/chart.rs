use crate::layout::id::Id;
use crate::layout::Element;

use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Chart {
    id: Id,
    pub name: String,
    pub spec: ChartSpec,
}

impl Chart {
    pub fn new() -> Chart {
        Self {
            id: Id::next(),
            name: String::new(),
            spec: ChartSpec::new(),
        }
    }
}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Chart {
    fn id(&self) -> &Id {
        &self.id
    }
}

#[derive(Serialize, Debug, Default)]
pub struct ChartSpec {
    pub trace: Trace,
    pub layout: Layout,
}

impl ChartSpec {
    pub fn new() -> Self {
        Self {
            trace: Trace::new(),
            layout: Layout::new(),
        }
    }
}

#[derive(Serialize, Debug, Default)]
pub struct Trace {
    #[serde(rename = "type")]
    kind: TraceKind,
    mode: Mode,
    fill: Fill,
}

impl Trace {
    pub fn new() -> Self {
        Self {
            kind: TraceKind::Scatter,
            mode: Mode::Lines,
            fill: Fill::Tozeroy,
        }
    }
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum TraceKind {
    #[default]
    Scatter,
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Lines,
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Fill {
    #[default]
    Tozeroy,
}

#[derive(Serialize, Debug, Default)]
pub struct Layout {
    #[serde(rename = "xaxis")]
    x_axis: Axis,
    #[serde(rename = "yaxis")]
    y_axis: Axis,
    width: u32,
    height: u32,
    #[serde(rename = "showlegend")]
    show_legend: bool,
    #[serde(rename = "autosize")]
    auto_size: bool,
    #[serde(rename = "hovermode")]
    hover_mode: HoverMode,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            x_axis: Axis::new(),
            y_axis: Axis::new(),
            width: 800,
            height: 400,
            show_legend: false,
            auto_size: false,
            hover_mode: HoverMode::Closest,
        }
    }
}

#[derive(Serialize, Debug, Default)]
pub struct Axis {
    #[serde(rename = "type")]
    kind: AxisKind,
    mirror: bool,
    #[serde(rename = "showgrid")]
    show_grid: bool,
    #[serde(rename = "showline")]
    show_line: bool,
    #[serde(rename = "zeroline")]
    zero_line: bool,
    #[serde(rename = "autorange")]
    auto_range: bool,
    #[serde(rename = "showticklabels")]
    show_tick_labels: bool,
}

impl Axis {
    pub fn new() -> Self {
        Self {
            kind: AxisKind::Linear,
            mirror: false,
            show_grid: false,
            show_line: false,
            zero_line: false,
            auto_range: true,
            show_tick_labels: false,
        }
    }
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum AxisKind {
    #[default]
    Linear,
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum HoverMode {
    #[default]
    Closest,
}
