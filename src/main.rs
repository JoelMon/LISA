use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs::File;
use std::io;
use std::process;

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Po {
    po: String,
    style_code: String,
    color_code: String,
    msrp_size: String,
    style_desc: String,
    color_desc: String,
    upc: String,
    store_num: String,
    qty: i64,
}

fn read_file() -> Result<()> {
    let file_path = "examples/RFID.csv"; // Hard coded path for debugging.
    let file = File::open(file_path).context("Failed to open file")?;
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.records() {
        let record = result?;
        println!("{:?}", record);
    }

    Ok(())
}

fn main() -> Result<()> {
    let results = read_file()?;
    println!("{:?}", results);
    Ok(())
}
