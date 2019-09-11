pub struct WBMS {
    pub indices: Vec<(f64,  WCommand)>
}

pub enum WCommand {
    Channel(u32, u32),
}

pub struct WBMSWorkspace {
    wbms: WBMS,
}

