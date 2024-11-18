# shami-rs

This repository contains a very basic implementation of Shamir's secret-sharing
with semi-honest security for the honest majority setting in Rust. The implementation is
not intended to be production-ready. Instead, the implementation aims to be an
educational resource on how to implement Shamir secret-sharing from scratch.

The project is a CLI application that allows parties $P_1, P_2, \dots, P_n$ to compute
$x_1 \times \cdots \times x_n$ where $x_i$ is the input of party $P_i$. Currently, the
project runs using the IP of localhost, but the source code can be modified to run
in a distributed way.

This protocol is implemented using the Mersenne61 field, which means that all the
operations are performed in $\mathbb{Z}_p$ for $p = 2^{61} - 1$. The implementation
of the field arithmetic is done from scratch.

This project does not consider the following features yet:

- The network implementation does not consider delays in communication.
- There are not enough tests.
- The implementation is not performant.

This project is intended to be a learning resource for people who want to know the first
steps on implementing a network for an MPC protocol, as there is no (or very few) material on the topic.
Also, it presents a basic, non-performant implementation of the field arithmetic for those curious about it.

> [!NOTE]
> Contributions to improve this project are welcome and encouraged.

## How to run

The following block shows the output of the command `cargo run -- --help`.

```text
Implementation of a node to execute a Shamir secret-sharing protocol

Usage: shami-rs --id <ID> --net-config-file <NET_CONFIG_FILE> --corruptions <CORRUPTIONS> --input <INPUT>

Options:
  -i, --id <ID>                            ID of the current player
  -n, --net-config-file <NET_CONFIG_FILE>  Path to the network configuration file
  -c, --corruptions <CORRUPTIONS>          Number of corrupted parties
      --input <INPUT>                      The number you want to multiply
  -h, --help                               Print help
```

To run the application, you need to open multiple terminals and define the command-line inputs
accordingly. For example, suppose that you want to execute the protocol for three parties with one corruption.
Hence, you must open three different terminals and write the following commands for each terminal as follows:

```text
-- For Party 0:
$ shami-rs -i 0 -n ./net_config_p0.json -c 1 --input <INPUT>

-- For Party 1:
$ shami-rs -i 1 -n ./net_config_p1.json -c 1 --input <INPUT>

-- For Party 2:
$ shami-rs -i 2 -n ./net_config_p2.json -c 1 --input <INPUT>
```

It is important to mention that the parties are indexed in such a way that the first index is 0.
Also, they are indexed consecutively.

### Configuration

The configuration of the network for the execution of the protocol is written in a JSON format.

The following file is an example of the configuration JSON for the party with ID 0 for an execution of three parties:

```json
{
  "base_port": 5000,
  "timeout": 5000,
  "sleep_time": 500,
  "peer_ips": [
    "127.0.0.1",
    "127.0.0.1",
    "127.0.0.1"
  ],
  "server_cert": "./certs/server_cert_p0.crt",
  "priv_key": "./certs/priv_key_p0.pem",
  "trusted_certs": [
    "./certs/rootCA.crt"
  ]
}
```

The fields above are explained next:

- The `base_port`, is the port that will be used as a base to compute the actual port in which the party will be listening to.
For a party with index `i`, the listening port is `base_port + i`.
- The `timeout` is the number of ***milliseconds***
a party will repeatedly try to connect with another party. If the timeout is reached, the application returns an error.
- The `sleep_time` is the number of ***milliseconds*** that a party will wait before trying to connect again with another
party in case the connection is not successful.
- The `peer_ips` is the list of IPs for all the
peers engaged in the protocol. In this case, the array is specified in such a way that the party with index `i` has
IP `peer_ips[i]`.
- The `server_cert` is the certificate path for that node for secure communication.
- The `priv_key` is the file with the private key associated with the certificate in `server_cert`. This private key is used for secure communication.
- `trusted_certs` is a list of paths with trusted CA certificates. This is useful in executions where the certificates are self-signed.

> [!WARNING]
> Each party should have its configuration JSON file with the corresponding certificates and private keys.

### Generating self-signed certificates for local testing

The script `./generate_certs.sh` will help you to generate self-signed certificates to test the tool. To generate the certificates for
`N` parties, you should run the command

```text
bash ./generate_certs.sh <N>
```

Then the certificates will be generated in the `./certs/` folder. Remember to add `./certs/rootCA.crt` to the list of trusted certificates and generate the JSON with the private key and certificates accordingly to each party.

> [!NOTE]
> This repository came as a result of a learning project by @hdvanegasm.
