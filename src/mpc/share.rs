use std::cmp;

use serde::{Deserialize, Serialize};

use crate::math::FiniteField;

/// Represents a Shamir Share of a value.
#[derive(Debug, Serialize, Deserialize)]
pub struct ShamirShare<T> {
    /// The degree of the Shamir share.
    pub degree: usize,

    /// The random value of the share.
    pub value: T,
}

impl<T> ShamirShare<T>
where
    T: FiniteField,
{
    pub fn new(value: T, degree: usize) -> Self {
        Self { value, degree }
    }

    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            value: self.value.multiply(&other.value),
            degree: self.degree + other.degree,
        }
    }

    pub fn multiply_const(&self, other: &T) -> Self {
        Self {
            value: self.value.multiply(other),
            degree: self.degree,
        }
    }

    pub fn add_const(&self, other: &T) -> Self {
        Self {
            value: self.value.add(other),
            degree: self.degree,
        }
    }

    pub fn subtract_const(&self, other: &T) -> Self {
        Self {
            value: self.value.subtract(&other.negate()),
            degree: self.degree,
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            value: self.value.add(&other.value),
            degree: cmp::max(self.degree, other.degree),
        }
    }

    pub fn negate(&self) -> Self {
        Self {
            value: self.value.negate(),
            degree: self.degree,
        }
    }

    pub fn subtract(&self, other: &Self) -> Self {
        self.add(&other.negate())
    }
}
