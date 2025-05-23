// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use crate::schema;
#[derive(Debug, Clone, strum::Display, strum::IntoStaticStr)]
#[strum(serialize_all = "PascalCase")]
pub enum Permissions {
    Read,
    Write,
    ReadWrite,
}

impl From<&str> for Permissions {
    fn from(s: &str) -> Self {
        match s {
            "read-only" => Self::Read,
            "write-only" => Self::Write,
            "read-write" => Self::ReadWrite,
            _ => panic!("{} unsuported", s),
        }
    }
}

pub struct Device<'a> {
    pub name: &'a str,
    pub registers: Vec<&'a str>,
}

impl<'a> Device<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            registers: vec![],
        }
    }
}

pub struct Register<'a> {
    pub name: &'a str,
    pub desc: &'a str,
    pub offset: String,
    pub bitfields: Vec<Bitfield>,
}

impl<'a> Register<'a> {
    pub fn new(
        name: &'a str,
        offset: u32,
        desc: Option<&'a str>,
        bitfields: Vec<Bitfield>,
    ) -> Self {
        Self {
            name,
            desc: desc.unwrap_or(name),
            offset: format!("{:#x}", offset),
            bitfields,
        }
    }
}

pub struct Bitfield {
    pub name: String,
    pub desc: String,
    pub bit_size: u32,
    pub offset: u32,
    pub permissions: Permissions,
}

impl From<&schema::Field> for Bitfield {
    fn from(reg: &schema::Field) -> Self {
        Self {
            name: reg.name.clone(),
            offset: reg.bit_range.offset as u32,
            bit_size: reg.bit_range.size as u32,
            permissions: reg.access.clone(),
            desc: reg.description.clone(),
        }
    }
}

pub struct DeviceAddr {
    pub name: String,
    pub address: String,
}

pub struct DeviceTypes {
    pub type_name: String,
    pub devices: Vec<DeviceAddr>,
}

pub struct Platform {
    pub device_types: Vec<DeviceTypes>,
}

impl Platform {
    pub fn new() -> Self {
        Self {
            device_types: Vec::new(),
        }
    }

    pub fn add(&mut self, type_name: String, device_name: String, address: u64) {
        let new_device = DeviceAddr {
            name: device_name,
            address: format!("{:#x}", address),
        };
        if let Some(found) = self
            .device_types
            .iter_mut()
            .find(|elem| elem.type_name == type_name)
        {
            found.devices.push(new_device);
        } else {
            self.device_types.push(DeviceTypes {
                type_name,
                devices: vec![new_device],
            });
        }
    }
}

impl Default for Platform {
    fn default() -> Self {
        Self::new()
    }
}
