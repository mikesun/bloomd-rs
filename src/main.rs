use bloom::BloomFilter;

fn main() {
    let mut bloom = BloomFilter::new(100_000, 0.01);
    println!("m={}, k={}", bloom.m, bloom.k);

    bloom.insert(&"hi");
    assert!(bloom.contains(&"hi"));

    bloom.insert(&"no");
    assert!(bloom.contains(&"no"));

    assert!(!bloom.contains(&"yo"));
}
