use crate::filters;
use askama::Template;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::mmio;

#[derive(Template)]
#[template(path = "rust/lib.rs.txt")]
pub struct Lib<'a> {
    pub inner: &'a mmio::Platform,
}

#[derive(Template)]
#[template(path = "rust/platform.rs.txt")]
pub struct Platform<'a> {
    pub inner: &'a mmio::Platform,
}

#[derive(Template)]
#[template(path = "rust/device.rs.txt")]
pub struct Device<'a> {
    pub inner: &'a mmio::Device,
}

#[derive(Template)]
#[template(path = "rust/register.rs.txt")]
pub struct Register<'a> {
    pub inner: &'a mmio::Register,
}

pub fn generate(soc: &mmio::Platform, out_dir: PathBuf, file_header: &str) -> anyhow::Result<()> {
    let get_path = |path: &PathBuf, name: &str| -> anyhow::Result<(PathBuf, File)> {
        let mut filename = path.clone();
        filename.push(name);
        filename.set_extension("rs");
        let mut file = File::create(&filename)?;
        writeln!(file, "{}", file_header)?;
        Ok((filename, file))
    };

    for device in &soc.devices {
        let (device_filename, mut f_handle) = get_path(&out_dir, &device.type_)?;
        write_device_header_top(&mut f_handle)?;

        let template = Device { inner: device };
        writeln!(
            f_handle,
            "{}",
            template.render().unwrap().replace(",\n\n", ",\n")
        )?;

        for reg in &device.registers {
            let template = Register { inner: reg };
            writeln!(
                f_handle,
                "{}",
                template.render().unwrap().replace(",\n\n", ",\n")
            )?;
        }
        println!("{} generated", device_filename.display());
    }

    let (lib_fname, mut lib_fd) = get_path(&out_dir, "lib.rs")?;
    writeln!(lib_fd, "{}", Lib { inner: soc }.render().unwrap())?;
    println!("{} generated", lib_fname.display());

    let (platform_fname, mut platform_fd) = get_path(&out_dir, &soc.name.to_string())?;
    writeln!(platform_fd, "{}", Platform { inner: soc }.render().unwrap())?;
    println!("{} generated", platform_fname.display());

    std::fs::write(
        out_dir.join("zermio.rs"),
        include_str!("../../../resources/zermio.rs"),
    )?;
    Ok(())
}

fn write_device_header_top(fd: &mut std::fs::File) -> anyhow::Result<()> {
    writeln!(fd, "//zermio")?;
    Ok(())
}
