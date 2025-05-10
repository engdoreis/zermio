// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use askama::Template;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::filters;
use crate::mmio;
use crate::schema;

use regex::Regex;
fn peripheral_name(s: &str) -> String {
    let re = Regex::new(r"\d+$").unwrap();
    // Remove trailing digits
    re.replace(s, "").to_string().to_lowercase()
}

pub mod cpp {
    use super::*;

    #[derive(Template)]
    #[template(
        ext = "txt",
        source = "
    #pragma once
    #include <cstdint>
     enum Peripherals: uintptr_t{ 
        {% for peripheral in data.peripherals -%}
        {{ peripheral.name|pascal_case }} = {{peripheral.address}},
        {% endfor -%}
     };

    "
    )]
    pub struct PeripheralAddresses<'a> {
        pub data: &'a mmio::PeripheralAddresses<'a>,
    }

    #[derive(Template)]
    #[template(
        ext = "txt",
        source = "
     class {{ data.name|pascal_case }} { 
        protected:
        {% for register in data.registers -%}
        {{ register|pascal_case }}Reg {{register|lower}};
        {% endfor %}
        
        constexpr {{ data.name|pascal_case }} (uintptr_t addr): 
        {%- for register in data.registers -%}
            {{register|lower}}(addr) {%- if !loop.last -%},{%- endif -%}
        {%- endfor -%}
        {}
     };

    "
    )]
    pub struct Peripheral<'a> {
        pub data: &'a mmio::Peripheral<'a>,
    }

    #[derive(Template)]
    #[template(
        ext = "txt",
        source = "
     // {{ data.desc }}
     struct {{ data.name|pascal_case }}Reg: Mmio<{{ data.name|pascal_case }}Reg> { 
        {% for bitfield in data.bitfields -%}
        // {{ bitfield.desc }}
        Mmio::BitField<{{ data.name|pascal_case }}Reg, {{ bitfield.offset }}, {{ bitfield.bit_size }}> {{ bitfield.name|lower }};
        {% endfor -%}
        
        constexpr {{ data.name|pascal_case }}Reg (uintptr_t addr): Mmio(addr + {{ data.offset }}),
        {%- for bitfield in data.bitfields -%}
            {{ bitfield.name|lower }}(this) {%- if !loop.last -%},{%- endif -%}
        {%- endfor -%}
        {}
     };

    "
    )]
    pub struct Register<'a> {
        pub data: &'a mmio::Register<'a>,
    }

    pub fn generate(soc: &schema::Device, output: PathBuf) -> anyhow::Result<()> {
        let filepath = |name: &str| -> anyhow::Result<PathBuf> {
            let mut filename = output.clone();
            filename.push(name);
            filename.set_extension("hh");
            Ok(filename)
        };
        let name = soc.name.replace(" ", "_").to_lowercase();
        let mut platform = mmio::PeripheralAddresses {
            name: &name,
            peripherals: Vec::new(),
        };

        for peripheral_it in &soc.peripherals.peripheral {
            let name = peripheral_it.name.replace(" ", "_").to_uppercase();
            platform.peripherals.push(mmio::PeripheralAddress {
                name,
                address: format!("{:#x}", peripheral_it.base_address),
            });

            let Some(registers) = &peripheral_it.registers.as_ref() else {
                continue;
            };

            let peripheral_name = peripheral_name(&peripheral_it.name);

            let peripheral_header = filepath(&peripheral_name)?;
            let mut peripheral_handler = File::create(&peripheral_header)?;

            writeln!(peripheral_handler, r###"#pragma once "###)?;
            writeln!(peripheral_handler, r###"#include  "mmio.hh" "###)?;
            writeln!(peripheral_handler, "namespace MMIO {{",)?;

            let mut peripheral = mmio::Peripheral::new(&peripheral_name);
            for register_it in &registers.register {
                peripheral.registers.push(&register_it.name);

                let mut register = mmio::Register::new(
                    &register_it.name,
                    register_it.address_offset as u32,
                    Some(&register_it.description),
                );
                for register_field in &register_it.fields.field {
                    register.bitfields.push(mmio::Bitfields::new(
                        &register_field.name,
                        register_field.bit_range.offset as u32,
                        register_field.bit_range.size as u32,
                        Some(&register_field.description),
                    ));
                }
                writeln!(
                    peripheral_handler,
                    "{}",
                    Register { data: &register }.render().unwrap()
                )?;
                // println!("{}", Register { data: &register }.render().unwrap());
            }
            writeln!(
                peripheral_handler,
                "{}",
                Peripheral { data: &peripheral }.render().unwrap()
            )?;
            writeln!(
                peripheral_handler,
                "}} // namespace {}",
                peripheral_name.to_uppercase()
            )?;
            println!("{} generated", peripheral_header.display());
        }

        let platform_header = filepath(&format!(
            "{}_peripherals",
            soc.name.replace(" ", "_").to_lowercase()
        ))?;
        let mut platform_header_handle = File::create(&platform_header)?;
        writeln!(
            platform_header_handle,
            "{}",
            PeripheralAddresses { data: &platform }.render().unwrap()
        )?;
        println!("{} generated", platform_header.display());
        Ok(())
    }
}
