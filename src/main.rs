mod math;
mod mpc;
mod net;

use clap::Parser;
use math::mersenne61::Mersenne61;
use mpc::{reconstruct_secret, run_multiply_protocol, share::ShamirShare};
use net::{Network, Packet};
use std::time::Duration;
use std::{error::Error, thread};

/// Implementation of a player connected to a network.
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    /// ID of the current player.
    #[arg(short, long)]
    id: usize,

    /// Number of parties participating in the protocol
    #[arg(short, long)]
    n_parties: usize,

    /// Number of corrupted parties.
    #[arg(short, long)]
    corruptions: usize,

    /// The number you want to multiply.
    #[arg(long)]
    input: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::parse();

    // Create the network for communication.
    let mut network = Network::create(args.id, args.n_parties)?;

    // Compute random shares to send to the other parties.
    let mut rng = rand::thread_rng();
    let own_shares = mpc::compute_shamir_share(
        &Mersenne61::from(args.input),
        args.n_parties,
        args.corruptions,
        &mut rng,
    );

    log::debug!("the shares of the inputs are {:?}", own_shares);

    // Send the share to all the parties.
    log::info!("sending the shares of the input to the other parties");
    for (i, share) in own_shares.iter().enumerate() {
        let share_bytes = bincode::serialize(&share)?;
        let share_packet = Packet::new(share_bytes);
        network.send_to(&share_packet, i)?;
    }

    // Receive the shares from all the parties.
    log::info!("receiving shares fo the inputs from other parties");
    let mut shares = Vec::with_capacity(args.n_parties);
    for i in 0..args.n_parties {
        let packet = network.recv_from(i)?;
        let share: ShamirShare<Mersenne61> = bincode::deserialize(packet.as_slice())?;
        shares.push(share);
    }

    log::debug!("the received shares are {:?}", shares);

    log::info!("running multiplication protocol");
    let mut mult_share = run_multiply_protocol(
        &shares[0],
        &shares[1],
        args.n_parties,
        args.corruptions,
        &mut rng,
        &mut network,
    )?;
    for share in shares.iter().rev().take(args.n_parties - 2) {
        mult_share = run_multiply_protocol(
            &mult_share,
            share,
            args.n_parties,
            args.corruptions,
            &mut rng,
            &mut network,
        )?;
    }

    // Open the secret by sending the shares to other parties.
    log::info!("sending the shares of the result to other parties");
    let mult_share_bytes = bincode::serialize(&mult_share)?;
    let mult_share_packet = Packet::new(mult_share_bytes);
    network.send(&mult_share_packet)?;

    // Receive the shares from the Network
    log::info!("receiving the shares of the result from other parties");
    let mut mult_shares_remote = Vec::with_capacity(args.n_parties);
    for i in 0..args.n_parties {
        let packet = network.recv_from(i)?;
        let share: ShamirShare<Mersenne61> = bincode::deserialize(packet.as_slice())?;
        mult_shares_remote.push(share);
    }

    let mult_result = reconstruct_secret(mult_shares_remote);

    log::info!("the multiplication result is: {:?}", mult_result);

    Ok(())
}
