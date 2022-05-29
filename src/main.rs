use anyhow::{Context, Ok, Result};
use clap::Parser;
use csv::StringRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The PO csv file to be used
    #[clap(short, long, parse(from_os_str))]
    input: PathBuf,
    /// The destination directory where the processed POs will be saved
    #[clap(short, long, parse(from_os_str))]
    output: PathBuf,
    /// The text file that contains all of the store numbers to be processed
    #[clap(short, long, parse(from_os_str))]
    list: PathBuf,
    /// Print all RFIDs including items marked with a '$'
    #[clap(short = 'a', long = "print-all", conflicts_with = "report")]
    printall: bool,
    /// Produce a report of selected PO
    #[clap(short, long, conflicts_with_all = &["printall", "output"])]
    report: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Order {
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

// filter_store() returns a vector of items that are found in `list: Vec<String>`.
//
// The csv files received for purchase orders for direct to store includes orders made for a
// variety of different stores. Each store is identified by a _store number_.
// This function takes a list which is a list of store numbers we
// are interested in and returns only the POs of the sores found in the list.
//
// The `list` is made by the end user. It is a text file that lists the store numbers
// to be returned.
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

// has_rfid() returns a `true` if an item is _thought_ to have a RFID tag already applied
// from the factory.
//
// Some items already have an RFID tag applied, or will have one in the near future. Items
// that may have said RFID will have a `$` charter at the end of the  item name description.
//
// The reason we care to know this information within the context of this application is because
// if an item already has an RFID tag, we do not need to print an RFID tag. This function dictates
// weather the qty is left as is or set to `0`.
fn has_rfid(record: &StringRecord) -> bool {
    if record.get(4).unwrap().to_string().contains("$") {
        return true;
    }

    return false;
}

// list() takes a path to a text file which contains a list of numbers store numbers.
//
// The csv files received for purchase orders for direct to store includes orders made for a
// variety of different stores. Each store is identified by a _store number_.
//
// This function reads the text file the end user creates which lists all the store numbers
// we are interested in. Each store number contains three digits, for example store `1` would
// be written as `001. Each of the store numbers _must_ be written using a three digit format
// or errors, such as items duplication, will occur. Also, each store number must be separated by
// comma, `,`, for the `list` function to work.
//
// TODO: Come up with a better and more robust method to acquire store numbers from the user.
// TODO: Perhaps using a format such as TOML.
// TODO: Also, write checks and tests to catch user errors when store numbers are added, such as one or two digits for a store number.
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

fn write_file(
    records: Vec<StringRecord>,
    destination_path: PathBuf,
    print_all: bool,
) -> Result<()> {
    // By using a HashSet, we remove all duplicated records from the vector.
    // We acquire a set of unique POs that we can use as file names below.
    let store_list = records
        .iter()
        .map(|num| num.get(0).unwrap().to_owned())
        .collect::<HashSet<String>>();

    let file_path = destination_path;

    // This outer loop creates a file and iterates through `store_list` to find the POs for said file.
    //
    // For example, if we have a store_list of the following POs: ["po14423-001", "po14423-002", "po14423-003"]
    // then this outer loop will step through the list and pull out each po one at a time and do the following:
    //      1) Set the po as a file name
    //      2) Create a find all matching POs in store_list and use it with `wtr.serialize()`
    //      3) Push it to a file
    for store in store_list {
        let file_name = file_path.join(format!("{}.csv", &store));

        println!("Saving file: {}", &file_name.to_string_lossy());

        let mut wtr = csv::Writer::from_writer(File::create(file_name)?);

        for item in records.iter() {
            // If an item contains a `$` in the name description, then the qty should be set to `0`.
            // See comments for `has_rfid()`.
            if has_rfid(item) && !print_all && item.get(0).unwrap().to_owned() == store {
                wtr.serialize(Order {
                    po: item.get(0).unwrap().to_owned(),
                    style_code: item.get(1).unwrap().to_owned(),
                    color_code: item.get(2).unwrap().to_owned(),
                    msrp_size: item.get(3).unwrap().to_owned(),
                    style_desc: item.get(4).unwrap().to_owned(),
                    color_desc: item.get(5).unwrap().to_owned(),
                    upc: item.get(6).unwrap().to_owned(),
                    store_num: "".to_owned(), // This field must always be an empty string
                    qty: "0".to_owned(),      // If it `has_rfid` is `true` then set qty to 0
                })?;
            } else if item.get(0).unwrap().to_owned() == store {
                wtr.serialize(Order {
                    po: item.get(0).unwrap().to_owned(),
                    style_code: item.get(1).unwrap().to_owned(),
                    color_code: item.get(2).unwrap().to_owned(),
                    msrp_size: item.get(3).unwrap().to_owned(),
                    style_desc: item.get(4).unwrap().to_owned(),
                    color_desc: item.get(5).unwrap().to_owned(),
                    upc: item.get(6).unwrap().to_owned(),
                    store_num: "".to_owned(), // This field must always be an empty string
                    qty: item.get(8).unwrap().to_owned(),
                })?;
            }
        }
        wtr.flush()?;
    }

    Ok(())
}

// Produce a report of stores in a PO and the number of items
fn produce_report(list_path: PathBuf, read_path: PathBuf) -> Result<()> {
    let store_list: Vec<String> = list(list_path);
    let results = read_file(read_path)?;
    let results = filter_store(results, store_list)?;

    #[derive(Debug)]
    struct Store {
        store_number: String,
        qty_high: u32,
        qty_low: u32,
    }

    let mut stores: Vec<Store> = Vec::new();

    for item in &results {
        let po = item.get(0).unwrap().to_owned();
        let qty: u32 = item.get(8).unwrap().parse()?;
        let has_rfid: bool = has_rfid(&item);

        let store = match has_rfid {
            true => Store {
                store_number: po,
                qty_high: 0,
                qty_low: qty,
            },
            false => Store {
                store_number: po,
                qty_high: qty,
                qty_low: 0,
            },
        };

        stores.push(store);
    }

    // By using a HashSet, we remove all duplicated records from the vector.
    // We acquire a set of unique POs that we can use as file names below.
    let store_list = results
        .iter()
        .map(|num| num.get(0).unwrap().to_owned())
        .collect::<HashSet<String>>();

    let mut t_high: u32 = 0;
    let mut t_low: u32 = 0;

    for item in store_list {
        let mut high: u32 = 0;
        let mut low: u32 = 0;

        for store in &stores {
            if store.store_number == item {
                high = high + store.qty_high;
                low = low + store.qty_low;
            }
        }

        // Reports by store number
        println!(
            "Store {} - TOTAL: {}. WITH RFID: {} MAY HAVE RFID: {}. {} boxes.",
            item,
            high + low,
            high,
            low,
            ((high as f32 + low as f32) / 60.0).ceil()
        );

        t_high = t_high + high;
        t_low = t_low + low;
    }

    println!(
        "\nTOTALS FOR THIS ORDER:
        TOTAL LABELS: {}
        NEEDS RFID PRINTED: {}
        MAY NOT NEED RFID: {}
        TOTAL BOXES: {}",
        t_high + t_low,
        t_high,
        t_low,
        ((t_high as f32 + t_low as f32) / 60.0).ceil()
    );
    Ok(())
}

fn produce_po_files(
    list_path: PathBuf,
    read_path: PathBuf,
    output_path: PathBuf,
    print_all: bool,
) -> Result<()> {
    let store_list: Vec<String> = list(list_path);
    let results = read_file(read_path)?;
    let results = filter_store(results, store_list)?;
    write_file(results, output_path, print_all);

    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // Default behavior is not to print items that contain a '$' at the end of the line
    let list_path = args.list;
    let output_path = args.output;
    let read_path = args.input;
    let print_all = args.printall;
    let is_report = args.report;

    match is_report {
        true => produce_report(list_path, read_path)?,
        false => produce_po_files(list_path, read_path, output_path, print_all)?,
    }

    Ok(())
}
