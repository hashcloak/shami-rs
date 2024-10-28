# shami-rs

This repository contains a very basic implementation of Shamir's secret-sharing
with semi-honest security for the honest majority setting in Rust. The implementation is
not intended to be production-ready. Instead, the implementation aims to be an
educational resource on how to implement Shamir secret-sharing from scratch.

The project is a CLI application that allows parties $P_1, P_2, \dots, P_n$ to compute
$x_1 \times \cdots \times x_n$ where $x_i$ is the input of party $P_i$. Currently, the
project runs using the IP of localhost, but the source code can be modified to run
in a distributed way.

This project does not consider the following features yet:

- Private and reliable communication channels.
- The network implementation does not consider delays in communication.
- There are not enough tests.
- The implementation is not performant.
- The parameters of the application are not configurable without modifying the source
  code.

## How to run

The following block shows the output of the command `cargo run -- --help`.

```text
Implementation of a player connected to a network

Usage: shami-rs --id <ID> --n-parties <N_PARTIES> --corruptions <CORRUPTIONS> --input <INPUT>

Options:
  -i, --id <ID>                    ID of the current player
  -n, --n-parties <N_PARTIES>      Number of parties participating in the protocol
  -c, --corruptions <CORRUPTIONS>  Number of corrupted parties
      --input <INPUT>              The number you want to multiply
  -h, --help                       Print help
```

> [!NOTE]
> This repository came as a result of a learning project by @hdvanegasm.
