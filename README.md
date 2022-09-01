Simple Arithmetic for Byte-like Objects

`Vec<u8>` object that implements a subset of basic arithmetic, namely addition and integer
multiplication.

Multiplication is implemented as multiplicative addition.
```
use byte_arithmetic::Base256;
assert_eq!(
    Base256::new(vec![1,2,3]) + Base256::new(vec![1,2,3]),
    Base256::new(vec![2,4,6])
);
assert_eq!(
    Base256::new(vec![1,2,3]) * 3,
    Base256::new(vec![3,6,9])
);
```