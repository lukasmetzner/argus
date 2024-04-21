use std::{fs, io::Read, net::TcpStream, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use ssh2::Session;
use tracing::{debug, error, info};

use crate::scrolls::Scroll;
#[derive(Serialize, Deserialize, Debug)]
pub struct Host {
    pub host: String,
    pub user: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hosts {
    pub scrolls: Vec<String>,
    pub hosts: Vec<Host>,
    pub pubkey_path: String,
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

pub fn exec_hosts(host: Host, scrolls: &[Scroll]) -> Result<()> {
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
