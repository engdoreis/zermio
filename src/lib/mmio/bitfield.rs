// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

pub use crate::mmio::Permissions;
pub use crate::rdljson;

#[derive(Debug)]
pub struct Bitfield {
    pub name: String,
    pub desc: String,
    pub bit_size: u32,
    pub offset: u32,
    pub permissions: Permissions,
}

impl Bitfield {
    fn new(
        name: impl Into<String>,
        desc: impl Into<String>,
        bit_size: u32,
        offset: u32,
        permissions: Permissions,
    ) -> Self {
        Self {
            name: name.into(),
            desc: desc.into(),
            offset,
            bit_size,
            permissions,
        }
    }
}

impl Default for Bitfield {
    fn default() -> Self {
        Self::new(
            "value".to_string(),
            "value".to_string(),
            super::WIDTH,
            0,
            Permissions::default(),
        )
    }
}

impl From<&svd_rs::field::FieldInfo> for Bitfield {
    fn from(field: &svd_rs::field::FieldInfo) -> Self {
        Self::new(
            field.name.clone(),
            field.description.clone().unwrap_or(field.name.clone()),
            field.bit_range.width,
            field.bit_range.offset,
            Permissions::from(field.access.unwrap_or_default()),
        )
    }
}

impl TryFrom<&svd_rs::field::Field> for Bitfield {
    type Error = String;
    fn try_from(field: &svd_rs::field::Field) -> Result<Self, Self::Error> {
        Ok(Self::from(match field {
            svd_rs::field::Field::Single(info) => info,
            svd_rs::field::Field::Array(_, _) => {
                return Err("Field Array not supported".to_string());
            }
        }))
    }
}

impl From<&rdljson::RegisterField> for Bitfield {
    fn from(field: &rdljson::RegisterField) -> Self {
        Self::new(
            field.name.clone(),
            field.desc.clone(),
            field.msb - field.lsb + 1,
            field.lsb,
            Permissions::from(field),
        )
    }
}
