extern crate regex;
extern crate lazy_static;
extern crate num;

use regex::Regex;
use num::integer::lcm;
use std::str::FromStr;
use std::fs::File;
use std::io::Read;

lazy_static!{
    static ref channel_cmd_regex: Regex = Regex::new(r"#(?P<measure>[0-9]{3})(?P<channel>[0-9]{2}):(?P<indices>[[:alnum:]]*)").unwrap();
}

use crate::cbms::*;
use crate::util::pair_diff;

#[derive(Copy, Clone, Debug)]
pub enum BMSImportError {
    NumericFormatError,
    InvalidBase36Format,
    CouldntOpenFile,
    ErrorReadingFile,
}

#[derive(Copy, Clone, Debug)]
enum BMSCommand {
    Channel(ChannelCommandSet),
    //Other,
}

#[derive(Copy, Clone, Debug)]
struct ChannelCommandSet {
    measure: u32,
    channel: u32,
    args_idx: (usize, usize),
}

#[derive(Debug)]
pub struct ImportedBMS {
    cmd_list: Vec<BMSCommand>,
    channel_args: Vec<u32>,
}



impl ImportedBMS {
    //Currently ignores any commands other than channel commands
    //So, no eval.
    pub fn eval_and_compile(&self) -> CBMS {
        //Copy only channel command sets from cmd_list to channel_cmd_sets
        //Any logic flow, etc. should take place here
        let mut channel_cmd_sets = Vec::new();
        for cmd in &self.cmd_list {
            if let BMSCommand::Channel(ch_set) = cmd {
                channel_cmd_sets.push(ch_set);
            }
        }
        //Sort command sets
        channel_cmd_sets.sort_by(|a, b| a.measure.cmp(&b.measure));
        //Initialize CBMS vectors
        let mut command_cnt = Vec::new();
        let mut commands = Vec::new();
        let mut measure_sets = Vec::new();
        let mut idx = 0;
        let mut command_cnt_idx = 0;
        let mut command_idx = 0;
        //Translate data to CBMS format
        while idx < channel_cmd_sets.len() {
            let measure = channel_cmd_sets[idx].measure;
            let mut arg_cnt_lcm = pair_diff(channel_cmd_sets[idx].args_idx);
            let mut set_cnt = 1;
            while idx + set_cnt < channel_cmd_sets.len() && channel_cmd_sets[idx + set_cnt].measure == measure {
                arg_cnt_lcm = lcm(arg_cnt_lcm, pair_diff(channel_cmd_sets[idx + set_cnt].args_idx));
                set_cnt += 1;
            }
            //Skip empty bars
            if arg_cnt_lcm == 0 {
                idx += 1;
                continue;
            }
            for i in 0 .. arg_cnt_lcm {
                let mut cmd_cnt = 0;
                for set in &channel_cmd_sets[idx .. idx + set_cnt] {
                    if i % (arg_cnt_lcm / pair_diff(set.args_idx)) == 0 {
                        cmd_cnt += 1;
                        commands.push(ChannelCommand {
                            channel: set.channel,
                            value: self.channel_args[set.args_idx.0 + (i * pair_diff(set.args_idx)) / arg_cnt_lcm],
                        });
                    }
                }
                command_cnt.push(cmd_cnt);
            }
            measure_sets.push(MeasureCommandSet {
                measure,
                command_cnt_idx: (command_cnt_idx, command_cnt.len()),
                commands_idx: (command_idx, commands.len()),
            });
            command_cnt_idx = command_cnt.len();
            command_idx = commands.len(); 
            idx += set_cnt;
        }
        CBMS {
            command_cnt,
            commands,
            measure_sets,
        }
    }
}

pub fn import_bms_from_file(path: &str) -> Result<ImportedBMS, BMSImportError> {
    let mut file = File::open(path)
        .or_else(|_| Err(BMSImportError::CouldntOpenFile))?;
    let mut file_str = String::new();
    file.read_to_string(&mut file_str)
        .or_else(|_| Err(BMSImportError::ErrorReadingFile))?;
    import_bms(&file_str)
}

pub fn import_bms(raw_bms: &str) -> Result<ImportedBMS, BMSImportError> {
    let mut cmd_list = Vec::new();
    let mut channel_args = Vec::new();
    for line in raw_bms.lines() {
        if let Some(cmd) = parse_bmscript_line(line, &mut channel_args)? {
            cmd_list.push(cmd);
        }
    }
    Ok(ImportedBMS {
        cmd_list,
        channel_args,
    })
}

fn parse_bmscript_line(line: &str, channel_args: &mut Vec<u32>) -> Result<Option<BMSCommand>, BMSImportError> {
    //println!("baba");
    if let Some(captures) = channel_cmd_regex.captures(line) {
        //println!("dziad");
        let args_beg = channel_args.len();
        let mut args_cnt = 0;
        let measure = u32::from_str(captures.name("measure").unwrap().as_str())
            .or_else(|_| Err(BMSImportError::NumericFormatError))?;
        let channel = u32::from_str(captures.name("channel").unwrap().as_str())
            .or_else(|_| Err(BMSImportError::NumericFormatError))?;
        let indices_str = captures.name("indices").unwrap().as_str();
        //println!("ind: {}", indices_str);
        push_indices_from_str_to_arglist(indices_str, channel_args, &mut args_cnt)?;
        let channel_cmd = BMSCommand::Channel(ChannelCommandSet{
            measure,
            channel,
            args_idx: (args_beg, args_beg + args_cnt)
        });
        return Ok(Some(channel_cmd));
    }
    Ok(None)
}

fn push_indices_from_str_to_arglist(indices_str: &str, args: &mut Vec<u32>, args_cnt: &mut usize) -> Result<(), BMSImportError> {
    let mut a_iter = indices_str.chars().step_by(2);
    let mut b_iter = indices_str.chars().skip(1).step_by(2);
    while let (Some(a), Some(b)) = (a_iter.next(), b_iter.next()) {
        let b36num_str = [a, b];
        let num = from_base36(b36num_str.iter())
            .or_else(|_| Err(BMSImportError::InvalidBase36Format))?;
        args.push(num);
        *args_cnt += 1;
    }
    Ok(())
}

fn from_base36<'a, I>(numstr: I) -> Result<u32, ()> where I: IntoIterator<Item = &'a char> {
    let mut v = 0;
    let iter = numstr.into_iter();
    for &c in iter {
        v *= 36;
        if c >= '0' && c <= '9' {
            v += c as u32 - '0' as u32;
        } else if c >= 'A' && c <= 'Z' {
            v += c as u32 - 'A' as u32 + 10;
        } else if c >= 'a' && c<= 'z' {
            v += c as u32 - 'a' as u32 + 10;
        } else {
            return Err(());
        }
    }
    Ok(v)
}