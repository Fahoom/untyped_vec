# untyped_vec
A type-erased vector type for Rust

## Installation

Add untyped_vec to your [Cargo.toml](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to use untyped_vec.

```toml
[dependencies]
untyped_vec = *
```

## Usage

```rust
let mut vec = untyped_vec::UntypedVec::new::<usize>()

vec.push(42);

assert_eq!(vec.get::<usize>(0), &42)
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License
[MIT](https://choosealicense.com/licenses/mit/)
