export const DEFAULT_CONFIG_TEMPLATE = `config_name = "Default Config"
server = ["tcp://127.0.0.1:6660"]
network_code = "replace_with_your_network"
network_secret = "replace_with_random_secret"
device_name = "desktop-node"

compress = true
rtx = true
fec = false
no_punch = false
no_nat = false
no_tun = false

# ip = "10.26.0.10"
# password = "optional_packet_password"
# tun_name = "vnt0"
# mtu = 1380
# tunnel_port = 0

udp_stun = ["stun.miwifi.com:3478"]
tcp_stun = ["stun.miwifi.com:3478"]
`
