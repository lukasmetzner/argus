use std::io::Read;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use ssh2::{Channel, Session};
use tracing::{error, info};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Bash {
    name: String,
    command: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct BashScript {
    name: String,
    script: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Task {
    Bash(Bash),
    BashScript(BashScript),
}

impl Task {
    pub fn run(&self, sess: &mut Session) -> Result<()> {
        match self {
            Self::Bash(bash) => bash_command(bash, sess),
            Self::BashScript(b_script) => bash_script(b_script, sess),
        }
    }
}

fn remote_cmd(command: &String, sess: &mut Session) -> Result<()> {
    let mut channel = sess.channel_session()?;
    channel.exec(&command)?;

    let mut output = String::new();
    channel.read_to_string(&mut output)?;

    for line in output.lines() {
        info!("{}", line);
    }

    channel.wait_close()?;

    let exit_status = channel.exit_status()?;
    if exit_status > 0 {
        error!("exited with error code: {:}", exit_status);
    }
    Ok(())
}

fn bash_command(bash: &Bash, sess: &mut Session) -> Result<()> {
    info!("--- {} ---", bash.name);
    remote_cmd(&bash.command, sess)?;
    Ok(())
}

fn bash_script(b_script: &BashScript, sess: &mut Session) -> Result<()> {
    info!("--- {} ---", b_script.name);

    for command in b_script.script.iter() {
        remote_cmd(command, sess)?;
    }

    Ok(())
}
