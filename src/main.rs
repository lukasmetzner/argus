use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;
use scrolls::Scroll;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{
    hosts::{exec_hosts, parse_hosts},
    scrolls::parse_scroll,
};

mod args;
mod hosts;
mod scrolls;
mod tasks;

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("LOG"))
        .init();

    let args = args::Args::parse();

    let root_path = Path::new(&args.project_path);
    let hosts = parse_hosts(root_path);

    let scrolls_path = hosts
        .scrolls
        .iter()
        .rev()
        .map(|scroll_name| PathBuf::from(format!("./scrolls/{scroll_name}/")))
        .collect::<Vec<PathBuf>>();

    let scrolls: Vec<Scroll> = scrolls_path
        .iter()
        .map(|scroll_path| parse_scroll(scroll_path))
        .collect::<Result<Vec<Scroll>>>()?;

    // Add identity to ssh agent
    let output = Command::new("ssh-add").output()?;
    let o_string = String::from_utf8(output.stdout)?;
    info!("{}", o_string);

    hosts
        .hosts
        .into_par_iter()
        .map(|host| exec_hosts(host, &scrolls))
        .collect::<Result<Vec<()>>>()?;

    Ok(())
}
