use std::{
    fs,
    io::Read,
    net::TcpStream,
    path::{Path, PathBuf},
    thread::{self, JoinHandle},
};

use anyhow::Result;
use pcap::{Capture, Device};
use serde::{Deserialize, Serialize};
use ssh2::Session;
use tracing::{debug, error, info};

use crate::scrolls::Scroll;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Host {
    pub host: String,
    pub pubkey_path: PathBuf,
    pub privkey_path: PathBuf,
    pub ssh_passphrase: Option<String>,
    pub user: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hosts {
    pub scrolls: Vec<String>,
    pub hosts: Vec<Host>,
}

pub fn parse_hosts(root_path: &Path) -> Hosts {
    debug!("{:?}", root_path);
    let hosts_path = root_path.join("hosts.yml");
    debug!("{:?}", hosts_path);
    let mut file = fs::File::open(hosts_path).expect("Could not open hosts.yaml!");
    let mut str_buf = String::new();
    file.read_to_string(&mut str_buf)
        .expect("Could not read hosts.yaml!");
    let hosts: Hosts = serde_yaml::from_str(&str_buf).expect("host.yaml could not be parsed!");
    hosts
}

pub fn exec_hosts(host: Host, scrolls: Vec<Scroll>, pcap: bool) -> Result<()> {
    info!("=========== {} ===========", &host.host);
    info!("Executing scrolls on host {}", &host.host);

    let host_c = host.host.clone();

    // Execution in seperate thread to stop capture after scrolls are finished
    let handle = thread::spawn(move || {
        _exec_hosts(host, scrolls).unwrap();
    });

    if pcap {
        capture(handle, host_c)?;
    } else {
        handle.join().unwrap();
    }

    Ok(())
}

fn _exec_hosts(host: Host, scrolls: Vec<Scroll>) -> Result<()> {
    let tcp = TcpStream::connect(format!("{}:22", &host.host))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    info!("Before Auth");

    let user = host.user.unwrap_or("root".to_string());
    sess.userauth_pubkey_file(
        &user,
        Some(&host.pubkey_path),
        &host.privkey_path,
        host.ssh_passphrase.as_deref(),
    )
    .unwrap();

    if !sess.authenticated() {
        panic!("Session not authenticated");
    }

    info!("Authenticated");

    for scroll in scrolls.iter().rev() {
        for task in scroll.tasks.iter() {
            let exit_status = match task.run(&mut sess) {
                Ok(it) => it,
                Err(err) => {
                    error!("Argus error occured in task: {:?}", err);
                    break;
                }
            };

            if exit_status > 0 {
                error!(
                    "Task in Scroll {} exited with error code {}",
                    scroll.name, exit_status
                );
                break;
            }
        }
    }

    Ok(())
}

fn capture(handle: JoinHandle<()>, host_c: String) -> Result<()> {
    let device = Device::lookup()?.expect("no device available");
    let mut cap = Capture::from_device(device)?.immediate_mode(true).open()?;

    cap.filter(format!("host {}", host_c).as_str(), true)?;

    let mut savefile = cap.savefile(format!("{}.pcap", host_c))?;

    loop {
        if handle.is_finished() {
            break;
        }

        let packet = match cap.next_packet() {
            Ok(it) => it,
            Err(_) => break,
        };

        savefile.write(&packet);
    }
    Ok(())
}
