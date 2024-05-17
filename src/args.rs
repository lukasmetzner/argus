use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub project_path: String,

    #[arg(long, default_value_t = false)]
    pub pcap: bool,
}
