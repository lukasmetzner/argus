use serde::{Deserialize, Serialize};
use ssh2::Session;

use anyhow::Result;

use super::utils::remote_exec;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Bash {
    pub command: String,
}

impl Bash {
    pub fn exec(&self, sess: &mut Session) -> Result<i32> {
        let exit_status = remote_exec(&self.command, sess)?;
        Ok(exit_status)
    }
}
