use std::{
    fs,
    io::Read,
    net::TcpStream,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use ssh2::{Channel, Session};
use tracing::{error, info};

type Scroll = Vec<Task>;


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    project_path: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Bash {
    name: String,
    command: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum Task {
    Bash(Bash),
}

impl Task {
    pub fn run(&self, channel: &mut Channel) -> Result<()> {
        match self {
            Self::Bash(bash) => bash_command(bash, channel),
        }
    }
}

fn bash_command(bash: &Bash, channel: &mut Channel) -> Result<()> {
    info!("--- {} ---", bash.name);
    channel.exec(&bash.command)?;

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

#[derive(Serialize, Deserialize, Debug)]
struct Host {
    host: String,
    user: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Hosts {
    scrolls: Vec<String>,
    hosts: Vec<Host>,
    pubkey_path: String,
}

fn parse_hosts(root_path: &Path) -> Hosts {
    let mut file = fs::File::open(root_path.join("hosts.yml")).expect("Could not open hosts.yaml!");
    let mut str_buf = String::new();
    file.read_to_string(&mut str_buf)
        .expect("Could not read hosts.yaml!");
    let hosts: Hosts = serde_yaml::from_str(&str_buf).expect("host.yaml could not be parsed!");
    hosts
}

fn parse_scroll(scroll_dir_path: &PathBuf) -> Scroll {
    let mut file =
        fs::File::open(scroll_dir_path.join("main.yml")).expect("Could not open hosts.yaml!");
    let mut str_buf = String::new();
    file.read_to_string(&mut str_buf)
        .expect("Could not read hosts.yaml!");
    let tasks: Scroll = serde_yaml::from_str(&str_buf).unwrap();
    tasks
}

fn exec_hosts(host: Host, scrolls: &Vec<Scroll>) -> Result<()> {
    info!("=========== {} ===========", &host.host);
    info!("Executing scrolls on host {}", host.host);
    let tcp = TcpStream::connect(format!("{}:22", &host.host))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    let user = host.user.unwrap_or("root".to_string());
    sess.userauth_agent(&user)?;

    if !sess.authenticated() {
        panic!("Session not authenticated");
    }

    for scroll in scrolls {
        for task in scroll {
            let mut scroll_channel = sess.channel_session()?;
            task.run(&mut scroll_channel)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let args = Args::parse();

    let root_path = Path::new(&args.project_path);
    let hosts = parse_hosts(&root_path);

    let scrolls_path = fs::read_dir(root_path.join("scrolls"))?
        .into_iter()
        .map(|path| path.unwrap().path())
        .filter(|path| path.is_dir())
        .collect::<Vec<PathBuf>>();

    let scrolls: Vec<Scroll> = scrolls_path
        .iter()
        .map(|scroll_path| parse_scroll(scroll_path))
        .collect();

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
