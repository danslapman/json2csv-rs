#[macro_use]
extern crate clap;
extern crate serde_json;

pub mod json2csv;
pub mod schema;
pub mod utils;

use crate::json2csv::{compute_paths, show_value};
use crate::schema::{extract, generate_tuples, json_path_string, to_schema, JsonPath, JsonSchema};
use itertools::Itertools;
use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, LineWriter};

type PathSet = HashSet<JsonPath>;
type PathSetCombine = fn(PathSet, PathSet) -> PathSet;

fn main() -> io::Result<()> {
    let json2csv_app_matches = clap_app!(j2c =>
        (version: "0.1")
        (author: "Daniel Slapman <danslapman@gmail.com>")
        (about: "Json -> CSV conversion utility")
        (@arg json_file: +required "Newline-delimited JSON input file name")
        (@arg csv_file: +required "CSV output file name")
        (@arg flatten: -f --flatten "Flatten array iterators")
        (@arg intersect: -i --intersect "\"Inner join\" fields while constructing schema")
    )
    .get_matches();

    let json_file_name = value_t!(json2csv_app_matches, "json_file", String).expect("json_file");
    let csv_file_name = value_t!(json2csv_app_matches, "csv_file", String).expect("csv_file");

    let header = compute_header_multiline(union, &json_file_name)?;
    let schema = to_schema(header.clone());

    let columns = header.into_iter().unique().map(json_path_string).collect::<Vec<_>>();

    let json_file = File::open(json_file_name)?;
    let reader = BufReader::new(json_file);

    let csv_file = File::create(csv_file_name)?;
    let mut writer = LineWriter::new(csv_file);

    writer.write_all(format!("{}\n", columns.join(";")).as_ref())?;

    let mut n = 0;
    for line in reader.lines() {
        let line_value =
            serde_json::from_str::<Value>(line?.as_str()).expect(format!("Can't parse line {}", n).as_str());
        let lines = extract_lines(|strs| strs.join(";"), false, &schema, &columns, line_value);
        for line in lines {
            writer.write_all(format!("{}\n", line).as_ref())?;
        }
        n += 1;
    }

    writer.flush()
}

fn intersect_or_non_empty(lhs: PathSet, rhs: PathSet) -> PathSet {
    match (lhs, rhs) {
        (l, r) if l.is_empty() => r,
        (l, r) if r.is_empty() => l,
        (l, r) => l.into_iter().filter(|el| r.contains(el)).collect(),
    }
}

fn union(lhs: PathSet, rhs: PathSet) -> PathSet {
    let mut result = lhs.clone();
    result.extend(rhs);
    result
}

fn compute_header_multiline(combine: PathSetCombine, file_name: &String) -> io::Result<Vec<JsonPath>> {
    let json_file = File::open(file_name)?;
    let reader = BufReader::new(json_file);

    let mut pathes = HashSet::new();

    for line in reader.lines() {
        let parsed = serde_json::from_str::<Value>(line?.as_str())?;
        let header = compute_paths(true, parsed).expect("unsupported line contents");
        pathes = combine(pathes, header);
    }

    Ok(pathes.into_iter().collect::<Vec<_>>())
}

fn extract_lines(
    mk_sep_string: fn(Vec<String>) -> String,
    flat: bool,
    schema: &JsonSchema,
    columns: &Vec<String>,
    line_value: Value,
) -> Vec<String> {
    let tree = extract(schema, line_value);

    let tuples = generate_tuples(flat, tree);

    tuples
        .into_iter()
        .map(|hm| {
            mk_sep_string(
                columns
                    .iter()
                    .map(|col| hm.get(col).cloned().map(show_value).unwrap_or_default())
                    .collect(),
            )
        })
        .collect()
}
