// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Device {
    Mem(Memory),
    Device(Peripheral),
}

impl Device {
    fn get_offset(&self) -> u32 {
        match self {
            Device::Mem(mem) => mem.offset,
            Device::Device(dev) => *dev.offsets.first().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub name: String,
    pub entries: u32,
    pub sw_writable: bool,
    pub sw_readable: bool,
    pub width: u8,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peripheral {
    pub name: String,
    pub type_name: String,
    pub offsets: Vec<u32>,
    pub size: u32,
    pub interfaces: Vec<Interface>,
}

impl From<&Interface> for Peripheral {
    fn from(interface: &Interface) -> Self {
        Self {
            name: interface.name.clone().unwrap_or(String::from("device")),
            type_name: String::from(""),
            interfaces: vec![interface.clone()],
            size: 0,
            offsets: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    pub name: Option<String>,
    pub regs: Vec<Register>,
}

impl PartialEq for Interface {
    fn eq(&self, other: &Self) -> bool {
        self.regs == other.regs
    }
}
impl Eq for Interface {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Register {
    pub name: String,
    pub type_name: String,
    pub desc: Option<String>,
    pub offsets: Vec<u32>,
    #[serde(default)]
    pub fields: Vec<RegisterField>,
    pub sw_writable: bool,
    pub sw_readable: bool,
    pub reset: u32,
}

// Exclude the offsets
impl PartialEq for Register {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
            && self.fields.eq(&other.fields)
            && self.sw_writable.eq(&other.sw_writable)
            && self.sw_readable.eq(&other.sw_readable)
            && self.reset.eq(&other.reset)
    }
}

impl Eq for Register {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterField {
    pub name: String,
    pub desc: String,
    pub lsb: u32,
    pub msb: u32,
    #[serde(rename = "enum")]
    pub enum_field: Option<String>,
    pub sw_writable: bool,
    pub sw_readable: bool,
    pub set_onread: bool,
    pub clear_onread: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoC {
    pub name: String,
    pub devices: Vec<Device>,
}

impl SoC {
    pub fn try_from(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// If homogeneous interfaces are found, transform them into separate devices/pheripherals.
    pub fn homogeneous_interfaces_to_periperals(&mut self) {
        let mut new_devices: Vec<Device> = vec![];
        for device in self.devices.iter_mut() {
            let Device::Device(device) = device else {
                continue;
            };
            if device.interfaces.len() <= 1 {
                continue;
            }
            let mut iter = device.interfaces.iter();
            let first = iter.next().unwrap();
            if iter.all(|elem| elem.eq(first)) {
                let base = device.offsets[0];
                let type_name = &device.type_name;
                let size = first.regs.last().unwrap().offsets.last().unwrap() + 4u32;
                // let num_new_devices = device.interfaces.len() - 1;
                // device.offsets.extend((1..=num_new_devices).map(|elem| base + (size * elem as u32)));
                new_devices.extend(device.interfaces.iter().enumerate().skip(1).map(
                    |(idx, interface)| {
                        let mut device: Peripheral = interface.into();
                        device.type_name = type_name.clone();
                        device.size = size;
                        device.offsets = vec![size * idx as u32 + base];
                        Device::Device(device)
                    },
                ));
                device.interfaces.truncate(1);
            }
        }
        self.devices.extend(new_devices);
        self.devices.sort_by_key(|a| a.get_offset());
    }
}
