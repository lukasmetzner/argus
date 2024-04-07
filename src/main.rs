use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;
use scrolls::Scroll;
use tracing::info;

use crate::{hosts::{exec_hosts, parse_hosts}, scrolls::parse_scroll};

mod args;
mod tasks;
mod scrolls;
mod hosts;

fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let args = args::Args::parse();

    let root_path = Path::new(&args.project_path);
    let hosts = parse_hosts(&root_path);

    let scrolls_path = fs::read_dir(root_path.join("scrolls"))?
        .into_iter()
        .map(|path| {
            let path = path?.path();
            Ok(path)
        })
        .collect::<Result<Vec<PathBuf>>>()?
        .into_iter()
        .filter(|path| path.is_dir())
        .collect::<Vec<PathBuf>>();

    let scrolls: Vec<Scroll> = scrolls_path
        .iter()
        .map(|scroll_path| parse_scroll(scroll_path))
        .collect::<Result<Vec<Scroll>>>()?;

    // Add identity to ssh agent
    let output = Command::new("ssh-add").arg(hosts.pubkey_path).output()?;
    let o_string = String::from_utf8(output.stdout)?;
    info!("{}", o_string);

    hosts
        .hosts
        .into_par_iter()
        .for_each(|host| exec_hosts(host, &scrolls).unwrap());

    Ok(())
}
