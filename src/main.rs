// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use zermiolib::generator;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Subcommand, Debug)]
enum Input {
    ImportSvd {
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
        dir: PathBuf,

        /// A dir to output the header with the peripheral addresses.
        #[arg(long, short, value_parser =  PathBuf::from_str)]
        periph_dir: Option<PathBuf>,
    },
}

// Define a struct to represent command-line options
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    input: Input,
}

fn main() -> anyhow::Result<(), String> {
    let args: Args = Args::parse();

    let (device, output) = match args.input {
        Input::ImportSvd { svd, output } => {
            if !svd.exists() {
                return Err("Svd does not exist!".to_string());
            }

            println!("Loading the {}...", svd.display());
            let xml = std::fs::read_to_string(&svd).unwrap();
            let svd_rs = svd_parser::parse(&xml).unwrap().try_into()?;
            (svd_rs, output)
        }
    };

    match output {
        Output::ExportCpp { dir, periph_dir } => {
            if !dir.is_dir() {
                return Err("Output path is not a dir!".to_string());
            }
            let periph_dir = periph_dir.unwrap_or(dir.clone());
            if !periph_dir.is_dir() {
                return Err("Addresses path is not a dir!".to_string());
            }

            generator::cpp::generate(&device, dir, periph_dir).unwrap();
        }
    }

    Ok(())
}
