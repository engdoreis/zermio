// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

pub use crate::mmio::Register;
pub use crate::rdljson;

#[derive(Debug)]
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

    pub fn get_type(s: &str) -> String {
        let re = regex::Regex::new(r"\d+$").unwrap();
        // Remove trailing digits
        re.replace(s, "").to_string().to_lowercase()
    }

    pub fn get_cluster_name(base: &str, index: &str, reg: &str) -> String {
        let name = base.to_string().replace("%s", "");
        let reg = reg.to_string().replace(&name, "");
        format!("{}{}{}", name, index, &reg)
    }
}

impl TryFrom<&svd_rs::peripheral::PeripheralInfo> for Device {
    type Error = String;
    fn try_from(periph: &svd_rs::peripheral::PeripheralInfo) -> Result<Self, Self::Error> {
        // i.e UART0
        let device_name = periph.name.replace(" ", "_").to_uppercase();

        // i.e UART
        let device_type = Device::get_type(periph.derived_from.as_ref().unwrap_or(&device_name));

        let mut device = Device::new(&device_name, &device_type);
        let Some(ref registers) = periph.registers else {
            return Ok(device);
        };

        for register_cluster in registers {
            match register_cluster {
                svd_rs::registercluster::RegisterCluster::Register(register) => {
                    device.registers.push(register.into())
                }
                svd_rs::registercluster::RegisterCluster::Cluster(cluster) => {
                    match Register::try_from(cluster) {
                        Ok(registers) => {
                            device.registers.extend(registers);
                        }
                        Err(msg) => println!("Warning: {} in {}, skipping.", msg, device_name),
                    }
                }
            };
        }
        Ok(device)
    }
}

impl From<&rdljson::Peripheral> for Device {
    fn from(periph: &rdljson::Peripheral) -> Self {
        // i.e UART0
        let device_name = periph.name.replace(" ", "_").to_uppercase();

        // i.e UART
        let device_type = &periph.type_name;

        let mut device = Device::new(&device_name, device_type);
        device.registers.extend(
            periph
                .interfaces
                .iter()
                .flat_map(|inter| inter.regs.iter())
                .map(|reg| reg.into())
                .collect::<Vec<_>>(),
        );

        device
    }
}
