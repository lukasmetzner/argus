# Set in terraform.tfvars
variable "hcloud_token" {
  sensitive = true # Requires terraform >= 0.14
}

variable "ssh_key_id" {
}

# Configure the Hetzner Cloud Provider
provider "hcloud" {
  token = var.hcloud_token
}

# Create a server
# Create a new server running debian
resource "hcloud_server" "node1" {
  count		     = 4
  name         = "argus-tests-${count.index}"
  image        = "ubuntu-22.04"
  server_type  = "cpx11"
  location     = "fsn1"
  ssh_keys 	= [var.ssh_key_id]
  public_net {
    ipv4_enabled = true
    ipv6_enabled = true
  }

  provisioner "local-exec" {
    command = "echo ${self.ipv4_address} >> ips"
  }
  
  provisioner "local-exec" {
    when    = destroy
    command = "rm -f ips"
  }
}
