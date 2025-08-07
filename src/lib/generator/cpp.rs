use crate::filters;
use askama::Template;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::mmio;

#[derive(Template)]
#[template(
    ext = "txt",
    source = "
#include  <cstdint>
namespace platform  {
{% for device_type in data.device_types -%}
    /* Addresses for the devices of the type {{ device_type.type_name|pascal_case }}.*/
    enum {{ device_type.type_name|pascal_case }}: uintptr_t{ 
        {%- for device in device_type.devices -%}
            {{ device.name|pascal_case }} = {{device.address}},
        {%- endfor -%}
    };

{% endfor -%}

enum Interrupt: uintptr_t{ 
    {%- for interrupt in data.interrupts -%}
        {{ interrupt.name|pascal_case }} = {{interrupt.value}}, 
    {%- endfor -%}
};
} // namespace platform
"
)]
pub struct Platform<'a> {
    pub data: &'a mmio::Platform,
}

#[derive(Template)]
#[template(
    ext = "txt",
    source = "
/* To facilitate compiler optimization of this abstraction, prefer using this struct within a small scope.*/
struct {{ device.type_|pascal_case }} { 
{% for register in device.registers %}
    {%- for info in register.info -%}
    {{ info.type_|pascal_case }}Reg {{info.name|lower}};
    {% endfor -%}
{% endfor %}
    
    constexpr {{ device.type_|pascal_case }} (platform::{{ device.type_|pascal_case }} addr): 
{%- for register in device.registers -%}
    {%- for info in register.info -%}
        {{info.name|lower}}(addr + {{ info.offset|hex }}){%- if !loop.last -%}, {% endif %}
    {%- endfor -%}
    {%- if !loop.last -%}, {% endif -%}
{%- endfor -%}
    {}
};

"
)]
pub struct Device<'a> {
    pub device: &'a mmio::Device,
}

#[derive(Template)]
#[template(
    ext = "txt",
    source = "
/* {{ register.info[0].desc }} */
union {{ register.info[0].type_|pascal_case }}Reg { 
    private:
      zermio::Register reg_;
    public:
    {% for bitfield in register.bitfields -%}
    /* {{ bitfield.desc }} */
    zermio::BitField<{{ bitfield.offset }}, {{ bitfield.bit_size }}, zermio::Permissions::{{ bitfield.permissions }}> {{ bitfield.name|lower }};
    {% endfor -%}
    
    constexpr {{ register.info[0].type_|pascal_case }}Reg (uintptr_t addr): reg_{.addr = addr}
    {}

    inline void commit() { reg_.commit(); }

    inline {{ register.info[0].name|pascal_case }}Reg& fetch() {
        reg_.fetch();
        return *this;
    }
};

"
)]
pub struct Register<'a> {
    pub register: &'a mmio::Register,
}

pub fn generate(
    soc: &mmio::Platform,
    out_dir: PathBuf,
    addr_dir: PathBuf,
    file_header: &str,
) -> anyhow::Result<()> {
    let get_path = |path: &PathBuf, name: &str| -> anyhow::Result<(PathBuf, File)> {
        let mut filename = path.clone();
        filename.push(name);
        filename.set_extension("hh");
        let mut file = File::create(&filename)?;
        writeln!(file, "{}", file_header)?;
        writeln!(file, "#pragma once")?;
        Ok((filename, file))
    };

    for device_iter in &soc.devices {
        let (device_fd, mut device_handler) = get_path(&out_dir, &device_iter.type_)?;
        write_device_header_top(&mut device_handler)?;

        writeln!(
            device_handler,
            "namespace {} {{",
            device_iter.type_.clone().to_lowercase()
        )?;

        for register_iter in &device_iter.registers {
            writeln!(
                device_handler,
                "{}",
                Register {
                    register: &register_iter
                }
                .render()
                .unwrap()
                .replace(",\n\n", ",\n")
            )?;
        }
        writeln!(
            device_handler,
            "{}",
            Device {
                device: &device_iter
            }
            .render()
            .unwrap()
            .replace(",\n\n", ",\n")
        )?;
        writeln!(
            device_handler,
            "}} // namespace {}\n}} // namespace mmio",
            device_iter.type_.to_lowercase()
        )?;
        println!("{} generated", device_fd.display());
    }

    let (platform_fname, mut platform_fd) = get_path(
        &addr_dir,
        &format!("{}_platform", soc.name.replace(" ", "_").to_lowercase()),
    )?;
    writeln!(platform_fd, "{}", Platform { data: &soc }.render().unwrap())?;
    println!("{} generated", platform_fname.display());

    std::fs::write(
        out_dir.join("mmio.hh"),
        include_str!("../../../resources/mmio.hh"),
    )?;
    Ok(())
}

fn write_device_header_top(fd: &mut std::fs::File) -> anyhow::Result<()> {
    writeln!(
        fd,
        "/* The `platform.hh` should be created and include the specific platform header which will contain the device addresses.*/"
    )?;
    writeln!(fd, r###"#include  "platform.hh" "###)?;
    writeln!(fd, r###"#include  "mmio.hh" "###)?;
    writeln!(fd, "namespace mmio {{")?;
    Ok(())
}
