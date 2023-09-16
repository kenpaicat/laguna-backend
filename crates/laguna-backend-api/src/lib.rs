//! API handlers, states and errors.
#![doc(html_logo_url = "https://sloveniaengineering.github.io/laguna-backend/logo.png")]
#![doc(html_favicon_url = "https://sloveniaengineering.github.io/laguna-backend/favicon.ico")]
#![doc(issue_tracker_base_url = "https://github.com/SloveniaEngineering/laguna-backend")]
extern crate core;

pub mod error;
pub mod helpers;
pub mod login;
pub mod meta;
pub mod peer;
pub mod rating;
pub mod register;
pub mod stats;
pub mod torrent;
pub mod user;
