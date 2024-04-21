use std::io::Read;

use anyhow::Result;
use ssh2::Session;
use tracing::debug;

pub fn remote_exec(command: &str, sess: &mut Session) -> Result<i32> {
    let mut channel = sess.channel_session()?;
    channel.exec(command)?;

    let mut output = String::new();
    channel.read_to_string(&mut output)?;

    output.lines().for_each(|line| debug!("{line}"));

    channel.wait_close()?;

    let exit_status = channel.exit_status()?;

    Ok(exit_status)
}
