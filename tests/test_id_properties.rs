use itertools::{EitherOrBoth, Itertools};
use std::ops::BitXor;

#[derive(PartialEq, Ord, PartialOrd, Eq, Debug, Clone, Hash)]
pub struct Base256 {
    inner: Vec<u8>,
}

impl std::ops::Deref for Base256 {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Into<Vec<u8>> for Base256 {
    fn into(self) -> Vec<u8> {
        self.inner
    }
}

impl Base256 {
    pub fn new(inner: Vec<u8>) -> Self {
        Base256 { inner }
    }

    pub fn scalar_multiply(self, value: u8) -> Self {
        let mut res = Base256::new(vec![0]);
        for _ in 0..value {
            res = res + self.clone();
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

impl std::ops::Add for Base256 {
    type Output = Base256;

    fn add(self, rhs: Self) -> Self::Output {
        let mut overflow: u8 = 0;
        let mut res: Vec<u8> = Vec::with_capacity(std::cmp::max(self.inner.len(), rhs.inner.len()));
        let mut rev_a = self.inner.clone();
        let mut rev_b = rhs.inner.clone();
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

fn add_scalar_overflow(a: u8, b: u8, overflow: u8) -> (u8, u8) {
    let mut next_overflow = 0;
    let res = match a.checked_add(b) {
        Some(val) => match val.checked_add(overflow) {
            Some(val_overflow) => val_overflow,
            None => {
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