extern crate clap;
extern crate serde_json;

pub mod json2csv;
pub mod schema;
pub mod utils;

use crate::json2csv::{compute_paths, show_value};
use crate::schema::{drop_iterators, extract, generate_tuples, json_path_string, to_schema, JsonPath, JsonSchema};
use clap::Parser;
use itertools::Itertools;
use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, LineWriter};

type PathSet = HashSet<JsonPath>;
type PathSetCombine = fn(PathSet, PathSet) -> PathSet;

#[derive(Parser, Debug)]
#[clap(
    author = "Daniel Slapman <danslapman@gmail.com>",
    version = "0.1",
    about = "Json -> CSV conversion utility"
)]
struct Args {
    #[clap(help = "Newline-delimited JSON input file name")]
    json_file: String,
    #[clap(help = "CSV output file name")]
    csv_file: String,
    #[clap(short, long, help = "Flatten array iterators")]
    flatten: bool,
    #[clap(short, long, help = "\"Inner join\" fields while constructing schema")]
    intersect: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let header = compute_header_multiline(
        if args.intersect { intersect_or_non_empty } else { union },
        &(args.json_file),
    )?;
    let schema = to_schema(header.clone());

    let columns = if args.flatten {
        header
            .into_iter()
            .unique()
            .map(drop_iterators)
            .map(json_path_string)
            .collect::<Vec<_>>()
    } else {
        header.into_iter().unique().map(json_path_string).collect::<Vec<_>>()
    };

    let json_file = File::open(args.json_file)?;
    let reader = BufReader::new(json_file);

    let csv_file = File::create(args.csv_file)?;
    let mut writer = LineWriter::new(csv_file);

    writer.write_all(format!("{}\n", columns.join(";")).as_ref())?;

    let mut n = 0;
    for line in reader.lines() {
        let line_value =
            serde_json::from_str::<Value>(line?.as_str()).expect(format!("Can't parse line {}", n).as_str());
        let lines = extract_lines(|strs| strs.join(";"), args.flatten, &schema, &columns, line_value);
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
