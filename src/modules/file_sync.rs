use std::{
    fs,
    io::{BufReader, Read, Write},
    path::Path,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use ssh2::Session;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FileSync {
    pub source: Box<Path>,
    pub destination: Box<Path>,
}

impl FileSync {
    pub fn exec(&self, sess: &mut Session, scroll_dir_path: &Path) -> Result<i32> {
        let file_path = scroll_dir_path.join(&self.source);
        let file_io = fs::OpenOptions::new().read(true).open(file_path)?;
        let mut buf_reader = BufReader::new(file_io);

        let sftp = sess.sftp()?;
        let mut sftp_file = sftp.create(&self.destination)?;

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
}
