// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

pub use crate::mmio::Device;
pub use crate::mmio::Interrupt;
pub use crate::mmio::Register;

use svd_rs::device;
use svd_rs::peripheral;
use svd_rs::registercluster;

pub struct DeviceAddr {
    pub name: String,
    pub address: String,
}

pub struct DeviceTypes {
    pub type_name: String,
    pub devices: Vec<DeviceAddr>,
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

impl TryFrom<device::Device> for Platform {
    type Error = String;
    fn try_from(svd_device: device::Device) -> Result<Self, Self::Error> {
        let mut this = Self {
            name: svd_device.name,
            device_types: Vec::new(),
            interrupts: Vec::new(),
            bus_width: svd_device.width,
            devices: Vec::new(),
        };

        for periph in &svd_device.peripherals {
            let device_iter = match periph {
                peripheral::Peripheral::Single(info) => info,
                peripheral::Peripheral::Array(_, _) => {
                    return Err("Peripheral array not supported".to_string());
                }
            };

            // i.e 0x8000_0000
            let device_addr = device_iter.base_address;
            // i.e UART0
            let device_name = device_iter.name.replace(" ", "_").to_uppercase();
            // i.e UART
            let device_type =
                super::device_type(device_iter.derived_from.as_ref().unwrap_or(&device_name));

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
                    registercluster::RegisterCluster::Register(register) => {
                        match register.try_into() {
                            Ok(register) => device.registers.push(register),
                            Err(msg) => println!("Warning: {} in {}, skipping.", msg, device_name),
                        }
                    }
                    registercluster::RegisterCluster::Cluster(cluster) => {
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
