extern crate rand;

mod player;

use crate::util::pair_diff;
use crate::bms::BMSTime;

use std::rc::*;

//use rand::Rng;

//TODO: Sorting?
//Can contriol flow be precompiled?
//Yes, it should be.
//2-stage compilation?

#[derive(Copy, Clone, Debug)]
pub enum CBMSError {
    BarOutOfRange,
    BarIsEmpty,
}

#[derive(Copy, Clone, Debug)]
pub struct MeasureCommandSet {
    pub measure: u32,
    pub command_cnt_idx: (usize, usize),
    pub commands_idx: (usize, usize),
}

#[derive(Copy, Clone, Debug)]
pub struct ChannelCommand {
    pub channel: u32,
    pub value: u32,
}

#[derive(Debug)]
pub struct CBMS {
    pub command_cnt: Rc<Vec<usize>>,
    pub commands: Vec<ChannelCommand>,
    pub measure_sets: Rc<Vec<MeasureCommandSet>>,
}

impl CBMS {
    pub fn new() -> Self {
        Self {
            command_cnt: Rc::new(Vec::new()),
            commands: Vec::new(),
            measure_sets: Rc::new(Vec::new()),
        }
    }
    pub fn iter(&self) -> CBMSIterator {
        CBMSIterator::new(&self)
    }
    pub fn iter_from_bar<>(&self, bar: usize) -> Result<CBMSIterator, CBMSError> {
        //Check wether the bar exists
        if bar >= self.measure_sets.len() { return Err(CBMSError::BarOutOfRange); }
        let mut bar_id = bar;
        while (bar as u32) < self.measure_sets[bar_id].measure {
            if bar_id == 0 { return Err(CBMSError::BarIsEmpty); }
            bar_id -= 1;
        }
        //Create iterator
        if (bar as u32) > self.measure_sets[bar_id].measure { return Err(CBMSError::BarIsEmpty); }
        Ok(CBMSIterator {
            measure_sets: Rc::clone(&self.measure_sets),
            command_cnt: Rc::clone(&self.command_cnt),
            current_set: bar,
            current_cmd_pos: self.measure_sets[bar].commands_idx.0,
            current_cmd_cnt_pos: self.measure_sets[bar].command_cnt_idx.0,
        })
    }
    pub fn iter_data_from_bar(&self, bar: usize) -> CBMSIteratorData {
        CBMSIteratorData {
            current_set: bar,
            current_cmd_pos: self.measure_sets[bar].commands_idx.0,
            current_cmd_cnt_pos: self.measure_sets[bar].command_cnt_idx.0,
        }
    }
    pub fn bar_count(&self) -> usize {
        let len = self.measure_sets.len();
        if len == 0 { return 0 };
        self.measure_sets[len - 1].measure as usize + 1
    }
    pub fn command(&self, idx: usize) -> Option<ChannelCommand> {
        self.commands.get(idx).map(|v| *v)
    }
}

//This fuckery is here only to make it possible to iterate over bms
//in the scope it's in using CBMSIterator::iterate
pub struct CBMSIteratorData {
    current_set: usize,
    current_cmd_pos: usize,
    current_cmd_cnt_pos: usize,
}

impl Default for CBMSIteratorData {
    fn default() -> Self{
        Self {
            current_set: 0,
            current_cmd_pos: 0,
            current_cmd_cnt_pos: 0,
        }
    }
}

pub struct CBMSIterator {
    current_set: usize,
    current_cmd_pos: usize,
    current_cmd_cnt_pos: usize,
    command_cnt: Rc<Vec<usize>>,
    measure_sets: Rc<Vec<MeasureCommandSet>>,
}

impl<'bms> CBMSIterator {
    pub fn new(bms: &'bms CBMS) -> Self {
        Self {
            current_set: 0,
            current_cmd_pos: 0,
            current_cmd_cnt_pos: 0,
            command_cnt: Rc::clone(&bms.command_cnt),
            measure_sets: Rc::clone(&bms.measure_sets),
        }
    }
}

impl CBMSIterator {
    pub fn flatten(self) -> CBMSFlatten {
        CBMSFlatten {
            idx: 0,
            range: (0 .. 0),
            bms_time: BMSTime::from(0.0),
            iter: self,
        }
    }
}

impl Iterator for CBMSIterator {
    type Item = (std::ops::Range<usize>, BMSTime);
    fn next(&mut self) -> Option<(std::ops::Range<usize>, BMSTime)> {
        //Return None if no more commands are avaible to pull
        if self.current_set > self.measure_sets.len() - 1 { return None; }
        //Jump to next command set if all commands from the current set were already pulled and if no more commands are avaible to pull return None 
        while self.current_cmd_cnt_pos >= self.measure_sets[self.current_set].command_cnt_idx.1 {
            self.current_set += 1;
            if self.current_set >= self.measure_sets.len() - 1 { return None; }
        }
        let measure_set = self.measure_sets[self.current_set];
        //Obtain command count and commands
        let cmd_cnt = self.command_cnt[self.current_cmd_cnt_pos];
        let cmd_range = self.current_cmd_pos .. self.current_cmd_pos + cmd_cnt;
        //Calculate bar progress
        let measure_progress = (self.current_cmd_cnt_pos - measure_set.command_cnt_idx.0) as f64 / (pair_diff(measure_set.command_cnt_idx)) as f64;
        //Move command count array cursor and command array cursor
        self.current_cmd_cnt_pos += 1;
        self.current_cmd_pos += cmd_cnt;
        //Return commands
        Some((
            cmd_range,
            BMSTime::from(measure_set.measure as f64 + measure_progress)))
    }
}

pub struct CBMSFlatten {
    idx: usize,
    range: std::ops::Range<usize>,
    bms_time: BMSTime,
    iter: CBMSIterator,
}

impl Iterator for CBMSFlatten {
    type Item = (usize, BMSTime);
    fn next(&mut self) -> Option<(usize, BMSTime)> {
        while !self.range.contains(&self.idx) {
            let rt = self.iter.next()?;
            self.range = rt.0;
            self.bms_time = rt.1;
            self.idx = self.range.start;
        }
        let cmd_idx = self.idx;
        self.idx += 1;
        Some((cmd_idx, self.bms_time))
    }
}