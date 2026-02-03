// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

pub use crate::rdljson;

#[derive(Debug)]
pub struct Windows {
    pub name: String,
    pub type_name: String,
    pub offset: u32,
    pub desc: String,
    pub width: u32,
    pub entries: u32,
}

impl Windows {
    pub fn new(
        name: impl Into<String>,
        type_name: Option<String>,
        offset: u32,
        desc: Option<String>,
        width: u32,
        entries: u32,
    ) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            type_name: type_name.unwrap_or(name.clone().replace("%s", "")),
            desc: desc.unwrap_or(name),
            offset,
            width,
            entries,
        }
    }

    pub fn is_readable(&self) -> bool {
        true
    }

    pub fn is_writable(&self) -> bool {
        true
    }
}

impl From<&rdljson::Windows> for Windows {
    fn from(windows: &rdljson::Windows) -> Self {
        Self::new(
            windows.name.clone(),
            None,
            windows.offset,
            windows.desc.clone(),
            windows.width,
            windows.entries,
        )
    }
}
