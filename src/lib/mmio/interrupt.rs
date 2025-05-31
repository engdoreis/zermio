// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone)]
pub struct Interrupt {
    pub name: String,
    pub description: String,
    pub value: u32,
}

impl From<&svd_rs::Interrupt> for Interrupt {
    fn from(interrupt: &svd_rs::Interrupt) -> Self {
        Self {
            name: interrupt.name.clone(),
            description: interrupt
                .description
                .clone()
                .unwrap_or(interrupt.name.clone()),
            value: interrupt.value,
        }
    }
}
