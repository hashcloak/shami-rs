use super::FiniteField;

/// Computes the lagrange basis evaluated at `x`
pub fn compute_lagrange_basis<T: FiniteField>(nodes: Vec<T>, x: &T) -> Vec<T> {
    let mut lagrange_basis = Vec::with_capacity(nodes.len());
    for j in 0..nodes.len() {
        let mut basis = T::ONE;
        let x_j = &nodes[j];
        for (m, node) in nodes.iter().enumerate() {
            if m != j {
                let x_m = node;
                let numerator = x.subtract(x_m);
                let denominator = x_j.subtract(x_m);

                // The unwrap is safe because x_j - x_m is not zero.
                let term = numerator.multiply(&denominator.inverse().unwrap());
                basis = basis.multiply(&term);
            }
        }
        lagrange_basis.push(basis);
    }
    lagrange_basis
}

/// Computes the evaluation of the interpolated polynomial at `x`.
pub fn interpolate_polynomial_at<T: FiniteField>(evaluations: Vec<T>, alphas: Vec<T>, x: &T) -> T {
    assert!(alphas.len() == evaluations.len());
    let lagrange_basis = compute_lagrange_basis(alphas, x);
    let mut interpolation = T::ZERO;
    for (eval, basis) in evaluations.into_iter().zip(lagrange_basis) {
        interpolation = interpolation.add(&eval.multiply(&basis));
    }
    interpolation
}

#[cfg(test)]
mod tests {

    use crate::{math::FiniteField, Mersenne61};
    use rand::{seq::SliceRandom, thread_rng, Rng};

    use crate::math::Polynomial;

    use super::interpolate_polynomial_at;

    #[test]
    fn interpolation() {
        const MAX_DEGREE: u64 = 100;
        const N_SAMPLES: u64 = 100;

        let mut rng = thread_rng();

        for _ in 0..N_SAMPLES {
            let degree: usize = (rng.gen::<u64>() % MAX_DEGREE) as usize;
            let random_poly: Polynomial<Mersenne61> = Polynomial::random(degree, &mut rng);

            let evaluation_test = Mersenne61::random(&mut rng);

            // Generates degree + 1 evaluation points
            let mut eval_points: Vec<usize> = (0..1000).collect();
            eval_points.shuffle(&mut rng);
            let eval_points: Vec<Mersenne61> = eval_points[0..degree + 1]
                .iter()
                .map(|elem| Mersenne61::from(*elem as u64))
                .collect();

            let evaluations = eval_points
                .iter()
                .map(|x| random_poly.evaluate(x))
                .collect();

            let interpolated_evaluation =
                interpolate_polynomial_at(evaluations, eval_points, &evaluation_test);

            assert_eq!(
                interpolated_evaluation,
                random_poly.evaluate(&evaluation_test)
            )
        }
    }
}
