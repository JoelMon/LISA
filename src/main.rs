use anyhow::{Context, Ok, Result};
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

fn filter_store(records: Vec<StringRecord>, list: Vec<String>) -> Result<Vec<StringRecord>> {
    let mut filtered_records = vec![];

    for num in list {
        for item in records.clone().into_iter() {
            if item.get(0).unwrap().to_owned().contains(&num) {
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

fn list(path: PathBuf) -> Vec<String> {
    let file = std::fs::read_to_string(path)
        .expect("Could not read the file containing the stores to search for, check file")
        .lines()
        .collect::<String>();

    let file = file
        .split(",")
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();
    file
}

fn write_file(records: Vec<StringRecord>, destination_path: PathBuf) -> Result<()> {
    let store_list = records
        .iter()
        .map(|num| num.get(0).unwrap().to_owned())
        .collect::<HashSet<String>>();

    let file_path = dbg!(destination_path);

    for store in store_list {
        let file_name = dbg!(file_path.join(format!("{}.csv", &store)));
        let mut wtr = csv::Writer::from_writer(File::create(file_name)?);

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

    let store_list: Vec<String> = list(arg.list);
    let results = read_file(arg.input)?;
    let results = filter_store(results, store_list)?;
    write_file(results, arg.output)?;
    Ok(())
}
