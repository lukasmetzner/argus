use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;
use scrolls::Scroll;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{
    hosts::{exec_hosts, parse_hosts},
    scrolls::parse_scroll,
};

mod args;
mod hosts;
mod modules;
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

    let scrolls_root_path = root_path.join("scrolls");
    let scrolls_path = hosts
        .scrolls
        .iter()
        .rev()
        .map(|scroll_name| scrolls_root_path.join(scroll_name))
        .collect::<Vec<PathBuf>>();

    let scrolls: Vec<Scroll> = scrolls_path
        .iter()
        .map(|scroll_path| parse_scroll(scroll_path))
        .collect::<Result<Vec<Scroll>>>()?;

    hosts
        .hosts
        .into_par_iter()
        .map(|host| exec_hosts(host, scrolls.clone(), args.pcap))
        .collect::<Result<Vec<()>>>()?;

    Ok(())
}
