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

> 👀 If I could go back and do this, I would use the `--versioned-dirs` option
> for fewer changes when vendoring with bazel...
> I might also put it in a different directory...
>
> Ooh, foreshadowing!

We can verify this is successful with `cargo build --offline`.

# What's next?

Well, this currently isn't building with bazel.
Let's try to do that, while maintaining cargo build-ability
so local development is nice and what developers are used to.
Onwards to the next stage!
