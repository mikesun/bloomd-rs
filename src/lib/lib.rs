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
    // Number of hash functions
    num_hash_functions: usize,

    // Bit vector storing Bloom filter
    bits: BitVec<u8>,
}

impl BloomFilter {
    /// Instantiate a new [`BloomFilter`] by providing the expected `num_elements` that will
    /// be added to the Bloom filter and the target `false_positive_rate`.
    ///
    /// [`BloomFilter`]: BloomFilter
    pub fn new(num_elements: usize, false_positive_rate: f32) -> BloomFilter {
        let m = calc_m(num_elements, false_positive_rate);
        let k = calc_k(num_elements, m);

        BloomFilter {
            num_hash_functions: k,
            bits: bitvec![u8, Lsb0; 0; m],
        }
    }

    /// Returns size in bytes of the Bloom filter's bit vector.
    pub fn size(&self) -> usize {
        self.bits.len() / 8
    }

    /// Insert an item into the Bloom filter.
    ///
    /// To insert an item *`x`* into the Bloom filter, we first compute the *`k`* hash
    /// functions on *`x`*, and for each resulting hash, set the corresponding slot of `A`
    /// to 1.
    pub fn insert<T: Hash>(&mut self, item: &T) {
        for i in 0..self.num_hash_functions {
            let b = self.calc_bit(item, i);
            self.bits.set(b, true);
        }
    }

    /// Returns whether Bloom filter contains the item. It may return a false positive
    /// but will never return a false negative.
    ///
    /// Computes *`k`* hash functions on *`x`*, and the first time one of the corresponding
    /// slots of *`A`* equals `0`, the lookup reports the item as `Not Contained`; otherwise
    /// it reports the item as `Contained`.
    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        for i in 0..self.num_hash_functions {
            if !(self.bits[self.calc_bit(item, i)]) {
                return false;
            }
        }
        true
    }

    /// Calculate index of bit for given item and hashing function number
    fn calc_bit<T: Hash>(&self, item: &T, hash_func_num: usize) -> usize {
        let mut hasher = SipHasher::new_with_keys(hash_func_num as u64, 0);
        item.hash(&mut hasher);
        hasher.finish() as usize % self.bits.len()
    }
}

/// Calculate the appropriate size in bits of the Bloom filter, `m`, given
/// `n` and `f`, the expected number of elements contained in the Bloom filter and the
/// target false positive rate, respectively.
///
/// *`(-nln(f))/ln(2)^2`*
fn calc_m(n: usize, f: f32) -> usize {
    // https://en.wikipedia.org/wiki/Bloom_filter#Optimal_number_of_hash_functions
    (-f.ln() * (n as f32) / 2_f32.ln().powf(2_f32)) as usize
}

/// Calculate the number of hash functions to use, `k`, given `n` and `m`, the expected
/// number of elements contained in the Bloom filter and the size in bits of the Bloom
/// filter.
///
/// *`(mln(2)/n)`*
fn calc_k(n: usize, m: usize) -> usize {
    // https://en.wikipedia.org/wiki/Bloom_filter#Optimal_number_of_hash_functions
    ((m as f32 * 2_f32.ln()) / n as f32) as usize
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn check_sync<T: Sync>(_t: &T) {}

    fn check_send<T: Send>(_t: &T) {}

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
    fn size() {
        let bloom = BloomFilter::new(100_000, 0.01);
        assert_eq!(bloom.size(), 119813);
    }

    #[test]
    fn contains_true() {
        let mut bloom = BloomFilter::new(100_000, 0.01);
        bloom.insert(&"hi");
        assert!(bloom.contains(&"hi"));
    }

    #[test]
    fn contains_false() {
        let mut bloom = BloomFilter::new(100_000, 0.01);
        bloom.insert(&"hi");
        assert!(!bloom.contains(&"yo"));
    }

    #[test]
    fn thread_safe() {
        let b = BloomFilter::new(100_000, 0.01);
        check_sync(&b);
        check_send(&b);
    }
}
