# Argus

Ansible inspired automation tool written in Rust.

# Installation
```bash
git clone https://github.com/lukasmetzner/argus.git
cd argus
cargo install --path .
```

# Usage

Argus project file structure
``` bash
├── hosts.yml
└── scrolls
    ├── init-setup
    │   └── main.yml
    └── install-devtools
        └── main.yml
```

- **hosts.yml:** Contains information about the hosts and the execution order of scrolls
- **scrolls/init-setup/main.yml:** Contains the tasks and execution order of a single scroll

```bash
argus --project-path .
# OR
argus -p examples/vm-init # Fill in hosts.yml first
```