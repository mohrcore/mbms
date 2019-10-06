extern crate regex;
extern crate lazy_static;
extern crate num;

use regex::Regex;
use num::integer::lcm;
use std::str::FromStr;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::rc::Rc;

lazy_static!{
    static ref CHANNEL_CMD_REGEX: Regex = Regex::new(r"#(?P<measure>[0-9]{3})(?P<channel>[0-9]{2}):(?P<indices>[[:alnum:]]*)").unwrap();
    static ref HEADER_TITLE_REGEX: Regex = Regex::new(r"#TITLE (?P<title>[[:alnum:]]*)").unwrap();
    static ref BPM_REGEX: Regex = Regex::new(r"#BPM (?P<bpm>[[:alnum:]]*)").unwrap();
    static ref WAV_REGEX: Regex = Regex::new(r"WAV(?P<idx>[[:alnum:]]{2}) (?P<path>.*)").unwrap();
}

use crate::cbms::*;
use crate::util::pair_diff;
use crate::bms::{BMSTimings, BMSTime};

#[derive(Copy, Clone, Debug)]
pub enum BMSImportError {
    NumericFormatError,
    InvalidBase36Format,
    CouldntOpenFile,
    ErrorReadingFile,
}

#[derive(Clone, Debug)]
enum BMSCommand {
    Channel(ChannelCommandSet),
    WAVResource {idx: u32, path: String },
    SongInfo(BMSSongInfo),
    //Other,
}

#[derive(Clone, Debug)]
enum BMSSongInfo {
    Title(String),
    BPM(f32),
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
    pub resource_table: Vec<String>,
    pub title: String,
    pub bpm: f32,
    pub timing: BMSTimings,
}

impl ImportedBMS {
    //Currently ignores any commands other than channel commands
    //So, no eval.
    pub fn eval_and_compile(&self) -> CBMS {
        //Copy only channel command sets from cmd_list to channel_cmd_sets
        //Any logic flow, etc. should take place here
        let mut channel_cmd_sets = eval_ibms(&self.cmd_list);
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
            command_cnt: Rc::new(command_cnt),
            commands,
            measure_sets: Rc::new(measure_sets),
        }
    }
}

fn eval_ibms<'l>(cmds: &'l [BMSCommand]) -> Vec<&'l ChannelCommandSet> {
    let mut channel_cmd_sets = Vec::new();
    for cmd in cmds {
        match cmd {
            BMSCommand::Channel(ch_set) => channel_cmd_sets.push(ch_set),
            _ => (),
        }
    }
    channel_cmd_sets
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
    let mut title = String::new();
    let mut bpm = 0.0;
    for line in raw_bms.lines() {
        if let Some(cmd) = parse_bmscript_line(line, &mut channel_args)? {
            if let BMSCommand::SongInfo(ref sinfo) = &cmd {
                match sinfo {
                    BMSSongInfo::Title(t) => title = t.clone(),
                    BMSSongInfo::BPM(b) => bpm = *b,
                }
            }
            cmd_list.push(cmd);
        }
    }
    let resource_table = make_bms_resource_table(&cmd_list);
    Ok(ImportedBMS {
        cmd_list,
        channel_args,
        resource_table,
        title,
        bpm,
        timing: vec![(BMSTime::from(0.0), bpm, 4)], //THIS IS A PLACEHOLDER VALUE, true for most charts tho.
    })
}

fn make_bms_resource_table<'s>(cmd_list: &'s Vec<BMSCommand>) -> Vec<String> {
    let mut wavmap = HashMap::<u32, &'s String>::new();
    let mut wav_highest_idx = -1;
    for cmd in cmd_list {
        if let BMSCommand::WAVResource {idx, path} = cmd {
            wavmap.insert(*idx, &path);
            if *idx as isize > wav_highest_idx {
                wav_highest_idx = *idx as isize;
            }
        }
    }
    let mut paths = vec![String::new(); (wav_highest_idx + 1) as usize];
    for (idx, path) in wavmap {
        paths[idx as usize] = path.clone();
    }
    paths
}

fn parse_bmscript_line(line: &str, channel_args: &mut Vec<u32>) -> Result<Option<BMSCommand>, BMSImportError> {
    //Capture channel commands
    if let Some(captures) = CHANNEL_CMD_REGEX.captures(line) {
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
    //Capture WAV resource definitions
    } else if let Some(captures) = WAV_REGEX.captures(line) {
        let idx = from_base36(captures.name("idx").unwrap().as_str().chars())
            .or_else(|_| Err(BMSImportError::NumericFormatError))?;
        let path = captures.name("path").unwrap().as_str();
        return Ok(Some(BMSCommand::WAVResource {
            idx,
            path: path.to_string(),
        }));
    //Capture song title
    } else if let Some(captures) = HEADER_TITLE_REGEX.captures(line) {
        let title = captures.name("title").unwrap().as_str().to_string();
        return Ok(Some(BMSCommand::SongInfo(BMSSongInfo::Title(title))));
    //Capture song BPM
    } else if let Some(captures) = BPM_REGEX.captures(line) {
        let bpm = f32::from_str(captures.name("bpm").unwrap().as_str())
            .or_else(|_| Err(BMSImportError::NumericFormatError))?;
        return Ok(Some(BMSCommand::SongInfo(BMSSongInfo::BPM(bpm))));
    }
    Ok(None)
}

fn push_indices_from_str_to_arglist(indices_str: &str, args: &mut Vec<u32>, args_cnt: &mut usize) -> Result<(), BMSImportError> {
    let mut a_iter = indices_str.chars().step_by(2);
    let mut b_iter = indices_str.chars().skip(1).step_by(2);
    while let (Some(a), Some(b)) = (a_iter.next(), b_iter.next()) {
        let b36num_str = [a, b];
        let num = from_base36(b36num_str.iter().cloned())
            .or_else(|_| Err(BMSImportError::InvalidBase36Format))?;
        args.push(num);
        *args_cnt += 1;
    }
    Ok(())
}

fn from_base36<'a, I>(numstr: I) -> Result<u32, ()> where I: IntoIterator<Item = char> {
    let mut v = 0;
    let iter = numstr.into_iter();
    for c in iter {
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