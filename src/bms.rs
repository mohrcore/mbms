#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct BMSTime(f64);

//This should be fixed!
//BPM can change not only per-bar!
//(bar number, bpm, beats per bar - usually 4 which would imply 4/4 meter), elapsed tim
pub type BMSTimings = Vec<(BMSTime, f32, usize)>;

impl BMSTime {
    pub fn from_absolute_time(mut atime: f64, timings: &BMSTimings) -> BMSTime {
        let mut ctime = BMSTime(0.0);
        let mut idx = 0;
        while idx < timings.len() {
            let (time, bpm, beatsno) = timings[idx];
            let (section_time, section_length) = if idx + 1 < timings.len() {
                let section_length = timings[idx + 1].0 - time;
                (section_length.0 * beatsno as f64 * 60.0 / bpm as f64, section_length)
            } else { (-1.0, BMSTime(0.0)) };
            if atime > section_time && section_time > 0.0 {
                atime -= section_time;
                ctime += section_length;
            } else {
                let beats = atime * bpm as f64 / 60.0;
                ctime += BMSTime(beats / beatsno as f64);
                break;
            }
            idx += 1;
        }
        ctime
    }
    pub fn to_absolute_time_and_hint(&self, timings: &BMSTimings, hint: Option<BMSAbsoluteTimingHint>) -> (f64, BMSAbsoluteTimingHint) {
        let mut ctime = match hint {
            None => 0.0,
            Some(hint) => hint.last_elapsed_time,
        };
        let mut cbar = self.clone();
        let mut idx = match hint {
            None => 0,
            Some(hint) => hint.last_idx,
        };
        while idx < timings.len() {
            let (bar, bpm, beatsno) = timings[idx];
            let (section_time, section_length) = if idx + 1 < timings.len() {
                let section_length = timings[idx + 1].0 - bar;
                (section_length.0 * beatsno as f64 * 60.0 / bpm as f64, section_length)
            } else { (-1.0, BMSTime(0.0)) };
            if section_length < cbar && section_time > 0.0 {
                cbar -= section_length;
                ctime += section_time;
            } else {
                let t = cbar.0 * beatsno as f64 * 60.0 / bpm as f64;
                ctime += t;
                break;
            }
            idx += 1;
        }
        (
            ctime,
            BMSAbsoluteTimingHint {
                last_elapsed_time: ctime,
                last_idx: idx,
        })
    }
    pub fn to_absolute_time(&self, timings: &BMSTimings, hint: Option<BMSAbsoluteTimingHint>) -> f64 {
        self.to_absolute_time_and_hint(timings, hint).0
    }
    pub fn bar(&self) -> usize {
        self.0.floor() as usize
    }
    pub fn prog(&self) -> f64 {
        self.0.fract()
    }
}

impl std::ops::Add for BMSTime {
    type Output = Self;
    fn add(self, b: Self)-> Self {
        Self(self.0 + b.0)
    }
}
impl std::ops::AddAssign for BMSTime {
    fn add_assign(&mut self, b: Self) {
        self.0 += b.0;
    }
}
impl std::ops::Sub for BMSTime {
    type Output = Self;
    fn sub(self, b: Self)-> Self {
        Self(self.0 - b.0)
    }
}
impl std::ops::SubAssign for BMSTime {
    fn sub_assign(&mut self, b: Self) {
        self.0 -= b.0;
    }
}
impl From<f64> for BMSTime {
    fn from(v: f64) -> Self {
        Self(v)
    }
}
impl From<BMSTime> for f64 {
    fn from(v: BMSTime) -> f64 {
        v.0
    }
}

#[derive(Copy, Clone)]
pub struct BMSAbsoluteTimingHint {
    last_elapsed_time: f64,
    last_idx: usize,
}

#[cfg(test)]
#[test]
fn test_bms_time_from_absolute_time_1() {
    let timings: BMSTimings = vec![
        (0.0.into(), 120.0, 4),
        (8.5.into(), 240.0, 4),
    ];
    let bmstime = BMSTime::from_absolute_time(10.0, &timings);
    assert_eq!(bmstime, 5.0.into());
}

#[cfg(test)]
#[test]
fn test_bms_time_from_absolute_time_2() {
    let timings: BMSTimings = vec![
        (0.0.into(), 120.0, 4),
        (8.5.into(), 240.0, 4),
    ];
    let bmstime = BMSTime::from_absolute_time(20.0, &timings);
    assert_eq!(bmstime, 11.5.into());
}

#[cfg(test)]
#[test]
fn test_bms_time_from_absolute_time_3() {
    let timings: BMSTimings = vec![
        (0.0.into(), 120.0, 4),
        (8.0.into(), 240.0, 4),
        (9.0.into(), 120.0, 3),
    ];
    let bmstime = BMSTime::from_absolute_time(20.0, &timings);
    assert_eq!(bmstime, 11.0.into());
}

#[cfg(test)]
#[test]
fn test_bms_time_to_absolute_time_1() {
    let timings: BMSTimings = vec![
        (0.0.into(), 120.0, 4),
        (8.0.into(), 240.0, 4),
    ];
    let atime = BMSTime::from(4.5).to_absolute_time(&timings, None);
    assert_eq!(atime, 9.0);
}

#[cfg(test)]
#[test]
fn test_bms_time_to_absolute_time_2() {
    let timings: BMSTimings = vec![
        (0.0.into(), 120.0, 4),
        (8.0.into(), 240.0, 4),
    ];
    let atime = BMSTime::from(8.0).to_absolute_time(&timings, None);
    assert_eq!(atime, 16.0);
}

#[cfg(test)]
#[test]
fn test_bms_time_to_absolute_time_3() {
    let timings: BMSTimings = vec![
        (0.0.into(), 120.0, 4),
        (8.0.into(), 240.0, 4),
    ];
    let atime = BMSTime::from(9.5).to_absolute_time(&timings, None);
    assert_eq!(atime, 17.5);
}