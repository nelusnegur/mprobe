//! [mprobe]'s visualization library.
//!
//! [mprobe]: https://github.com/nelusnegur/mprobe
//!
//! **WARNING**: This library is the mprobe's visualization internal library and
//! there are no plans to stabilize it. The API may break at any time without notice.

#![warn(missing_docs)]

pub(crate) mod chart;
pub(crate) mod id;
pub(crate) mod template;

pub mod error;
pub mod layout;
