// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

mod filters;
pub mod generator;
pub mod mmio;
pub mod schema;

#[cfg(test)]
mod libtest {
    use super::*;
    use std::fs::File;
    use std::path::PathBuf;

    #[test]
    #[function_name::named]
    fn generate_cpp_from_svd() {
        let svd = PathBuf::from("resources/tests/input/i2c.svd");
        let snapshot_dir = PathBuf::from("resources/tests/snapshots");
        let output_dir = PathBuf::from(format!("target/test_{}", function_name!()));

        let _ = std::fs::create_dir(&output_dir);

        let file = File::open(svd).unwrap();
        let reader = std::io::BufReader::new(file);
        let schema = quick_xml::de::from_reader(reader).unwrap();

        generator::cpp::generate(&schema, output_dir.clone(), output_dir.clone()).unwrap();

        let check_eq = |name: &str, snapshot: Option<&str>| {
            let res = output_dir.join(name);
            let snapshot = snapshot_dir.join(snapshot.unwrap_or(name));
            assert!(
                compare_files(&snapshot, &res).unwrap(),
                "Run the command to check the diff: meld {} {}",
                snapshot.as_os_str().to_str().unwrap(),
                res.as_os_str().to_str().unwrap()
            );
        };

        check_eq("i2c.hh", None);
        check_eq("timer.hh", None);
        check_eq("test_platform.hh", None);
        check_eq("mmio.hh", Some("../../mmio.hh"));
    }

    pub fn compare_files(file_path1: &PathBuf, file_path2: &PathBuf) -> anyhow::Result<bool> {
        // Read the contents of the first file into a vector
        let contents1: Vec<_> = std::fs::read(file_path1)
            .expect(&format!(
                "Cant't read file {}",
                file_path1.to_str().unwrap()
            ))
            .into_iter()
            .filter(|x| *x != b'\r' && *x != b'\n')
            .collect();
        // Read the contents of the second file into a vector
        let contents2: Vec<_> = std::fs::read(file_path2)
            .expect(&format!(
                "Cant't read file {}",
                file_path2.to_str().unwrap()
            ))
            .into_iter()
            .filter(|x| *x != b'\r' && *x != b'\n')
            .collect();

        Ok(contents1 == contents2)
    }
}
