mod layout;
mod render;

// # Layout
//
// Layout module provides a description of the page structure.
// It defines all elements of a page. That includes:
//  - text
//  - collapsable sections
//  - charts
//  - links to other pages?
//
// ## Example
//
// <page>
//   <text>...</text>
//   <chart>...</chart>
// </page>
//
// struct Page {
//    elements: Vec<RenderableItem>, // maybe use a slice, since we know the size
//    // ... other page properties
// }
//
// let page = Page {
//    elements: vec![
//
//    ],
//    // ...
// };
//
//
// # Rendering
//
// The rendering module determines how a page layout is displayed.
// It generates all the necessary files that are used by a rendering engine
// to display the page with its elements.
// The rendering module provides an interface used to render a page layout
// that is interpreted by a given rendering engine.
//
// render
//      plotly
//      ...
