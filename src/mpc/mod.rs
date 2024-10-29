use rand::Rng;
use share::ShamirShare;

use crate::{
    math::{
        lagrange::{compute_lagrange_basis, interpolate_polynomial_at},
        FiniteField, Polynomial,
    },
    net::{Network, Packet},
};

pub mod share;

/// Computes the shamir shares of a secret.
pub fn compute_shamir_share<T, R>(
    secret: &T,
    n_parties: usize,
    threshold: usize,
    rng: &mut R,
) -> Vec<ShamirShare<T>>
where
    T: FiniteField,
    R: Rng,
{
    log::info!("computing Shamir share of secret value: {:?}", secret);
    let mut rand_poly = Polynomial::random(threshold, rng);
    rand_poly[0] = secret.clone();

    log::debug!("using polynomial to share the secret: {:?}", rand_poly);

    let mut shares = Vec::with_capacity(n_parties);

    for idx in 1..n_parties + 1 {
        let evaluation_point = T::from(idx as u64);
        let evaluation = rand_poly.evaluate(&evaluation_point);
        shares.push(ShamirShare::new(evaluation, threshold));
    }
    shares
}

/// Reconstructs a secret given its shares.
pub fn reconstruct_secret<T>(shares: Vec<ShamirShare<T>>) -> T
where
    T: FiniteField,
{
    let alphas: Vec<T> = (1..shares.len() + 1)
        .map(|idx| T::from(idx as u64))
        .collect();
    let share_values: Vec<T> = shares.into_iter().map(|share| share.value).collect();
    interpolate_polynomial_at(share_values, alphas, &T::ZERO)
}

/// Run the protocol to multiply `a` and `b`, where `a` and `b` are already secret shared.
pub fn run_multiply_protocol<T, R>(
    a: &ShamirShare<T>,
    b: &ShamirShare<T>,
    n_parties: usize,
    threshold: usize,
    rng: &mut R,
    network: &mut Network,
) -> anyhow::Result<ShamirShare<T>>
where
    T: FiniteField,
    R: Rng,
{
    let h = a.multiply(b);
    let h_own_shares = compute_shamir_share(&h.value, n_parties, threshold, rng);

    // Send product shares to other parties
    log::info!("sending shares of the product share of degree 2 * d");
    for (i, share) in h_own_shares.iter().enumerate() {
        let share_bytes = bincode::serialize(&share)?;
        network.send_to(&Packet::new(share_bytes), i)?;
    }

    log::debug!("sending own shares of h(i): {:?}", h_own_shares);

    // Get the shares from other parties.
    log::info!("receiving shares of the product from other parties");
    let mut h_shares = Vec::with_capacity(n_parties);
    for i in 0..n_parties {
        let share_packet = network.recv_from(i)?;
        let share: ShamirShare<T> = bincode::deserialize(share_packet.as_slice())?;
        h_shares.push(share);
    }

    log::debug!("received shares of h(i): {:?}", h_shares);

    // Compute recombination vector.
    let basis = compute_lagrange_basis(
        (1..n_parties + 1).map(|idx| T::from(idx as u64)).collect(),
        &T::ZERO,
    );

    let mut mult_share = h_shares[0].multiply_const(&basis[0]);
    for (r, share) in basis.into_iter().zip(h_shares).skip(1) {
        mult_share = mult_share.add(&share.multiply_const(&r));
    }

    Ok(mult_share)
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};

    use crate::math::mersenne61::Mersenne61;
    use crate::math::FiniteField;

    use super::{compute_shamir_share, reconstruct_secret};

    #[test]
    fn secret_sharing_reconstruction_correctness() {
        const N_MAX_PARTIES: usize = 100;
        const N_SAMPLES: usize = 30;

        for _ in 0..N_SAMPLES {
            let mut rng = thread_rng();
            let secret = Mersenne61::random(&mut rng);
            let n_parties = rng.gen::<usize>() % N_MAX_PARTIES + 3;
            let threshold = n_parties / 2;
            let shares = compute_shamir_share(&secret, n_parties, threshold, &mut rng);

            let reconst_secret = reconstruct_secret(shares);

            assert_eq!(secret, reconst_secret);
        }
    }
}
