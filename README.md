rustiff
===
TIFF decoding/encoding library for Rust.

[![crates.io][cratesio-image]][cratesio]
[![docs.rs][docsrs-image]][docsrs]


[cratesio-image]: https://img.shields.io/crates/v/rustiff.svg
[cratesio]: https://crates.io/crates/rustiff
[docsrs-image]: https://docs.rs/rustiff/badge.svg
[docsrs]: https://docs.rs/rustiff


## Use

Put this in your `Cargo.toml`:

```toml
[dependencies]
rustiff = "0.1"
```

Then put this in your crate root:

```rust
extern crate rustiff
```

## Example

This example shows how to read TIFF data.

```rust
extern crate rustiff;

use rustiff::{
    Decoder,
    DecodeResult,
    DecodeError,
    Image,
    ImageData,
};
use std::fs::File;

fn main() -> DecodeResult<()> {
    let f = File::open("sample.tiff")?;
    let mut decoder = Decoder::new(f)?;
    let image = decoder.image()?;
    let image_data = image.data(); // Vec<u8> or Vec<u16>

    Ok(())
}
```

You can get the value associated with the tag.

```rust
extern crate rustiff;

use rustiff::{
    tag,
    IFD,
    Decoder,
    DecodeResult,
    DecodeError,
};
use std::fs::File;

fn main() -> DecodeResult<()> {
    let f = File::open("sample.tiff")?;
    let mut decoder = Decoder::new(f)?;
    let ifd = decoder.ifd()?;
    let width = decoder.get_value(&ifd, tag::ImageWidth)?;
    let height = decoder.get_value(&ifd, tag::ImageLength)?;

    Ok(())
}
```


