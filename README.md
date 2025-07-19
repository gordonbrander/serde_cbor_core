# serde_cbor_core

This is a [Serde](https://github.com/serde-rs/serde) implementation for [CBOR Core](https://datatracker-ietf-org.lucaspardue.com/doc/draft-rundgren-cbor-core/). It provides a deterministic encoding for CBOR, detailed [Section 4.2.1. of RFC 8949](https://datatracker.ietf.org/doc/html/rfc8949#core-det) and expanded on in the [IETF draft for CBOR Core](https://datatracker-ietf-org.lucaspardue.com/doc/draft-rundgren-cbor-core/).

Deterministic encoding means the same CBOR will be encoded the same way, every time. This is useful when signing or hashing data, since it guarantees the same data will produce the same hash/sig.

The underlying library for CBOR encoding/decoding is [cbor4ii](https://github.com/quininer/cbor4ii) and the Serde implementation is also heavily based on their code.

This crate started as a fork of [serde_ipld_dagcbor](https://github.com/ipld/serde_ipld_dagcbor). Many thanks to everyone involved there.

## Usage

Storing and loading Rust types is easy and requires only
minimal modifications to the program code.

```rust
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

// Types annotated with `Serialize` can be stored as DAG-CBOR.
// To be able to load them again add `Deserialize`.
#[derive(Debug, Serialize, Deserialize)]
struct Mascot {
    name: String,
    species: String,
    year_of_birth: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let ferris = Mascot {
        name: "Ferris".to_owned(),
        species: "crab".to_owned(),
        year_of_birth: 2015,
    };

    let ferris_file = File::create("examples/ferris.cbor")?;
    // Write Ferris to the given file.
    // Instead of a file you can use any type that implements `io::Write`
    // like a HTTP body, database connection etc.
    serde_cbor_core::to_writer(ferris_file, &ferris)?;

    let tux_file = File::open("examples/tux.cbor")?;
    let tux_reader = BufReader::new(tux_file);
    // Load Tux from a file.
    // Performs roundtrip serialization meaning that the data will not change in any way.
    let tux: Mascot = serde_cbor_core::from_reader(tux_reader)?;

    println!("{tux:?}");
    // prints: Mascot { name: "Tux", species: "penguin", year_of_birth: 1996 }

    Ok(())
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
