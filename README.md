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

- Private and reliable communication channels.
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
$ shami-rs -i 0 -n ./net_config.json -c 1 --input <INPUT>

-- For Party 1:
$ shami-rs -i 1 -n ./net_config.json -c 1 --input <INPUT>

-- For Party 2:
$ shami-rs -i 2 -n ./net_config.json -c 1 --input <INPUT>
```

It is important to mention that the parties are indexed in such a way that the first index is 0.
Also, they are indexed consecutively.

### Configuration

The configuration of the network for the execution of the protocol is written in a JSON format.

The following file is an example of the configuration JSON for an execution of three parties:

```json
{
  "base_port": 5000,
  "timeout": 1000,
  "sleep_time": 500,
  "peer_ips": [
    "127.0.0.1",
    "127.0.0.1",
    "127.0.0.1"
  ]
}
```

The `base_port`, is the port that will be used as a base to compute the actual port in which the party will be listening to.
For a party with index `i`, the listening port is `base_port + i`. The `timeout` is the number of ***milliseconds*** that
a party will repeatedly try to connect to another party. If the timeout is reached, the application returns an error.
The `sleep_time` is the number of ***milliseconds*** that a party will wait before trying to connect again with another
party in case the connection is not successful. And finally, the `peer_ips` is the list of IPs for all the
peers engaged in the protocol. In this case, the array is specified in such a way that the party with index `i` has
IP `peer_ips[i]`.

> [!WARNING]
> All parties must have the same JSON configuration file.

> [!NOTE]
> This repository came as a result of a learning project by @hdvanegasm.
