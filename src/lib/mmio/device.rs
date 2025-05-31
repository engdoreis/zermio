// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

pub use crate::mmio::Register;

pub struct Device {
    pub name: String,
    pub type_: String,
    pub registers: Vec<Register>,
}

impl Device {
    pub fn new(name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_: type_name.into(),
            registers: vec![],
        }
    }
}
