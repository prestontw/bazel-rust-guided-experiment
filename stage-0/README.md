# Stage 0

This is our starting version. This tries to mimic a monorepo where we have a backend
(where our Rust code goes) as well as a frontend. We also have a utility `deployment`
folder for general tools, which may include additional Rust code.
Since this might contain more Rust code, I'm setting this up with a [workspace `Cargo.toml` file](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).

For building the backend, this stage uses cargo.

## How did we get here?

This is copied from Axum's [`readme` example](https://github.com/tokio-rs/axum/tree/19fe93262fc14862f828b1db8b434fd8608a2a87/examples/readme).
For actually vendoring the dependencies, this uses `cargo-vendor`:
https://doc.rust-lang.org/cargo/commands/cargo-vendor.html.

There are two things to note on this vendoring:
- we used versioned directories via the `--versioned-dirs` flag to `cargo vendor`, and
- it's actually nested inside of our vendored directory, specifically for Rust.
This anticipates adding other vendored dependencies for our front-end code
(and better supports bazel's structure later).

With the above two notes, the command that we run is
```sh
cargo vendor --versioned-dirs 3rd-party/crates
```

> :facepalm: I actually messed both of these things up when I originally went through this.
> This lead to larger diffs when using bazel since I had to change the vendoring strategy
> (vendored directories)
> as well as the directory structure.

We can verify this is successful with `cargo build --offline`.

# What's next?

Well, this currently isn't building with bazel.
Let's try to do that, while maintaining cargo build-ability
so local development is nice and what developers are used to.
Onwards to the next stage!
