// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Enable the `doc_notable_trait` feature, which allows certain traits to be
// designated as notable in documentation, thus bringing the reader's attention
// to them above others.
//
// This is used in traits that provide core functionality for a type where that
// type would not be "complete" without it. We use it for [`Request`] and
// [`Reply`], for example.
#![feature(doc_notable_trait)]

//! # X Rust Bindings
//! X Rust Bindings is a Rust library directly implementing the types and protocol messages of the
//! [X11 protocol specification](https://x.org/releases/X11R7.7/doc/xproto/xprotocol.html/). XRB is
//! _not_ a high-level API library, and it does not provide a direct connection to an X server, nor
//! does it do anything else on its own. XRB's development purpose is to provide a foundation for
//! higher-level Rust API wrapper libraries. It is used by [X.RS](https://crates.io/crates/xrs),
//! the official accompanying API library for XRB.

/// The major version of the X protocol used in XRB.
///
/// The X protocol major version may increment if breaking changes are introduced; seeing as this
/// has not happened since the 80s, it's probably safe to assume it won't.
pub const PROTOCOL_MAJOR_VERSION: u16 = 11;
/// The minor version of the X protocol used in XRB.
///
/// The X protocol minor version may increment if non-breaking features are added to the X
/// protocol; seeing as this has not happened since the 80s, it's probably safe to assume it won't.
pub const PROTOCOL_MINOR_VERSION: u16 = 0;

/// Implementations for the core X11 protocol.
mod x11;

pub use x11::traits::{Reply, Request};
pub use x11::*;

pub mod queries {}
pub mod events {}

pub mod replies {}
