// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use reismmiolib::generator;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;

use quick_xml::de::from_reader;
use std::fs::File;
use std::io::BufReader;

#[derive(Subcommand, Debug)]
enum Input {
    InportSvd {
        /// A path to a svd file
        #[arg(long, short, value_parser =  PathBuf::from_str)]
        svd: PathBuf,
        #[command(subcommand)]
        output: Output,
    },
}

#[derive(Subcommand, Debug)]
enum Output {
    ExportCpp {
        /// A dir to output the peripheral implementation.
        #[arg(long, short, value_parser =  PathBuf::from_str)]
        peripheral_dir: PathBuf,

        /// A dir to output the header with the peripheral addresses.
        #[arg(long, short, value_parser =  PathBuf::from_str)]
        addresses_dir: Option<PathBuf>,
    },
}

// Define a struct to represent command-line options
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    input: Input,
}

fn main() -> anyhow::Result<(), &'static str> {
    let args: Args = Args::parse();

    let (device, output) = match args.input {
        Input::InportSvd { svd, output } => {
            if !svd.exists() {
                return Err("Svd does not exist!");
            }
            println!("Loading the {}...", svd.display());
            let file = File::open(svd).unwrap();
            let reader = BufReader::new(file);
            (from_reader(reader).unwrap(), output)
        }
    };

    match output {
        Output::ExportCpp {
            peripheral_dir,
            addresses_dir,
        } => {
            if !peripheral_dir.is_dir() {
                return Err("Output path is not a dir!");
            }
            let addresses_dir = addresses_dir.unwrap_or(peripheral_dir.clone());
            if !addresses_dir.is_dir() {
                return Err("Addresses path is not a dir!");
            }

            generator::cpp::generate(&device, peripheral_dir, addresses_dir).unwrap();
        }
    }

    Ok(())
}
