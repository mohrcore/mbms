use super::*;

pub struct CBMSPlayer<'bms> {
    cbms: &'bms CBMS,
}

impl<'b> CBMSPlayer<'b> {
    pub fn new(cbms: &'b CBMS) -> Self {
        Self {
            cbms,
        }
    }
}