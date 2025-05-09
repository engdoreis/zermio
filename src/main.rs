use askama::Template; // bring trait in scope
use reis_mmiogen::generator;
use reis_mmiogen::mmio;
use reis_mmiogen::schema;

use clap::Parser;
use std::path::PathBuf;
use std::str::FromStr;

use quick_xml::de::from_reader;
use std::fs::File;
use std::io::BufReader;

// Define a struct to represent command-line options
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// A path to a svd file
    #[arg(long, value_parser =  PathBuf::from_str)]
    svd: PathBuf,

    /// A path to the output dir
    #[arg(long, value_parser =  PathBuf::from_str)]
    output: PathBuf,
}

fn main() {
    let args: Args = Args::parse();

    println!("Loading xml");

    let file = File::open(args.svd).unwrap();
    let reader = BufReader::new(file);

    let device: schema::Device = from_reader(reader).unwrap();
    // println!("{:#?}", device);
    generator::cpp::generate(&device, args.output).unwrap();
}
