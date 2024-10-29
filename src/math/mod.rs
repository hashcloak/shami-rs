use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod lagrange;
pub mod mersenne61;

#[derive(Error, Debug)]
pub enum FieldError {
    #[error("trying to compute the inverse of zero")]
    ZeroInverse,
}

/// Trait that represent a finite field of integers modulo a prime p.
pub trait FiniteField:
    Debug + Sized + Clone + From<u64> + Serialize + for<'a> Deserialize<'a>
{
    /// Type of the underlying representation for a field element.
    type ValueType;

    /// Modulus used in for the field.
    const MODULUS: u64;

    /// Bit size of the elements in the field.
    const BIT_SIZE: usize;

    /// Additive identity of the field.
    const ZERO: Self;

    /// Multiplicative identity of the field.
    const ONE: Self;

    /// Adds two elements in the field.
    fn add(&self, other: &Self) -> Self;

    /// Multiplies to elements in the field.
    fn multiply(&self, other: &Self) -> Self;

    /// Computes the inverse of field element.
    fn inverse(&self) -> Result<Self, FieldError>;

    /// Compares equality between two field elements.
    fn equal(&self, other: &Self) -> bool;

    /// Computes the additive inverse of a field element.
    fn negate(&self) -> Self;

    /// Computes the subtraction between two field elements.
    fn subtract(&self, other: &Self) -> Self;

    /// Generates a random finite field element with a provided pseudo-random generator.
    fn random<R: Rng>(generator: &mut R) -> Self;
}

/// Represents a polynomial whose coefficients are elements in a finite field.
#[derive(PartialEq, Eq, Debug)]
pub struct Polynomial<T: FiniteField>(Vec<T>);

impl<T: FiniteField> Polynomial<T> {
    /// Evaluates the polynomial on a give value.
    pub fn evaluate(&self, value: &T) -> T {
        let mut result = self.0.last().unwrap().clone();
        for coeff in self.0[0..self.0.len() - 1].iter().rev() {
            result = coeff.add(&result.multiply(value));
        }
        result
    }

    /// Generates a random polynomial of a given degree using a given pseudo-random generator.
    pub fn random<R: Rng>(degree: usize, rng: &mut R) -> Self {
        let mut coefficients = Vec::with_capacity(degree + 1);
        for _ in 0..degree + 1 {
            coefficients.push(T::random(rng));
        }
        Self(coefficients)
    }
}

impl<T: FiniteField> Index<usize> for Polynomial<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: FiniteField> IndexMut<usize> for Polynomial<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<const N: usize, T: FiniteField> From<[T; N]> for Polynomial<T> {
    fn from(coefficients: [T; N]) -> Self {
        Self(Vec::from_iter(coefficients))
    }
}
