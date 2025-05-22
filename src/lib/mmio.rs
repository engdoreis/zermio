// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub clear: bool,
    pub execute: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            read: true,
            write: true,
            clear: false,
            execute: false,
        }
    }
}

impl From<&str> for Permissions {
    fn from(s: &str) -> Self {
        match &s[..] {
            "read-only" => Self {
                read: true,
                write: false,
                clear: false,
                execute: false,
            },
            "write-only" => Self {
                read: false,
                write: true,
                clear: true,
                execute: false,
            },
            "read-write" => Self {
                read: true,
                write: true,
                clear: true,
                execute: false,
            },
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
    pub bitfields: Vec<Bitfields<'a>>,
}

impl<'a> Register<'a> {
    pub fn new(name: &'a str, offset: u32, desc: Option<&'a str>) -> Self {
        Self {
            name,
            desc: desc.unwrap_or(name),
            offset: format!("{:#x}", offset),
            bitfields: vec![],
        }
    }
}

pub struct Bitfields<'a> {
    pub name: &'a str,
    pub desc: &'a str,
    pub bit_size: u32,
    pub offset: u32,
    pub permissions: Permissions,
}

impl<'a> Bitfields<'a> {
    pub fn new(
        name: &'a str,
        offset: u32,
        bit_size: u32,
        permissions: Permissions,
        desc: Option<&'a str>,
    ) -> Self {
        Self {
            name,
            offset,
            bit_size,
            permissions,
            desc: desc.unwrap_or(name),
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
                type_name: type_name,
                devices: vec![new_device],
            });
        }
    }
}
