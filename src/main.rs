use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use serde_derive::*;

mod categories;
mod category_matcher;
mod converter;

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Row<'a> {
    repository_name: &'a str,
    job_type_name: &'a str,
    platform: &'a str,
    job_group_symbol: &'a str,
    duration: u64,
}

fn main() {
    let path = std::env::args().nth(1).expect("Missing CSV path");
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);
    let mut reader = csv::Reader::from_reader(reader);
    let mut raw_record = csv::StringRecord::new();
    let headers = reader.headers().unwrap().clone();

    let mut converter = converter::Converter::new(&path, categories::CATEGORIES);
    let mut line_number = 2;

    while reader.read_record(&mut raw_record).unwrap() {
        let row: Row = match raw_record.deserialize(Some(&headers)) {
            Ok(row) => row,
            Err(e) => panic!("Error parsing line {line_number}: {e}"),
        };
        converter.process_row(&row, line_number);
        line_number += 1;
    }
    let profile = converter.finish();

    let filename = Path::new(&path).file_name().unwrap().to_string_lossy();
    let out_filename = format!("{filename}-profile.json");
    let out_path = Path::new(&path).with_file_name(out_filename);
    let writer = File::create(out_path).unwrap();
    let writer = BufWriter::new(writer);

    serde_json::to_writer(writer, &profile).unwrap();
}
