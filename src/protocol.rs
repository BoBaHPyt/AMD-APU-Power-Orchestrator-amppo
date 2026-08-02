use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct DaemonReq {
    pub cmd: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DaemonResp {
    pub modename: String,
    pub hook: Option<String>
}
