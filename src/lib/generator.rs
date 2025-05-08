use crate::mmio;
use askama::Template;

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
     }
    "
    )]
    pub struct Peripheral<'a> {
        pub data: mmio::Peripheral<'a>,
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
     }
    "
    )]
    pub struct Register<'a> {
        pub data: mmio::Register<'a>,
    }
}
