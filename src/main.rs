// extern crate rmp_serialize as msgpack;
extern crate rustc_serialize;

use rustc_serialize::json;
// use rustc_serialize::{Encodable, Decodable};
// use msgpack::{Encoder, Decoder};

use std::collections::hash_map::{HashMap, Entry};
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use std::io;

type InnerTable = HashMap<String, String>;
type LookupTable = HashMap<String, InnerTable>;

fn help() {
    let s = "lookup: Basic Key-Value store\n\
             Usage:
                 lookup set <project> <sample> <path> - add value to database, retrievable by key
                 lookup get <project> <sample>        - retrieve value associated with key
                 lookup get DB                        - get database location on disk
                 lookup delete <project>              - remove project and value from database
                 lookup delete <project> <sample>     - remove sample from project
                 lookup list                          - display contents of database
                 lookup help                          - display help
             ";
    println!("{}", s);
    std::process::exit(1);
}

fn main() {

    let db_file = std::env::home_dir().expect("Error finding HOME directory")
                                      .join(Path::new(".lookup.db"));
    let mut table: LookupTable;
    let mut f: File;

    if db_file.exists() {
        f = File::open(&db_file).expect("Error opening database file");
        let mut encoded = String::new();
        f.read_to_string(&mut encoded).expect("Error reading database contents");
        table = json::decode(encoded.as_str()).expect("Error decoding json");
    } else {
        println!("Creating database file {:?}", &db_file);
        f = File::create(&db_file).expect("Error creating database file");
        table = LookupTable::new();
        let mut inner = InnerTable::new();
        inner.insert("DB".to_string(), db_file.to_str().expect("Error collecting filename").to_string());
        table.insert("DB".to_string(), inner.clone());
        let encoded = json::encode(&table).expect("Error encoding json") + "\n";
        f.write_all(encoded.as_bytes()).expect("Error writing to file");
    };
    let initial_condition_table: LookupTable = table.clone();

    // Handle args
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.len() {
        0 => {
            println!("error: not enough arguments");
            help();
        }
        1 | 2 | 3 | 4 => {
            let cmd = &args[0];
            match &cmd[..] {
                "help" => help(),
                "get" => {
                    if args.len() < 3 {
                        println!("error: missing value for GET");
                        help();
                    }
                    match table.get(&args[1]) {
                        Some(inner) => {
                            match inner.get(&args[2]) {
                                Some(path) => println!("{}", path),
                                None => println!("error: no value for sample"),
                            };
                        }
                        None => println!("error: no value for project"),
                    };
                }
                "set" => {
                    if args.len() < 4 {
                        println!("error: missing value for SET");
                        help();
                    }
                    let inner = table.entry(args[1].clone()).or_insert_with(InnerTable::new);
                    inner.insert(args[2].clone(), args[3].clone());
                }
                "delete" => {
                    if args.len() < 2 {
                        println!("error: missing value for DELETE");
                        help()
                    }
                    if args.len() == 2 {
                        if table.contains_key(&args[1]) {
                            let mut input = String::new();
                            println!("This will delete project {} and all its samples. Continue (y/[n])?", args[1]);

                            if let Err(error) = io::stdin().read_line(&mut input) {
                                println!("error: {}", error);
                            }

                            match input.trim() {
                                "y" => {
                                    table.remove(&args[1]);
                                },
                                _ => println!("Not removing project {}", args[1]),
                            };
                        }
                        else {
                            println!("Project {} is not in the database.", args[1]);
                        }
                    }

                    if args.len() == 3 {
                        match table.entry(args[1].clone()) {
                            Entry::Occupied(mut inner_entry) => {
                                let inner_map = inner_entry.get_mut();
                                inner_map.remove(&args[2]);
                            }
                            Entry::Vacant(_) => {
                                println!("Project {} is not in the database.", args[1]);
                            }
                        }
                    }
                }
                "list" => {
                    for (key, inner) in &table {
                        for (innerkey, path) in inner {
                            if key != "DB" {
                                println!("{}\t{}\t{}", key, innerkey, path);
                            }
                        }
                    }
                }
                _ => {
                    println!("error: invalid command");
                    help();
                }
            };
        }
        _ => {
            println!("error: too many arguments");
            help();
        }
    };

    // Write to disk
    let mut inner = InnerTable::new();
    inner.insert("DB".to_string(), db_file.to_str().expect("Error collecting filename").to_string());
    table.insert("DB".to_string(), inner.clone());
    if table != initial_condition_table {
        let encoded = json::encode(&table).expect("Error encoding json") + "\n";
        let mut f = File::create(&db_file).expect("Error creating file");
        f.write_all(encoded.as_bytes()).expect("Error writing file");
    }
}
