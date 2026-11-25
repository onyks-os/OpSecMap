// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Command-line interface: parsing, execution and text rendering.
//!
//! Parsing turns a line into a [`command::Command`], the [`session::Session`]
//! applies it to a map, and [`render`] turns evaluation results into lines of
//! text. Only [`storage`] touches files and only [`password`] reads the
//! terminal; the input loop is in `main.rs`.

pub(crate) mod command;
pub(crate) mod password;
pub(crate) mod render;
pub(crate) mod session;
pub(crate) mod storage;
