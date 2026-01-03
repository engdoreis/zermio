// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

pub use crate::mmio::Device;
pub use crate::mmio::Interrupt;
pub use crate::mmio::Register;

pub use crate::rdljson;

#[derive(Debug)]
pub struct DeviceAddr {
    pub name: String,
    pub address: String,
}

#[derive(Debug)]
pub struct DeviceTypes {
    pub type_name: String,
    pub devices: Vec<DeviceAddr>,
}

#[derive(Debug)]
pub struct Platform {
    pub name: String,
    pub device_types: Vec<DeviceTypes>,
    pub interrupts: Vec<Interrupt>,
    //Define the number of data bit-width of the maximum single data transfer supported by the bus infrastructure
    pub bus_width: u32,
    pub devices: Vec<Device>,
}

impl Platform {
    pub fn add_device_addr(&mut self, type_name: String, device_name: String, address: u64) {
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

        for peripheral in &svd_device.peripherals {
            let svd_rs::peripheral::Peripheral::Single(info) = peripheral else {
                return Err("PeripheralInfo array not supported".to_string());
            };
            let device: Device = info.try_into()?;

            this.add_device_addr(device.type_.clone(), device.name.clone(), info.base_address);

            for interrupt in &info.interrupt {
                let mut interrupt: Interrupt = interrupt.into();
                interrupt.name = format!("{}_{}", device.name, interrupt.name);
                this.interrupts.push(interrupt);
            }
            // If it's empty, it's likely derived and should not generate a type.
            if !device.registers.is_empty() {
                this.devices.push(device);
            }
        }
        Ok(this)
    }
}

impl From<rdljson::SoC> for Platform {
    fn from(soc: rdljson::SoC) -> Self {
        let mut this = Self {
            name: soc.name,
            device_types: Vec::new(),
            interrupts: Vec::new(),
            bus_width: 32,
            devices: Vec::new(),
        };

        for peripheral in &soc.devices {
            let rdljson::Device::Device(peripheral) = peripheral else {
                continue;
            };
            let device: Device = peripheral.into();

            for (idx, addr) in peripheral.offsets.iter().enumerate() {
                let suffix = if peripheral.offsets.len() > 1 {
                    format!("{idx}")
                } else {
                    "".to_owned()
                };
                this.add_device_addr(
                    device.type_.clone(),
                    format!("{}{}", device.name, suffix),
                    *addr as u64,
                );
            }

            this.devices.push(device);
        }
        this
    }
}
