use anyhow::Ok;
use anyhow::{Context, Result};
use clap::Parser;
use csv::StringRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Args {
    #[clap(short, long)]
    input: PathBuf,
    #[clap(short, long)]
    output: PathBuf,
    #[clap(short, long)]
    list: PathBuf,
}

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

fn read_file(file_path: PathBuf) -> Result<Vec<StringRecord>> {
    let file = File::open(file_path).context("Failed to open file")?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records: Vec<StringRecord> = vec![];

    for result in rdr.records() {
        records.push(result?);
    }

    Ok(records)
}

fn filter_store(records: Vec<StringRecord>, list: Vec<&str>) -> Result<Vec<StringRecord>> {
    let mut filtered_records = vec![];

    for num in list {
        for item in records.clone().into_iter() {
            if item.get(0).unwrap().to_string().contains(num) {
                filtered_records.push(item)
            }
        }
    }

    Ok(filtered_records)
}

fn has_rfid(record: &StringRecord) -> bool {
    if record.get(4).unwrap().to_string().contains("$") {
        return true;
    }

    return false;
}

fn write_file(records: Vec<StringRecord>, destination_path: PathBuf) -> Result<()> {
    let store_list = records
        .iter()
        .map(|num| num.get(0).unwrap().to_owned())
        .collect::<HashSet<String>>();

    for store in store_list {
        let mut dest = destination_path;
        let mut file_path = dest.set_file_name("test");
        let mut wtr = csv::Writer::from_writer(File::create(file_path)?);

        for each in records.iter() {
            // Filter orders that has a '$' to qty "0".
            if has_rfid(each) && each.get(0).unwrap().to_owned() == store {
                wtr.serialize(Po {
                    po: each.get(0).unwrap().to_owned(),
                    style_code: each.get(1).unwrap().to_owned(),
                    color_code: each.get(2).unwrap().to_owned(),
                    msrp_size: each.get(3).unwrap().to_owned(),
                    style_desc: each.get(4).unwrap().to_owned(),
                    color_desc: each.get(5).unwrap().to_owned(),
                    upc: each.get(6).unwrap().to_owned(),
                    store_num: "".to_owned(), // This field must an empty string
                    qty: "0".to_owned(),      // If it has RFID then set qty to 0
                })?;
            } else if each.get(0).unwrap().to_owned() == store {
                wtr.serialize(Po {
                    po: each.get(0).unwrap().to_owned(),
                    style_code: each.get(1).unwrap().to_owned(),
                    color_code: each.get(2).unwrap().to_owned(),
                    msrp_size: each.get(3).unwrap().to_owned(),
                    style_desc: each.get(4).unwrap().to_owned(),
                    color_desc: each.get(5).unwrap().to_owned(),
                    upc: each.get(6).unwrap().to_owned(),
                    store_num: "".to_owned(), // This field must an empty string
                    qty: each.get(8).unwrap().to_owned(),
                })?;
            }
        }
        wtr.flush()?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let arg = Args::parse();

    let store_list: Vec<&str> = vec!["014", "071", "123", "010", "100"];
    let results = read_file(arg.input)?;
    let results = filter_store(results, store_list)?;
    write_file(results, arg.output)?;
    Ok(())
}
