use askama::Template;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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
     class {{ data.name|capitalize }} { 
        {% for register in data.registers -%}
        {{ register|capitalize }}Reg {{register|lower}};
        {% endfor -%}
        
        constexpr {{ data.name|capitalize }} (uintptr_t addr): 
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
     struct {{ data.name|capitalize }}Reg: Mmio<{{ data.name|capitalize }}Reg> { 
        {% for bitfield in data.bitfields -%}
        Mmio::BitField<{{ data.name|capitalize }}Reg, {{ bitfield.offset }}, {{ bitfield.bit_size }}> {{ bitfield.name|lower }};
        {% endfor -%}
        
        constexpr {{ data.name|capitalize }}Reg (uintptr_t addr): Mmio(addr + {{ data.offset }}),
        {%- for bitfield in data.bitfields -%}
            {{ bitfield.name|lower }}(addr) {%- if !loop.last -%},{%- endif -%}
        {%- endfor -%}
        {}
     };

    "
    )]
    pub struct Register<'a> {
        pub data: &'a mmio::Register<'a>,
    }

    pub fn generate(soc: &schema::Device, output: PathBuf) -> anyhow::Result<()> {
        for peripheral_it in &soc.peripherals.peripheral {
            let peripheral_name = peripheral_name(&peripheral_it.name);
            let mut filename = output.clone();
            filename.push(&peripheral_name);
            filename.set_extension("cc");
            let mut file = File::create(&filename)?;

            file.write_all(
                &format!("namespace {} {{\n", peripheral_name.to_uppercase()).as_bytes(),
            )?;

            let mut peripheral = mmio::Peripheral::new(&peripheral_name);
            for register_it in &peripheral_it.registers.as_ref().unwrap().register {
                peripheral.registers.push(&register_it.name);

                let mut register =
                    mmio::Register::new(&register_it.name, register_it.address_offset as u32);
                for register_field in &register_it.fields.field {
                    register.bitfields.push(mmio::Bitfields::new(
                        &register_field.name,
                        register_field.bit_range.offset as u32,
                        register_field.bit_range.size as u32,
                    ));
                }
                file.write_all(Register { data: &register }.render().unwrap().as_bytes())?;
                // println!("{}", Register { data: &register }.render().unwrap());
            }
            file.write_all(
                Peripheral { data: &peripheral }
                    .render()
                    .unwrap()
                    .as_bytes(),
            )?;
            file.write_all(
                &format!("}} // namespace {}", peripheral_name.to_uppercase()).as_bytes(),
            )?;
            // println!("{}", Peripheral { data: &peripheral }.render().unwrap());
            println!("File: {} generated", filename.display());
            break;
        }

        Ok(())
    }
}
