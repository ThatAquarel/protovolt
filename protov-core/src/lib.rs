//! Host-testable core logic for ProtoV (config, model, SCPI, app, protection).
//!
//! Future EEPROM persistence will load/save a [`config::FactorySettings`]-shaped snapshot;
//! runtime overrides remain in [`app::AppCore`] and [`scpi::state::ScpiState`].

#![cfg_attr(not(test), no_std)]

pub mod app;
pub mod config;
pub mod dfu;
pub mod fmt;
pub mod model;
pub mod pd;
pub mod protection;
pub mod scpi;
