// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Default, strum::Display, strum::IntoStaticStr)]
pub enum Permissions {
    Read,
    Write,
    #[default]
    ReadWrite,
    ReadWriteOnce,
    WriteOnce,
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

impl From<svd_rs::access::Access> for Permissions {
    fn from(s: svd_rs::access::Access) -> Self {
        match s {
            svd_rs::access::Access::ReadOnly => Permissions::Read,
            svd_rs::access::Access::ReadWrite => Permissions::ReadWrite,
            svd_rs::access::Access::ReadWriteOnce => Permissions::ReadWriteOnce,
            svd_rs::access::Access::WriteOnce => Permissions::WriteOnce,
            svd_rs::access::Access::WriteOnly => Permissions::Write,
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

pub struct Register {
    pub name: String,
    pub desc: String,
    pub offset: String,
    pub bitfields: Vec<Bitfield>,
}

impl Register {
    pub fn new(name: String, offset: u32, desc: Option<String>, bitfields: Vec<Bitfield>) -> Self {
        Self {
            name: name.clone(),
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

impl Bitfield {
    fn new(
        name: String,
        desc: String,
        bit_size: u32,
        offset: u32,
        permissions: Permissions,
    ) -> Self {
        Self {
            name,
            offset,
            bit_size,
            permissions,
            desc,
        }
    }
}

impl Default for Bitfield {
    fn default() -> Self {
        Self::new(
            "value".to_string(),
            "value".to_string(),
            32,
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
            Permissions::from(field.access.clone().unwrap_or_default()),
        )
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

pub struct Platform {
    pub device_types: Vec<DeviceTypes>,
    pub interrupts: Vec<Interrupt>,
}

impl Platform {
    pub fn new() -> Self {
        Self {
            device_types: Vec::new(),
            interrupts: Vec::new(),
        }
    }

    pub fn add_register(&mut self, type_name: String, device_name: String, address: u64) {
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

    pub fn add_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupts.push(interrupt);
    }
}

impl Default for Platform {
    fn default() -> Self {
        Self::new()
    }
}
