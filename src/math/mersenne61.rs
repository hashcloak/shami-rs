use serde::Deserialize;
use serde::Serialize;

use super::FieldError;
use super::FiniteField;

/// Representation of a field element modulo 2^{61} - 1.
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub struct Mersenne61(u64);

impl From<u64> for Mersenne61 {
    fn from(value: u64) -> Self {
        let mut final_value = value;
        while final_value >= Self::MODULUS {
            final_value -= Self::MODULUS;
        }
        Self(final_value)
    }
}

impl FiniteField for Mersenne61 {
    type ValueType = u64;

    const MODULUS: u64 = 0x1FFFFFFFFFFFFFFF;
    const BIT_SIZE: usize = 61;
    const ONE: Self = Self(1);
    const ZERO: Self = Self(0);

    fn add(&self, other: &Self) -> Self {
        let add_result = self.0 + other.0;
        Self::from(add_result)
    }

    fn random<R: rand::Rng>(generator: &mut R) -> Self {
        let value: u64 = generator.gen();
        Self::from(value)
    }

    fn multiply(&self, other: &Self) -> Self {
        let non_reduced_mult: u128 = (self.0 as u128) * (other.0 as u128);
        let mut most_sig_bits = (non_reduced_mult >> Self::BIT_SIZE) as u64;
        let mut least_sig_bits = non_reduced_mult as u64;

        most_sig_bits |= least_sig_bits >> Self::BIT_SIZE;
        least_sig_bits &= Self::MODULUS;

        // Apply modular addition.
        let most_sig_bits_mod = Self::from(most_sig_bits);
        let least_sig_bits_mod = Self::from(least_sig_bits);
        most_sig_bits_mod.add(&least_sig_bits_mod)
    }

    fn equal(&self, other: &Self) -> bool {
        self == other
    }

    fn inverse(&self) -> Result<Self, super::FieldError> {
        if self.equal(&Self::ZERO) {
            Err(FieldError::ZeroInverse)
        } else {
            let mut k: i64 = 0;
            let mut new_k: i64 = 1;
            let mut r: i64 = Self::MODULUS as i64;
            let mut new_r: i64 = self.0 as i64;

            while new_r != 0 {
                let q = r / new_r;
                assign(&mut k, &mut new_k, q);
                assign(&mut r, &mut new_r, q);
            }

            if k < 0 {
                k += Self::MODULUS as i64;
            }

            Ok(Self::from(k as u64))
        }
    }

    fn negate(&self) -> Self {
        if !self.equal(&Self::ZERO) {
            Self::from(Self::MODULUS - self.0)
        } else {
            Self::ZERO
        }
    }

    fn subtract(&self, other: &Self) -> Self {
        if other.0 > self.0 {
            Self::from(self.0 + Self::MODULUS - other.0)
        } else {
            Self::from(self.0 - other.0)
        }
    }
}

/// Given v1, v2 and a constant q, computes the multiplicative exchange
/// v1 <- v2 and v2 <- v1 - q * v2.
fn assign(v1: &mut i64, v2: &mut i64, q: i64) {
    let temp = *v2;
    *v2 = *v1 - q * temp;
    *v1 = temp;
}

#[cfg(test)]
mod tests {
    use super::Mersenne61;
    use crate::math::FiniteField;
    use rand::thread_rng;

    #[test]
    fn zero() {
        let mut rng = thread_rng();
        let elem = Mersenne61::random(&mut rng);
        let s = elem.add(&Mersenne61::ZERO);
        assert_eq!(elem, s);

        let elem = Mersenne61::random(&mut rng);
        let s = elem.subtract(&Mersenne61::ZERO);
        assert_eq!(elem, s);
    }

    #[test]
    fn one() {
        let mut rng = thread_rng();
        let elem = Mersenne61::random(&mut rng);
        let s = elem.multiply(&Mersenne61::ONE);
        assert_eq!(elem, s);
    }

    #[test]
    fn negate() {
        let mut rng = thread_rng();
        let elem = Mersenne61::random(&mut rng);
        let s = elem.add(&elem.negate());
        assert_eq!(s, Mersenne61::ZERO);
    }

    #[test]
    fn subract() {
        let mut rng = thread_rng();
        let elem = Mersenne61::random(&mut rng);
        let s = elem.subtract(&elem);
        assert_eq!(s, Mersenne61::ZERO);
    }

    #[test]
    fn inverse() {
        let mut rng = thread_rng();
        const SAMPLES: usize = 100;
        for _ in 0..SAMPLES {
            let elem = Mersenne61::random(&mut rng);
            let s = elem.multiply(&elem.inverse().unwrap());
            assert_eq!(s, Mersenne61::ONE);
        }
    }

    #[test]
    fn mult_test1() {
        let a = Mersenne61::from(2);
        let b = Mersenne61::from(6);
        let r = Mersenne61::from(12);

        let s = a.multiply(&b);
        assert_eq!(s, r);
    }

    #[test]
    fn mult_conmutativity() {
        const SAMPLES: usize = 50;
        let mut rng = thread_rng();
        for _ in 0..SAMPLES {
            let a = Mersenne61::random(&mut rng);
            let b = Mersenne61::random(&mut rng);
            let mult1 = a.multiply(&b);
            let mult2 = b.multiply(&a);
            assert_eq!(mult1, mult2);
        }
    }
}
