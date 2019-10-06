#[macro_use]
extern crate lazy_static;

mod util;
mod bms;
mod cbms;
mod compiler;
mod cbms_printer;


use self::compiler::*;
use std::io;
use std::io::prelude::*;
use std::str::FromStr;
use std::fmt::Display;
use self::util::GenericError;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut file_path = String::new();
    match args.len() {
        1 => {
            println!("Specify file path:");
            io::stdin().read_line(&mut file_path)?;
        },
        2 => file_path = args[1].clone(),
        _ => return Err(Box::new(GenericError::from_str("Invalid argument count! Expected 0 or 1.")?)),
    }

    println!("Importing...");
    let imported_bms = import_bms_from_file(file_path.trim())
        .expect("Error importing BMS: ");
    println!(" Title: {}\n BPM: {}", imported_bms.title, imported_bms.bpm);
    println!("Compiling...");
    let cbms = imported_bms.eval_and_compile();
    println!("Compiled BMS. Bar count: {} ({} measure sets)", cbms.bar_count(), cbms.measure_sets.len());
    loop {
        let mut buf = String::new();
        println!("Type in bar no. or \"end\", or \"restable\":");
        io::stdin().read_line(&mut buf)?;
        match buf.trim() {
            "end" => break,
            "restable" => {
                println!("Printing current resource table:");
                for (idx, resource_path) in imported_bms.resource_table.iter().enumerate() {
                    println!("Resource no. {:04}: {}", idx, resource_path);
                }
            },
            "print_timed" => print_timed_bms(&cbms, &imported_bms.timing),
            _ => {
                let bar = usize::from_str(&buf.trim())?;
                if bar >= cbms.bar_count() { return Err(Box::new(GenericError::from_str("Bar out of bounds!")?)); }
                println!("Here's bar no. {}:", bar);   
                cbms_printer::print_cbms_bar(&cbms, bar, 11, 12).or_else(|e| {
                    match e {
                        cbms::CBMSError::BarIsEmpty => { println!("Bar is empty"); Ok(()) },
                        cbms::CBMSError::BarOutOfRange => return Err(GenericError::from_str("Bar out of bounds!").unwrap()),
                    }
                })?;
            },
        }
    }
    println!("Goodbye!");
    Ok(())
}

fn print_timed_bms(cbms: &cbms::CBMS, timings: &bms::BMSTimings) {
    let mut iter = cbms.iter()
        .flatten()
        .filter(|(cmd_idx, _)| cbms.command(*cmd_idx).unwrap().value != 0)
        .map(|(cmd_idx, bms_time)| (cbms.command(cmd_idx).unwrap(), bms_time.to_absolute_time(timings, None)));
    for (command, time) in iter {
        println!("{:08}: [{}] {}", time, command.channel, command.value);
    }
}