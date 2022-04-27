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
    qty: String,
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

fn wrtie_file(records: Vec<StringRecord>) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    for each in records.iter() {
        wtr.serialize(Po {
            po: each.get(0).unwrap().to_owned(),
            style_code: each.get(1).unwrap().to_owned(),
            color_code: each.get(2).unwrap().to_owned(),
            msrp_size: each.get(3).unwrap().to_owned(),
            style_desc: each.get(4).unwrap().to_owned(),
            color_desc: each.get(5).unwrap().to_owned(),
            upc: each.get(6).unwrap().to_owned(),
            store_num: each.get(7).unwrap().to_owned(),
            qty: each.get(8).unwrap().to_owned(),
        })?;
    }
    wtr.flush()?;

    Ok(())
}

fn main() -> Result<()> {
    let results = read_file()?;
    // println!("{:#?}", results);
    wrtie_file(results);
    Ok(())
}
