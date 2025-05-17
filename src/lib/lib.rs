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
    fn generate_cpp_from_svd() {
        let svd = PathBuf::from("resources/tests/input/i2c.svd");
        let snapshot_dir = PathBuf::from("resources/tests/snapshots");
        let output_dir = PathBuf::from("target/");

        let file = File::open(svd).unwrap();
        let reader = std::io::BufReader::new(file);
        let schema = quick_xml::de::from_reader(reader).unwrap();

        generator::cpp::generate(&schema, output_dir.clone(), output_dir.clone()).unwrap();
        let i2c = output_dir.join("i2c.hh");
        let snapshot = snapshot_dir.join("i2c.hh");

        assert!(
            compare_files(&snapshot, &i2c).unwrap(),
            "Run the command to check the diff: meld {} {}",
            snapshot.as_os_str().to_str().unwrap(),
            i2c.as_os_str().to_str().unwrap()
        );

        let addrs = output_dir.join("test_peripherals.hh");
        let snapshot = snapshot_dir.join("test_peripherals.hh");
        assert!(
            compare_files(&snapshot, &addrs).unwrap(),
            "Run the command to check the diff: meld {} {}",
            snapshot.as_os_str().to_str().unwrap(),
            addrs.as_os_str().to_str().unwrap()
        );
    }

    use anyhow::Result;

    pub fn compare_files(file_path1: &PathBuf, file_path2: &PathBuf) -> Result<bool> {
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
