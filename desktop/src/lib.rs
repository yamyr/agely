//! Native Agely desktop application and walkthrough rules.

pub const PRODUCT_NAME: &str = "Agely";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod agent;
pub mod i18n;
pub mod live;
pub mod model;

#[cfg(feature = "native-ui")]
pub mod input;

#[cfg(feature = "native-ui")]
pub mod theme;

#[cfg(feature = "native-ui")]
pub mod widgets;

#[cfg(feature = "native-ui")]
pub mod app;

#[cfg(feature = "native-ui")]
mod screens;
