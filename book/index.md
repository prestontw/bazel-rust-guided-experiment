# bazel-rust-guided-experiment
A blog-post style collection of project states as we move from a vendored cargo project to bazel

## Motivation
- working on cargo chef
  - really, this gives us one mega layer, we want finer caching; doesn't cache local deps
- Amos's blog on ideal CI setup
  - looking into SCCache
- Reddit [comment](https://www.reddit.com/r/rust/comments/ua09tc/comment/i5w7n6g/?utm_source=share&utm_medium=web2x&context=3) on more dependable setup
- Looking at bazel at work

```rust
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[tokio::main]
```

![The Rust Logo](../src/Screen%20Shot%202022-06-04%20at%2012.42.40%20AM.png)

## Structure of this repo

Each child directory is the entire state of the project.
Hopefully, following along from one folder's readme
will lead to the next sequential folder.

The readme will go into detail on what we want to accomplish,
how we try to do this, and any errors we run into.

## TL;DR

Start from https://github.com/bazelbuild/rules_rust/tree/main/examples/crate_universe/vendor_local_manifests.
This project makes some arguably strange assumptions around vendoring locations
(and which BUILD file is responsible for keeping them up to date).
If you plan on having Rust in multiple directories, check out this repository.

If you are migrating an existing project, make sure that
your rust code is not at the root level of the repository.
(This is more than likely since bazel seems to be nicely
used in multi-lingual repos, in which case odds are rust source
directories are not at the root level.)

In comparison to the example directory,
This repo has some nice benefits like covering tests too,
but the rules_rust repo will stay in-sync with the actual bazel rules.

The bazel slack rust channel seems to be active and I found
people there to be very helpful! Thank you so much to them!

## Remaining tasks
Documentation-wise, maybe giving feedback to rules_rust?
rules rust repository link broken (https://bazelbuild.github.io/rules_rust/flatten.html#rust_register_toolchains)
(done!)

Naming is strange: crates_repository vs crate_repositories (though this seems consistent with scala rules naming?)
-> points to documentation again

Vendoring to top-level

Example of vendoring with manifests in documentation

Add comments to snippets in root example for rust rules.

For this project:
- Incorporating with CI
- Using a workspace root-level cargo.toml
- Building with Docker

Document links:
- https://bazelbuild.github.io/rules_rust/
- https://bazelbuild.github.io/rules_rust/defs.html
- https://bazelbuild.github.io/rules_rust/crate_universe.html
- https://github.com/bazelbuild/rules_rust/tree/main/examples/crate_universe
- https://bazelbuild.slack.com/archives/CSV56UT0F
- https://github.com/tokio-rs/axum/tree/19fe93262fc14862f828b1db8b434fd8608a2a87/examples/readme