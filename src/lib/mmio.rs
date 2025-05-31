// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

static WIDTH: u32 = 32;
pub fn device_type(s: &str) -> String {
    let re = regex::Regex::new(r"\d+$").unwrap();
    // Remove trailing digits
    re.replace(s, "").to_string().to_lowercase()
}

pub fn device_cluster_name(base: &str, index: &str, reg: &str) -> String {
    let name = base.to_string().replace("%s", "");
    let reg = reg.to_string().replace(&name, "");
    format!("{}{}{}", name, index, &reg)
}

#[derive(Debug, Clone, Default, strum::Display, strum::IntoStaticStr)]
pub enum Permissions {
    #[default]
    ReadWrite,
    ReadWriteOnce,
    Read,
    Write,
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

pub struct RegisterInfo {
    pub name: String,
    pub type_: String,
    pub desc: String,
    pub offset: u32,
}

impl RegisterInfo {
    pub fn new(
        name: impl Into<String>,
        type_name: Option<String>,
        desc: Option<String>,
        offset: u32,
    ) -> Self {
        let name = name.into();
        RegisterInfo {
            name: name.clone(),
            type_: type_name.unwrap_or(name.clone()),
            desc: desc.unwrap_or(name),
            offset: offset,
        }
    }
}

pub struct Register {
    pub info: Vec<RegisterInfo>, // Must have at least one.
    pub bitfields: Vec<Bitfield>,
}

impl Register {
    pub fn new(
        name: impl Into<String>,
        offset: u32,
        desc: Option<String>,
        bitfields: Vec<Bitfield>,
    ) -> Self {
        Self {
            info: vec![RegisterInfo::new(name, None, desc, offset)],
            bitfields,
        }
    }

    fn try_from(cluster: &svd_rs::cluster::Cluster) -> Result<Vec<Self>, String> {
        match cluster {
            svd_rs::cluster::Cluster::Array(info, dim) => {
                let mut res = Vec::new();
                let index = dim.dim_index.clone().unwrap_or(
                    (0..info.children.len())
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>(),
                );
                for children in info.children.iter() {
                    let mut indexes = index.iter();
                    match children {
                        svd_rs::registercluster::RegisterCluster::Register(register) => {
                            let mut register: Register = register.try_into()?;
                            let index = indexes.next().unwrap();
                            let type_name = dim
                                .dim_name
                                .clone()
                                .unwrap_or(register.info[0].name.clone());
                            register.info[0].name =
                                device_cluster_name(&info.name, &index, &type_name);
                            let mut offset = dim.dim_increment + register.info[0].offset;
                            for index in indexes {
                                let name = device_cluster_name(&info.name, &index, &type_name);
                                register.info.push(RegisterInfo::new(
                                    name,
                                    Some(type_name.clone()),
                                    None,
                                    offset,
                                ));
                                offset += dim.dim_increment;
                            }
                            res.push(register);
                        }
                        svd_rs::registercluster::RegisterCluster::Cluster(_) => {
                            panic!("Too much recursion")
                        }
                    }
                }
                return Ok(res);
            }
            svd_rs::cluster::Cluster::Single(_) => unreachable!(),
        }
    }
}

impl TryFrom<&svd_rs::register::Register> for Register {
    type Error = String;
    fn try_from(register: &svd_rs::register::Register) -> Result<Self, Self::Error> {
        let register = match register {
            svd_rs::register::Register::Single(info) => info,
            svd_rs::register::Register::Array(_, _) => {
                return Err("Register Array not supported".to_string());
            }
        };

        let bitfields = if let Some(ref bitfields) = register.fields {
            bitfields
                .iter()
                .map(|field| {
                    field
                        .try_into()
                        .unwrap_or_else(|e| panic!("{} register {}", e, register.name))
                })
                .collect::<Vec<Bitfield>>()
        } else {
            vec![Bitfield::default()]
        };

        Ok(Self::new(
            register.name.clone(),
            register.address_offset as u32,
            register.description.clone(),
            bitfields,
        ))
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
            WIDTH,
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

// Platform specific
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
    pub name: String,
    pub device_types: Vec<DeviceTypes>,
    pub interrupts: Vec<Interrupt>,
    //Define the number of data bit-width of the maximum single data transfer supported by the bus infrastructure
    pub bus_width: u32,
    pub devices: Vec<Device>,
}

impl Platform {
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
}

impl TryFrom<svd_rs::device::Device> for Platform {
    type Error = String;
    fn try_from(svd_device: svd_rs::device::Device) -> Result<Self, Self::Error> {
        let mut this = Self {
            name: svd_device.name,
            device_types: Vec::new(),
            interrupts: Vec::new(),
            bus_width: svd_device.width,
            devices: Vec::new(),
        };

        for periph in &svd_device.peripherals {
            let device_iter = match periph {
                svd_rs::peripheral::Peripheral::Single(info) => info,
                svd_rs::peripheral::Peripheral::Array(_, _) => {
                    return Err("Peripheral array not supported".to_string());
                }
            };

            // i.e 0x8000_0000
            let device_addr = device_iter.base_address;
            // i.e UART0
            let device_name = device_iter.name.replace(" ", "_").to_uppercase();
            // i.e UART
            let device_type =
                device_type(device_iter.derived_from.as_ref().unwrap_or(&device_name));

            this.add_register(device_type.clone(), device_name.clone(), device_addr);

            for interrupt in &device_iter.interrupt {
                let mut interrupt: Interrupt = interrupt.into();
                interrupt.name = format!("{}_{}", device_name, interrupt.name);
                this.interrupts.push(interrupt);
            }

            let Some(ref registers) = device_iter.registers else {
                continue;
            };

            let mut device = Device::new(&device_name, &device_type);
            for register_cluster in registers {
                match register_cluster {
                    svd_rs::registercluster::RegisterCluster::Register(register) => {
                        match register.try_into() {
                            Ok(register) => device.registers.push(register),
                            Err(msg) => println!("Warning: {} in {}, skipping.", msg, device_name),
                        }
                    }
                    svd_rs::registercluster::RegisterCluster::Cluster(cluster) => {
                        match Register::try_from(cluster) {
                            Ok(registers) => {
                                for reg in registers {
                                    device.registers.push(reg);
                                }
                            }
                            Err(msg) => println!("Warning: {} in {}, skipping.", msg, device_name),
                        }
                    }
                };
            }
            this.devices.push(device);
        }
        Ok(this)
    }
}
