//! This crate provides a simple Bloom filter.
//!
//! A [Bloom filter](https://en.wikipedia.org/wiki/Bloom_filter) is a space-efficient,
//! probabilistic data structure that is used to test whether an element is a member of a
//! set. False positive matches are possible, but false negatives are not. Thus, they are
//! useful for situations where the query answer is expected to be "not a member" most of
//! the time. Elements can be added to the set, but not removed.
//!
//! Example:
//!
//! ```
//! use bloom::BloomFilter;
//! let mut bloom = BloomFilter::new(100, 0.01);
//!
//! bloom.insert(&"hi");
//! assert!(bloom.contains(&"hi"));
//! assert!(!bloom.contains(&"yo"));
//! ```
//!
//! [Bloom filter]: https://en.wikipedia.org/wiki/Bloom_filter

use bitvec::prelude::*;
use siphasher::sip::SipHasher;
use std::hash::{Hash, Hasher};

/// Bloom filter data structure.
pub struct BloomFilter {
    /// Expected number of elements
    pub n: u32,

    /// False positive rate
    pub f: f32,

    /// Number of bits in filter
    pub m: u32,

    /// Number of hash functions
    pub k: u32,

    /// Bit vector
    bits: BitVec<u8>,
}

impl BloomFilter {
    /// Instantiate a new [`BloomFilter`] by providing the expected `num_elements` that will
    /// be added to the Bloom filter and the target `false_positive_rate`.
    ///
    /// [`BloomFilter`]: BloomFilter
    pub fn new(num_elements: u32, false_positive_rate: f32) -> BloomFilter {
        let m = calc_m(num_elements, false_positive_rate);
        let k = calc_k(num_elements, m);

        BloomFilter {
            n: num_elements,
            f: false_positive_rate,
            m,
            k,
            bits: bitvec![u8, Lsb0; 0; m as usize],
        }
    }

    /// Insert an item into the Bloom filter.
    ///
    /// To insert an item *`x`* into the Bloom filter, we first compute the *`k`* hash
    /// functions on *`x`*, and for each resulting hash, set the corresponding slot of `A`
    /// to 1.
    pub fn insert<T: Hash>(&mut self, item: &T) {
        for i in 0..self.k {
            self.bits.set(hash(item, i) % self.m as usize, true);
        }
    }

    /// Returns whether Bloom filter contains the item. It may return a false positive
    /// but will never return a false negative.
    ///
    /// Computes *`k`* hash functions on *`x`*, and the first time one of the corresponding
    /// slots of *`A`* equals `0`, the lookup reports the item as `Not Contained`; otherwise
    /// it reports the item as `Contained`.
    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        for i in 0..self.k {
            if !(*self
                .bits
                .get(hash(item, i) % self.m as usize)
                .expect("bitarray not allocated"))
            {
                return false;
            }
        }
        true
    }
}

/// Hash with given seed
fn hash<T: Hash + ?Sized>(t: &T, seed: u32) -> usize {
    let mut hasher = SipHasher::new_with_keys(seed.into(), 0);
    t.hash(&mut hasher);
    hasher
        .finish()
        .try_into()
        .expect("type conversion from u64 to usize failed")
}

/// Calculate the appropriate size in bits of the Bloom filter, `m`, given
/// `n` and `f`, the expected number of elements contained in the Bloom filter and the
/// target false positive rate, respectively.
///
/// *`(-nln(f))/ln(2)^2`*
fn calc_m(n: u32, f: f32) -> u32 {
    // https://en.wikipedia.org/wiki/Bloom_filter#Optimal_number_of_hash_functions
    (-f.ln() * (n as f32) / 2_f32.ln().powf(2_f32)) as u32
}

/// Calculate the number of hash functions to use, `k`, given `n` and `m`, the expected
/// number of elements contained in the Bloom filter and the size in bits of the Bloom
/// filter.
///
/// *`(mln(2)/n)`*
fn calc_k(n: u32, m: u32) -> u32 {
    // https://en.wikipedia.org/wiki/Bloom_filter#Optimal_number_of_hash_functions
    ((m as f32 * 2_f32.ln()) / n as f32) as u32
}

#[cfg(test)]
mod tests {
    use crate::{calc_k, calc_m, BloomFilter};

    #[test]
    fn m() {
        let (n, f) = (1_000_000, 0.02);
        assert_eq!(calc_m(n, f), 8_142_363);
    }

    #[test]
    fn k() {
        let (n, m) = (1_000_000, 8_142_363);
        assert_eq!(calc_k(n, m), 5);
    }

    #[test]
    fn contains_true() {
        let mut bloom = BloomFilter::new(100_000, 0.01);
        bloom.insert(&"hi");
        assert!(bloom.contains(&"hi"));
    }

    #[test]
    fn contains_failse() {
        let mut bloom = BloomFilter::new(100_000, 0.01);
        bloom.insert(&"hi");
        assert!(!bloom.contains(&"yo"));
    }
}
