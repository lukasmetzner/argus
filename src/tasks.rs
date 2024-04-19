use std::{
    fs,
    io::{BufReader, Read, Write},
    path::Path,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use ssh2::Session;
use tracing::{debug, info};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Bash {
    pub name: String,
    pub command: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct BashScript {
    pub name: String,
    pub script: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct File {
    pub name: String,
    pub source: Box<Path>,
    pub destination: Box<Path>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Task {
    Bash(Bash),
    BashScript(BashScript),
    File(File),
}

impl Task {
    pub fn run(&self, sess: &mut Session) -> Result<i32> {
        match self {
            Self::Bash(bash) => bash_command(bash, sess),
            Self::BashScript(b_script) => bash_script(b_script, sess),
            Self::File(file) => push_file(file, sess),
        }
    }
}

fn push_file(file: &File, sess: &mut Session) -> Result<i32> {
    info!("--- {} ---", file.name);

    let file_io = fs::OpenOptions::new().read(true).open(&file.source)?;
    let mut buf_reader = BufReader::new(file_io);

    let sftp = sess.sftp()?;
    let mut sftp_file = sftp.create(&file.destination)?;

    let mut buffer = vec![0; 8192]; // A buffer of 8 KB

    loop {
        let len = buf_reader.read(&mut buffer)?;
        if len == 0 {
            break; // End of file reached
        }
        sftp_file.write_all(&buffer[..len])?;
    }

    sftp_file.flush()?;

    Ok(0)
}

fn remote_exec(command: &str, sess: &mut Session) -> Result<i32> {
    let mut channel = sess.channel_session()?;
    channel.exec(command)?;

    let mut output = String::new();
    channel.read_to_string(&mut output)?;

    output.lines().for_each(|line| debug!("{line}"));

    channel.wait_close()?;

    let exit_status = channel.exit_status()?;

    Ok(exit_status)
}

fn bash_command(bash: &Bash, sess: &mut Session) -> Result<i32> {
    info!("--- {} ---", bash.name);
    let exit_status = remote_exec(&bash.command, sess)?;
    Ok(exit_status)
}

fn bash_script(b_script: &BashScript, sess: &mut Session) -> Result<i32> {
    info!("--- {} ---", b_script.name);

    let exit_codes = b_script
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
