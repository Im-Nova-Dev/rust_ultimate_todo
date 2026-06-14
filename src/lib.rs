//! Deep Todo — hierarchical task TUI library.
//!
//! The binary (`main.rs`) is a thin entry point; all logic lives here.

pub mod app;
pub mod date;
pub mod event;
pub mod keys;
pub mod log;
pub mod model;
pub mod persist;
pub mod sort;
pub mod terminal;
pub mod theme;
pub mod tree_walk;
pub mod ui;

pub use app::App;
pub use model::{Priority, Status, Task};
pub use persist::{LoadedProject, TDL_FORMAT, TDL_VERSION, TdlFile};
pub use sort::SortMode;
