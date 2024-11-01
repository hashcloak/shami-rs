mod math;
mod mpc;
mod net;

use clap::Parser;
use math::mersenne61::Mersenne61;
use mpc::{reconstruct_secret, run_multiply_protocol, share::ShamirShare};
use net::{Network, NetworkConfig, Packet};
use std::{error::Error, path::Path};

/// Implementation of a node to execute a Shamir secret-sharing protocol.
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    /// ID of the current player.
    #[arg(short, long)]
    id: usize,
    /// Path to the network configuration file.
    #[arg(short, long)]
    net_config_file: String,
    /// Number of corrupted parties.
    #[arg(short, long)]
    corruptions: usize,
    /// The number you want to multiply.
    #[arg(long)]
    input: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut log_builder = env_logger::Builder::new();
    log_builder.filter_level(log::LevelFilter::Debug).init();

    let args = Args::parse();

    let net_config = NetworkConfig::new(Path::new(&args.net_config_file))?;
    let n_parties = net_config.peer_ips.len();

    // Create the network for communication.
    let mut network = Network::create(args.id, net_config)?;

    // Compute random shares to send to the other parties.
    let mut rng = rand::thread_rng();
    let own_shares = mpc::compute_shamir_share(
        &Mersenne61::from(args.input),
        n_parties,
        args.corruptions,
        &mut rng,
    );

    log::debug!("the shares of the inputs are {:?}", own_shares);

    // Send the share to all the parties.
    log::info!("sending the shares of the input to the other parties");
    for (i, share) in own_shares.iter().enumerate() {
        log::debug!("sending share to party {i}: {:?}", share);
        let share_bytes = bincode::serialize(&share)?;
        let share_packet = Packet::new(share_bytes);
        network.send_to(&share_packet, i)?;
    }
    let mut shares = Vec::with_capacity(n_parties);

    // Receive the shares from all the parties.
    log::info!("receiving shares of the inputs from other parties");
    for i in 0..n_parties {
        let packet = network.recv_from(i)?;
        let share: ShamirShare<Mersenne61> = bincode::deserialize(packet.as_slice())?;
        log::debug!("received share from party {i}: {:?}", share);
        shares.push(share);
    }

    log::debug!("the received shares are {:?}", shares);

    log::info!("running multiplication protocol");
    let mut mult_share = run_multiply_protocol(
        &shares[0],
        &shares[1],
        n_parties,
        args.corruptions,
        &mut rng,
        &mut network,
    )?;
    for share in shares.iter().skip(2) {
        mult_share = run_multiply_protocol(
            &mult_share,
            share,
            n_parties,
            args.corruptions,
            &mut rng,
            &mut network,
        )?;
    }

    // Open the secret by sending the shares to other parties.
    log::info!("sending the shares of the result to other parties");
    log::debug!("the share of party {} is {:?}", args.id, mult_share);
    let mult_share_bytes = bincode::serialize(&mult_share)?;
    let mult_share_packet = Packet::new(mult_share_bytes);
    network.send(&mult_share_packet)?;

    // Receive the shares from the Network
    log::info!("receiving the shares of the result from other parties");
    let mut mult_shares_remote = Vec::with_capacity(n_parties);
    for i in 0..n_parties {
        let packet = network.recv_from(i)?;
        let share: ShamirShare<Mersenne61> = bincode::deserialize(packet.as_slice())?;
        log::debug!("received share from party {i}: {:?}", share);
        mult_shares_remote.push(share);
    }

    log::debug!("multiplications shares: {:?}", mult_shares_remote);

    let mult_result = reconstruct_secret(mult_shares_remote);

    log::info!("the multiplication result is: {:?}", mult_result);

    network.close()?;

    Ok(())
}
