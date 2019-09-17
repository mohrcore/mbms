extern crate rand;

mod player;

use crate::util::pair_diff;

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
    pub command_cnt: Vec<usize>,
    pub commands: Vec<ChannelCommand>,
    pub measure_sets: Vec<MeasureCommandSet>,
}

impl CBMS {
    pub fn new() -> Self {
        Self {
            command_cnt: Vec::new(),
            commands: Vec::new(),
            measure_sets: Vec::new(),
        }
    }
    pub fn iter<'bms>(&'bms self) -> CBMSIterator<'bms> {
        CBMSIterator::new(&self)
    }
    pub fn iter_from_bar<'bms>(&'bms self, bar: usize) -> Result<CBMSIterator<'bms>, CBMSError> {
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
            bms: self,
            idata: self.iter_data_from_bar(bar),
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

pub struct CBMSIterator<'bms> {
    bms: &'bms CBMS,
    idata: CBMSIteratorData,
}

impl<'bms> CBMSIterator<'bms> {
    pub fn new(bms: &'bms CBMS) -> Self {
        Self {
            bms,
            idata: CBMSIteratorData {
                current_set: 0,
                current_cmd_pos: 0,
                current_cmd_cnt_pos: 0,
            }
        }
    }
}

impl<'whatevs> CBMSIterator<'whatevs> {
    pub fn iterate<'bms>(bms: &'bms CBMS, idata: &mut CBMSIteratorData) -> Option<(&'bms [ChannelCommand], u32, f32)> {
        //Return None if no more commands are avaible to pull
        if idata.current_set > bms.measure_sets.len() - 1 { return None; }
        //Jump to next command set if all commands from the current set were already pulled and if no more commands are avaible to pull return None 
        while idata.current_cmd_cnt_pos >= bms.measure_sets[idata.current_set].command_cnt_idx.1 {
            idata.current_set += 1;
            if idata.current_set >= bms.measure_sets.len() - 1 { return None; }
        }
        let measure_set = bms.measure_sets[idata.current_set];
        //Obtain command count and commands
        let cmd_cnt = bms.command_cnt[idata.current_cmd_cnt_pos];
        let cmds = &bms.commands[idata.current_cmd_pos .. idata.current_cmd_pos + cmd_cnt];
        //Calculate bar progress
        let measure_progress = (idata.current_cmd_cnt_pos - measure_set.command_cnt_idx.0) as f32 / (pair_diff(measure_set.command_cnt_idx)) as f32;
        //Move command count array cursor and command array cursor
        idata.current_cmd_cnt_pos += 1;
        idata.current_cmd_pos += cmd_cnt;
        //Return commands
        Some((cmds, measure_set.measure, measure_progress))
    }
}

impl<'bms> Iterator for CBMSIterator<'bms> {
    type Item = (&'bms [ChannelCommand], u32, f32);
    fn next(&mut self) -> Option<(&'bms [ChannelCommand], u32, f32)> {
        CBMSIterator::iterate(self.bms, &mut self.idata)
    }
}