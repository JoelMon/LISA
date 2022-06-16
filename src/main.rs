use anyhow::{Context, Ok, Result};
use clap::Parser;
use csv::StringRecord;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
use lisa::message_box::ErrorMsgBox;

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
        let num = format!("-{}", &num);
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
    info!("Entering list()");

    let file = std::fs::read_to_string(path)
        .expect(
            "[ list() ] Could not read the file containing the stores to search for, check file",
        )
        .lines()
        .collect::<String>();

    let file = file
        .split(",")
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();

    debug!("file: {:#?}", &file);
    info!("Exiting list()");
    file
}

fn write_file(
    records: Vec<StringRecord>,
    destination_path: PathBuf,
    print_all: bool,
) -> Result<()> {
    info!("Entering write_file");
    debug!("`records` parameter: {:#?}", &records);
    debug!("destination_path: {}", &destination_path.to_str().unwrap());
    debug!("print_all: {}", &print_all);

    // Create a list of stores.
    //
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

        // println!("Saving file: {}", &file_name.to_string_lossy());

        let mut wtr = csv::Writer::from_writer(File::create(&file_name)?);

        for item in records.iter() {
            debug!(
                "The item being worked on: {} with UPC: {}",
                &item.get(0).unwrap().to_string(),
                &item.get(6).unwrap().to_string(),
            );
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
    info!("Entering produce_report()");
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
    let mut t_stores: u32 = 0;

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
        t_stores = t_stores + 1;
    }

    println!(
        "\nTOTALS FOR THIS ORDER:
        TOTAL STORES: {}
        TOTAL LABELS: {}
        NEEDS RFID PRINTED: {}
        MAY NOT NEED RFID: {}
        TOTAL BOXES: {}",
        t_stores,
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
    info!("Entering produce_po_files");
    debug!("list_path: {}", &list_path.to_str().unwrap());
    debug!("read_path: {}", &read_path.to_str().unwrap());
    debug!("output_path: {}", &output_path.to_str().unwrap());
    debug!("print_all: {}", &print_all);

    let store_list: Vec<String> = list(list_path);
    let results = read_file(read_path)?;
    let results = filter_store(results, store_list)?;
    match write_file(results, output_path, print_all) {
        Result::Ok(_) => {
            info!("write_file returned with Ok(), exciting produce_po_files");
            Ok(())
        }
        Err(e) => panic!("{}", e),
    }
}

#[derive(Debug, Default)]
struct Gui {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    list: Option<PathBuf>,
}

enum PathKind {
    Input,
    Output,
    List,
}

impl Gui {
    fn put_path(&mut self, path: Option<PathBuf>, kind: PathKind) -> &mut Gui {
        match kind {
            PathKind::Input => {
                self.input = path;
                self
            }
            PathKind::Output => {
                self.output = path;
                self
            }
            PathKind::List => {
                self.list = path;
                self
            }
        }
    }

    fn get_path(&mut self, kind: PathKind) -> Option<&PathBuf> {
        match kind {
            PathKind::Input => self.input.as_ref(),
            PathKind::Output => self.output.as_ref(),
            PathKind::List => self.list.as_ref(),
        }
    }
}

impl eframe::App for Gui {
    // TODO: Major need for refactoring. Move logic out of GUI code.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let mut paths = Gui::default();
        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("Files");

            egui::SidePanel::left("right_panel")
                .resizable(true)
                .default_width(150.0)
                .width_range(80.0..=200.0)
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Right Panel");
                    });
                    egui::ScrollArea::vertical().show(ui, |ui| ui.label("text"));
                });

            egui::Window::new("Process POs").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Input").clicked() {
                        let path = rfd::FileDialog::new()
                            .add_filter("csv", &["csv", "txt"])
                            .set_title("Select input file...")
                            .pick_file();

                        Gui::put_path(self, path, PathKind::Input);
                    }
                    let path = match Gui::get_path(self, PathKind::Input) {
                        Some(path) => path.to_str().unwrap(),
                        None => "Select a PO file.",
                    };
                    ui.label(path);
                });

                ui.horizontal(|ui| {
                    if ui.button("Output").clicked() {
                        let path = rfd::FileDialog::new()
                            .set_title("Select where to save output...")
                            .pick_folder();

                        Gui::put_path(self, path, PathKind::Output);
                    }

                    let path = match Gui::get_path(self, PathKind::Output) {
                        Some(path) => path.to_str().unwrap(),
                        None => "Select a destination.",
                    };
                    ui.label(path);
                });

                ui.horizontal(|ui| {
                    if ui.button("List").clicked() {
                        let path = rfd::FileDialog::new()
                            .set_title("Select list of stores...")
                            .pick_file();

                        Gui::put_path(self, path, PathKind::List);
                    }
                    let path = match Gui::get_path(self, PathKind::List) {
                        Some(path) => path.to_str().unwrap(),
                        None => "Select list of stores",
                    };
                    ui.label(path);
                });

                if ui.button("Run").clicked() {
                    let read_path = match Gui::get_path(self, PathKind::Input) {
                        Some(path) => path.to_owned(),
                        None => {
                            lisa::message_box::empty_field(ErrorMsgBox::EmptyInputField);
                            panic!("Input field can not be empty."); // TODO: Replace with proper error handling.
                        }
                    };

                    let output_path = match Gui::get_path(self, PathKind::Output) {
                        Some(path) => path.to_owned(),
                        None => {
                            lisa::message_box::empty_field(ErrorMsgBox::EmptyOutputField);
                            panic!("Output field can not be empty."); // TODO: Replace with proper error handling.
                        }
                    };
                    let list_path = match Gui::get_path(self, PathKind::List) {
                        Some(path) => path.to_owned(),
                        None => {
                            lisa::message_box::empty_field(ErrorMsgBox::EmptyListField);
                            panic!("List field can not be empty."); // TODO: Replace with proper error handling.
                        }
                    };

                    let print_all = false;
                    let _results: Result<(), anyhow::Error> =
                        produce_po_files(list_path, read_path, output_path, print_all)
                            .context("Something went wrong while 'produce_po_files()'");
                }
            });
        });
    }
}

fn run_gui() {
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native("LISA", options, Box::new(|_cc| Box::new(Gui::default())));
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The PO csv file to be used
    #[clap(short, long, parse(from_os_str), required_unless_present = "gui")]
    input: Option<PathBuf>,
    /// The destination directory where the processed POs will be saved
    #[clap(short, long, parse(from_os_str), required_unless_present_any = &["gui", "report"])]
    output: Option<PathBuf>,
    /// The text file that contains all of the store numbers to be processed
    #[clap(short, long, parse(from_os_str), required_unless_present = "gui")]
    list: Option<PathBuf>,
    /// Print all RFIDs including items marked with a '$'
    #[clap(short = 'a', long = "print-all")]
    printall: bool,
    /// Produce a report of selected PO
    #[clap(short, long, conflicts_with_all = &["printall"])]
    report: bool,
    /// Runs LISA in GUI mode
    #[clap(long = "gui", exclusive = true)]
    gui: bool,
}
fn run_app() -> Result<()> {
    info!("[run_app] Entering run_app()");
    let args = Cli::parse();

    // Run is_gui first to avoid
    let is_gui: bool = args.gui;
    if is_gui {
        run_gui();
    }

    // Default behavior is not to print items that contain a '$' at the end of the line
    let list_path: PathBuf = args.list.unwrap_or_default();
    let output_path: PathBuf = args.output.unwrap_or_default();
    let read_path: PathBuf = args.input.unwrap_or_default();
    let print_all: bool = args.printall;
    let is_report: bool = args.report;

    debug!("[run_app] is_report is set to: {}", &is_report);
    debug!("[run_app] is_gui is set to: {}", &is_gui);

    match is_report {
        true => produce_report(list_path, read_path)?,
        false => produce_po_files(list_path, read_path, output_path, print_all)?,
    }

    Ok(())
}

fn main() {
    pretty_env_logger::init();

    info!("[main] Initialling application");

    std::process::exit(match run_app() {
        Result::Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {err:?}");
            1
        }
    });
}
