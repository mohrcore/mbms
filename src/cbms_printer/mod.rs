use crate::cbms::*;

use crate::util::pair_diff;

pub fn print_cbms_bar(cbms: &CBMS, bar: usize, channels_beg: u32, channels_cnt: u32) -> Result<(), CBMSError> {
    let iter = cbms.iter_from_bar(bar)?;
    let bar_len = pair_diff(cbms.measure_sets[bar].command_cnt_idx);
    let mut d_vec: Vec<Option<u32>> = vec![None; bar_len * channels_cnt as usize];
    for (commands, measure, progress) in iter {
        if measure != bar as u32 { break; }
        let line = (progress * (bar_len as f32)).round() as usize;
        for command in commands {
            if command.channel < channels_beg || command.channel >= channels_beg + channels_cnt { continue; }
            d_vec[line * channels_cnt as usize + command.channel as usize - channels_beg as usize] = Some(command.value);
        } 
    }
    let mut i = bar_len - 1;
    loop {
        let mut s = "|".to_string();
        for j in 0 .. channels_cnt as usize {
            match d_vec[i * channels_cnt as usize + j] {
                None => s += "....|",
                Some(value) =>
                    if value != 0 {
                        s += &format!("{:04}|", value);
                    } else {
                        s += "....|";
                    },
            };
        }
        println!("{}", s);
        if i == 0 { break; }
        i -= 1;
    }
    let mut line_break_s = "-".to_string();
    let mut desc_s = "|".to_string();
    for channel in channels_beg .. channels_beg + channels_cnt {
        desc_s += &format!("{:04}|", channel);
        line_break_s += "-----";
    }
    println!("{}", line_break_s);
    println!("{}", desc_s);
    Ok(())
}