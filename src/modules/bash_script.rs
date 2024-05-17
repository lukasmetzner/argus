use anyhow::Result;
use serde::{Deserialize, Serialize};
use ssh2::Session;

use super::utils::remote_exec;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct BashScript {
    pub script: Vec<String>,
}

impl BashScript {
    pub fn exec(&self, sess: &mut Session) -> Result<i32> {
        let exit_codes = self
            .script
            .iter()
            .map(|cmd| remote_exec(cmd, sess)) // #TODO: Check
            .collect::<Result<Vec<i32>>>()?
            .into_iter()
            .collect::<Vec<i32>>();

        let failed = exit_codes.iter().filter(|f| f > &&0).collect::<Vec<&i32>>();

        if !failed.is_empty() {
            return Ok(failed[0].to_owned());
        }

        Ok(0)
    }
}
