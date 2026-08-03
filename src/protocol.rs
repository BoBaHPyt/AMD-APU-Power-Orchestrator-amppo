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

#[derive(Serialize, Debug)]
pub struct CliResp {
    pub text: String,
    pub alt: String
}
