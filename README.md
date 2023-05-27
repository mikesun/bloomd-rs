https://github.com/mikesun/bloomd-rs/actions/workflows/rust.yml/badge.svg

# bloomd

Wrote a simple Bloom filter library and gRPC service to learn some Rust and tonic.

## Bloom filter library
A [Bloom filter](https://en.wikipedia.org/wiki/Bloom_filter) is a space-efficient, probabilistic data structure that is used to test whether an element is a member of a set. False positive matches are possible, but false negatives are not. Thus, they are useful for situations where the query answer is expected to be "not a member" most of the time. Elements can be added to the set, but not removed.

Bloom filter parameters:
- `m` = space
- `n` = number of elements
- `k` = number of hash functions
- `f` = false positive rate

Bloom filters have two main components:
* a bit array *`A[0..m-1]`*, will all slots initially set to `0`
* *`k`* independent hash functions *`h1, h2, ..., hk`*, each mapping keys uniformly randomly onto a rang *`[0, m-1]`*

#### Insert
To insert an item *`x`* into the Bloom filter, we first compute the *`k`* hash functions on *`x`*, and for each resulting hash, set the corresponding slot of `A` to 1.

#### Lookup
Similar to insert, lookup computes *`k`* hash functions on *`x`*, and the first time one of the corresponding slots of *`A`* equals `0`, the lookup reports the item as `Not Present`; otherwise it reports the item as `Present`.

#### Configuration

This formula determines the false positive rate as a function of the other three parameters:

$$
f \approx (1-e^{-\frac{nk}{m}})^k 
$$

$m$ can be derived given $n$ and $f$ with:

$$
m = \frac{-nln(f)}{ln(2)^2}
$$

$k$ can be derived given $n$ and $m$ with:

$$
k = \frac{mln(2)}{n}
$$

## Bloom Filter gRPC Service

Start gRPC server:

```
cargo run --bin bloomd-server
```

Insert item:

```
grpcurl -plaintext -import-path ./proto -proto bloomd.proto 
  -d '{"item": "hi"}' \
  '[::1]:50051' bloomd.Bloomd/Insert
```

Check if item exists:

```
grpcurl -plaintext -import-path ./proto -proto bloomd.proto 
  -d '{"item": "hi"}' \
  '[::1]:50051' bloomd.Bloomd/Contains
```
