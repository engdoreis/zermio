// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use crate::mmio;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub vendor: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub license_text: String,
    pub cpu: Cpu,
    pub address_unit_bits: u32,
    pub width: u32,
    pub peripherals: Peripherals,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cpu {
    pub name: String,
    pub revision: String,
    pub endian: String,
    pub mpu_present: bool,
    pub fpu_present: bool,
    pub vtor_present: bool,
    pub nvic_prio_bits: u32,
    pub vendor_systick_config: bool,
    pub device_num_interrupts: u32,
}

#[derive(Debug, Deserialize)]
pub struct Peripherals {
    pub peripheral: Vec<Peripheral>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Peripheral {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    #[serde(deserialize_with = "from_hex_string")]
    pub base_address: u64,
    #[serde(deserialize_with = "from_hex_string", default)]
    pub size: u64,
    pub address_block: Option<AddressBlock>,
    #[serde(default)]
    pub interrupt: Vec<Interrupt>,
    pub registers: Option<Registers>,
    #[serde(rename = "@derivedFrom")]
    pub derived_from: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressBlock {
    #[serde(deserialize_with = "from_hex_string")]
    pub offset: u64,
    #[serde(deserialize_with = "from_hex_string")]
    pub size: u64,
    pub usage: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Interrupt {
    pub name: String,
    pub value: u32,
}

#[derive(Debug, Deserialize)]
pub struct Registers {
    pub register: Vec<Register>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Register {
    pub name: String,
    pub description: String,
    #[serde(deserialize_with = "from_hex_string")]
    pub address_offset: u64,
    #[serde(deserialize_with = "from_hex_string", default)]
    pub reset_value: u64,
    pub fields: Fields,
}

#[derive(Debug, Deserialize)]
pub struct Fields {
    pub field: Vec<Field>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    pub description: String,
    #[serde(rename = "bitRange", deserialize_with = "from_bitrange")]
    pub bit_range: AddressBlock,
    #[serde(deserialize_with = "from_acess")]
    pub access: mmio::Permissions,
}

// Helper function to deserialize hexadecimal strings
fn from_hex_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.starts_with("0x") {
        u64::from_str_radix(s.trim_start_matches("0x"), 16).map_err(serde::de::Error::custom)
    } else {
        s.parse::<u64>().map_err(serde::de::Error::custom)
    }
}

fn from_acess<'de, D>(deserializer: D) -> Result<mmio::Permissions, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(String::deserialize(deserializer)?.as_str().into())
}

fn from_bitrange<'de, D>(deserializer: D) -> Result<AddressBlock, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let re: regex::Regex = regex::Regex::new(r"\[(?<upper>\d+):(?<lower>\d+)\]").unwrap();

    let Some(caps) = re.captures(&s) else {
        panic!("no match!: \n{}", s);
    };
    let upper: u64 = caps["upper"].parse().unwrap();
    let offset: u64 = caps["lower"].parse().unwrap();
    Ok(AddressBlock {
        offset,
        size: upper - offset + 1,
        usage: "bitfield".to_owned(),
    })
}
