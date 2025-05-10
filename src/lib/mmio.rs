// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

pub struct Peripheral<'a> {
    pub name: &'a str,
    pub registers: Vec<&'a str>,
}

impl<'a> Peripheral<'a> {
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
}

impl<'a> Bitfields<'a> {
    pub fn new(name: &'a str, offset: u32, bit_size: u32, desc: Option<&'a str>) -> Self {
        Self {
            name,
            offset,
            bit_size,
            desc: desc.unwrap_or(name),
        }
    }
}

pub struct PeripheralAddress {
    pub name: String,
    pub address: String,
}

pub struct PeripheralAddresses<'a> {
    pub name: &'a str,
    pub peripherals: Vec<PeripheralAddress>,
}
