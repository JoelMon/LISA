use anyhow::{Context, Result};
use csv::StringRecord;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::process;

#[derive(Debug, Deserialize, Serialize)]
#[allow(unused)]
#[serde(rename_all = "PascalCase")]
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

fn read_file() -> Result<Vec<StringRecord>> {
    let file_path = "examples/RFID.csv"; // Hard coded path for debugging.
    let file = File::open(file_path).context("Failed to open file")?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records: Vec<StringRecord> = vec![];

    for result in rdr.records() {
        records.push(result?);
    }

    Ok(records)
}

fn main() -> Result<()> {
    let results = read_file()?;
    println!("{:#?}", results);
    Ok(())
}
