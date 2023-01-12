use itertools::{EitherOrBoth, Itertools};
use std::ops::BitXor;
use serde::{Deserialize, Serialize};

/// Base256 Object
///
/// Vec<u8> object that implements a subset of basic arithmetic, namely addition, subtraction,
/// and integer multiplication.
///
/// Also implements a wrapped addition around a specific byte length, for the purpose of adding
/// hashes of a specific size, i.e. 32byte hashes.
///
/// Multiplication is implemented as multiplicative addition.
/// ```
/// use byte_arithmetic::Base256;
/// assert_eq!(
///     Base256::new(vec![1,2,3]) + Base256::new(vec![1,2,3]),
///     Base256::new(vec![2,4,6])
/// );
/// assert_eq!(
///     Base256::new(vec![1,2,3]) * 3,
///     Base256::new(vec![3,6,9])
/// );
/// assert_eq!(
///      Base256::new(vec![255, 255]).wrapped_add(
///             Base256::new(vec![0, 1]), 2
///         ), Base256::new(vec![0, 0]));
/// assert_eq!(
///      Base256::new(vec![255, 255]).wrapped_add(
///             Base256::new(vec![0, 1]), 3
///         ), Base256::new(vec![1, 0, 0]));
/// ```
#[derive(PartialEq, Ord, PartialOrd, Eq, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct Base256 {
    inner: Vec<u8>,
}

impl std::ops::Deref for Base256 {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Base256> for Vec<u8> {
    fn from(base256: Base256) -> Self {
        base256.inner
    }
}

impl From<Vec<u8>> for Base256 {
    fn from(buffer: Vec<u8>) -> Self {
        Base256::new(buffer)
    }
}

impl Base256 {
    pub fn new(inner: Vec<u8>) -> Self {
        Base256 { inner }
    }

    pub fn empty() -> Self { Base256 { inner: vec![] }}

    pub fn scalar_multiply(self, value: u8) -> Self {
        let mut res = Base256::new(vec![0]);
        for _ in 0..value {
            res = res + self.clone();
        }
        res
    }

    pub fn wrapped_scalar_multiply(self, value: u8, byte_length: usize) -> Self {
        let mut res = Base256::new(vec![0]);
        for _ in 0..value {
            res = res.wrapped_add(self.clone(), byte_length);
        }
        res
    }

    pub fn wrapped_add(self, other: Self, byte_length: usize) -> Self {
        let mut res = self + other;
        let inner_len = res.inner.len();
        if inner_len > byte_length {
            res.inner = res.inner.into_iter().skip(inner_len - byte_length).collect();
        }
        res
    }
}

impl BitXor for Base256 {
    type Output = Base256;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Base256::new(
            self.iter()
                .zip_longest(rhs.iter())
                .map(|x| match x {
                    EitherOrBoth::Both(a, b) => *a ^ *b,
                    EitherOrBoth::Left(a) => *a,
                    EitherOrBoth::Right(b) => *b,
                })
                .collect(),
        )
    }
}

impl std::ops::Mul<u8> for Base256 {
    type Output = Base256;

    fn mul(self, rhs: u8) -> Self::Output {
        self.scalar_multiply(rhs)
    }
}

impl std::ops::Sub for Base256 {
    type Output = Base256;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut underflow = 0;
        let mut res: Vec<u8> = Vec::with_capacity(std::cmp::min(self.inner.len(), rhs.inner.len()));
        if self < rhs {
            panic!("Underflow")
        }
        let mut rev_a = self.inner;
        let mut rev_b = rhs.inner;
        rev_a.reverse();
        rev_b.reverse();
        for zipped_elem in rev_a.into_iter().zip_longest(rev_b.into_iter()) {
            let (x, y): (u8, u8) = match zipped_elem {
                EitherOrBoth::Both(a, b) => (a, b),
                EitherOrBoth::Left(a) => (a, 0),
                EitherOrBoth::Right(b) => (0, b),
            };
            let (result, local_underflow) = sub_scalar_underflow(x, y, underflow);
            res.insert(0, result);
            underflow = local_underflow;
        }
        Base256 { inner: res }
    }
}

impl std::ops::Add for Base256 {
    type Output = Base256;

    fn add(self, rhs: Self) -> Self::Output {
        let mut overflow: u8 = 0;
        let mut res: Vec<u8> = Vec::with_capacity(std::cmp::max(self.inner.len(), rhs.inner.len()));
        let mut rev_a = self.inner;
        let mut rev_b = rhs.inner;
        rev_a.reverse();
        rev_b.reverse();
        for zipped_elem in rev_a.into_iter().zip_longest(rev_b.into_iter()) {
            let (x, y): (u8, u8) = match zipped_elem {
                EitherOrBoth::Both(a, b) => (a, b),
                EitherOrBoth::Left(a) => (a, 0),
                EitherOrBoth::Right(b) => (0, b),
            };
            let (result, local_overflow) = add_scalar_overflow(x, y, overflow);
            res.insert(0, result);
            overflow = local_overflow;
        }
        if overflow > 0 {
            res.insert(0, overflow);
        }
        Base256 { inner: res }
    }
}

// [1, 5] - [0, 8] = [0, 7]
// (5, 8, 0) => (7, 1)
// (1, 0, 1) => (0, 0)
fn sub_scalar_underflow(a: u8, b:u8, underflow: u8) -> (u8, u8) {
    let mut next_underflow = 0;
    let res = match a.checked_sub(b) {
        Some(val) => match val.checked_sub(underflow) {
            Some(val_underflow) => val_underflow,
            None => {
                let res = (a as u16 + 255) - b as u16;
                next_underflow += 1;
                res as u8
            }
        },
        None => {
            let res = (a as u16 + 255) - b as u16;
            next_underflow += 1;
            res as u8
        }
    };
    (res, next_underflow)
}

fn add_scalar_overflow(a: u8, b: u8, overflow: u8) -> (u8, u8) {
    let mut next_overflow = 0;
    let res = match a.checked_add(b) {
        Some(val) => match val.checked_add(overflow) {
            Some(val_overflow) => val_overflow,
            None => {
                // Increase where overflow will happen, then drop the value.
                let res = val as u16 + overflow as u16;
                next_overflow = (res / 256) as u8;
                (res - (next_overflow as u16 * 256)) as u8
            }
        },
        None => {
            let res = a as u16 + b as u16 + overflow as u16;
            next_overflow = (res / 256) as u8;
            (res - (next_overflow as u16 * 256)) as u8
        }
    };
    (res, next_overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_add() {
        assert_eq!(add_scalar_overflow(0, 1, 0), (1, 0));
        assert_eq!(add_scalar_overflow(255, 1, 0), (0, 1));
        assert_eq!(add_scalar_overflow(255, 255, 2), (0, 2));
    }

    #[test]
    fn test_scalar_sub_direct() {
        assert_eq!(sub_scalar_underflow(5, 8, 0), (252, 1));
        assert_eq!(sub_scalar_underflow(1, 0, 0), (1, 0));
        assert_eq!(sub_scalar_underflow(1, 0, 1), (0, 0));
    }

    #[test]
    fn test_scalar() {
        assert_eq!(
            Base256::new(vec![0]) + Base256::new(vec![1]),
            Base256::new(vec![1])
        );
    }

    #[test]
    fn test_scalar_sub() {
        assert_eq!(
            Base256::new(vec![30]) - Base256::new(vec![25]),
            Base256::new(vec![5])
        );
    }

    #[test]
    fn test_multiple_sub() {
        assert_eq!(
            Base256::new(vec![30, 200]) - Base256::new(vec![30, 170]),
            Base256::new(vec![0, 30])
        );
    }

    #[test]
    fn test_underflow_multiple_sub() {
        assert_eq!(
            Base256::new(vec![200, 200]) - Base256::new(vec![0, 255]),
            Base256::new(vec![199, 200])
        );
    }

    #[test]
    fn test_multiple() {
        assert_eq!(
            Base256::new(vec![1, 2, 3]) + Base256::new(vec![2, 2, 2]),
            Base256::new(vec![3, 4, 5])
        );
    }

    #[test]
    fn test_overflow_multiple() {
        assert_eq!(
            Base256::new(vec![255, 255, 255]) + Base256::new(vec![1]),
            Base256::new(vec![1, 0, 0, 0])
        );
    }

    #[test]
    fn test_scalar_mult() {
        assert_eq!(
            Base256::new(vec![1, 1, 1]).scalar_multiply(3),
            Base256::new(vec![3, 3, 3])
        );
    }

    #[test]
    fn test_scalar_wrapped_mult() {
        assert_eq!(
            Base256::new(vec![1, 1, 1]).wrapped_scalar_multiply(40, 3),
            Base256::new(vec![40, 40, 40])
        );
    }


    #[test]
    fn test_wrapped_addition() {
        assert_eq!(
            Base256::new(vec![255, 255]).wrapped_add(
                Base256::new(vec![0, 1]), 2
            ),
            Base256::new(vec![0, 0])
        );

        assert_eq!(
            Base256::new(vec![0, 0]).wrapped_add(
                Base256::new(vec![0, 1]), 2
            ),
            Base256::new(vec![0, 1])
        );
    }
}
