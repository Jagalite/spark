#![forbid(unsafe_code)]
//! Six explicit-time designs: stored configuration and borrowed ordinary Widgets.
//!
//! ```
//! use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
//! use spark_catalog::SegmentedComet;
//! use spark_runtime::{ClipTime, Palette, Profile, SampleContext};
//!
//! let comet = SegmentedComet::new().with_seed(7); // Store once in host state.
//! let palette = Palette::default();
//! let profile = Profile::default();
//! let context = SampleContext::new(ClipTime::from_nanos(400_000_000), &palette, &profile);
//! let sample = comet.sample(&context); // Cheap view; no clock or advancing state.
//! let area = Rect::new(2, 1, 24, 3);
//! let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 5));
//! sample.render(area, &mut buffer);
//! ```
pub mod comet;
pub use comet::{CometSample, SegmentedComet};
pub mod border_runner;
pub mod prism_reveal;
pub use border_runner::{BorderRunner, BorderSample};
pub use prism_reveal::{PrismReveal, RevealError, RevealSample};
pub mod assembly_progress;
pub mod inputs;
pub mod telemetry_embers;
pub use assembly_progress::{AssemblyProgress, AssemblySample};
pub use inputs::{InputError, Level, Progress};
pub use telemetry_embers::{EmbersSample, TelemetryEmbers};
pub mod pocket_mascot;
pub use pocket_mascot::{MascotSample, PocketMascot};

/// Optional pure, caller-fed host-data adapters.
pub mod adapters;
