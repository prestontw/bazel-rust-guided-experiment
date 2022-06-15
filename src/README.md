# bazel-rust-guided-experiment

This article is about incorporating the [Bazel build tool](bazel.build) into a
Rust project so that both `cargo` and `bazel`-based builds, tests, and vendoring
still work. Specifically, this is a Rust project that makes use of vendored
dependencies. This integration works out really nicely, but there were some
painful details that I ran into. I'm writing this article to

1. Highlight those pain points and what I should have done instead,
1. Show how `cargo` and `bazel` work together to vendor dependencies, and
1. Point out opportunities to improve the `rules_rust` documentation.

I'm trying to keep this article focused on relevant issues that might come up
with integrating `bazel`. There were separate issues that came up due to
"incorrect" directory structure, using the wrong `bazel` rule, etc. Where I
tried something that didn't work, I will point it out as a kind of warning sign.

> :eyes: And I will point out places that might be problems in the future, or,
> in general, interesting things that you should think about to keep reading
> interactive.

> :facepalm: And I will respond like this when I made the wrong decision or ran
> into a sharp edge.

## Motivation---Why Bazel?

Rust compilation times are a common complaint. There have been significant
improvements here in raw Rust compiler performance,
but we can even bigger
savings by caching intermediate results between compilation runs. We can see the
value of this strategy in the push for turning on incremental mode for the Rust
compiler. Unfortunately, this isn't available everywhere that we might want to
build Rust code: local Docker, as part of a CI pipeline, etc.

My path to getting to Bazel has been somewhat circuitous. Here are some tools,
blogs, and comments that got me here:

1. [cargo-chef](https://github.com/LukeMathWalker/cargo-chef) is a tool to
   improve caching between Docker builds for Rust projects. This formalizes a
   process of turning a Rust project into a hollow skeleton just including its
   dependencies with the observation that the dependencies don't change
   frequently, so could be a good source of caching.
   [cargo-chef](https://github.com/LukeMathWalker/cargo-chef#limitations-and-caveats)
   lists some limitations, but they all really boil down to a simple caching
   strategy of one mega cache that only includes external dependencies. Local
   dependencies inside of the project are rebuilt from scratch each time, too,
   which is a major pain point when they dominate compilation times. This is
   something that I ran into at my previous strategy: there were some other
   issues that using `cargo-chef` presented (mostly around patched dependencies)
   but the main one was that we really wanted to cache local dependencies.
1. [fasterthanlime's blog on his ideal Rust setup](https://fasterthanli.me/articles/my-ideal-rust-workflow).
   I really enjoy Amos's articles. I really enjoy the depth, the narrative, the
   content, and the characters...

   > :eyes: What do you mean?

   > :facepalm: You don't... see it?

   > :eyes: Haha... No.

   I really enjoyed that article in particular because it had a lot of stuff I could
   ~~steal~~ look into for my own Rust development. He has a specific section on
   [CI](https://fasterthanli.me/articles/my-ideal-rust-workflow#circleci) which includes:

   > Because, as I mentioned before, I want different compile flags in CI... but
   > I also want sccache, which is a much better solution than any built-in CI
   > caching.
   >
   > The basic idea behind sccache, at least in the way I have it set up, it's
   > that it's invoked instead of rustc, and takes all the inputs (including
   > compilation flags, certain environment variables, source files, etc.) and
   > generates a hash. Then it just uses that hash as a cache key, using in this
   > case an S3 bucket in `us-east-1` as storage.

   I was poised to look into `sccache`,
   despite rumors of it being somewhat unreliable and unmaintained.
   And then...

1. I saw a comment on Reddit [for an alternative to `sccache`](https://www.reddit.com/r/rust/comments/ua09tc/comment/i5w7n6g/?utm_source=share&utm_medium=web2x&context=3).
   The parent post was about a Rust build tool, Fleet, that claims to lead to 5x faster build times.
   `bitemyapp` comments on various ways the tool achieves its speedups
   and offer some of their own thoughts and advice on optimizing Rust build times.
   I've copied what they say regarding `sccache`:

   > `sccache` with on-disk caching: `sccache` is really flaky and poorly maintained. I was only able to get the S3 backend to work by using a fork of `sccache` and even then it breaks all the time for mysterious reasons. Using the on-disk configuration is confusing to me, you can get the same benefit by just setting a shared Cargo target directory. I set `CARGO_TARGET_DIR` in my `.zshrc` to `$HOME/.cargo/cache` and that gets shared across all projects. My guess is [Fleet's authors] saw a benefit from using `sccache` because they hadn't tried that and were benefiting from the cross-project sharing.
   >
   > ...
   >
   > What I've found to be more effective than what Fleet does:
   >
   > - Bazel's caching is astoundingly good but `cargo-raze` is awkward and the ecosystem really wants to nudge you toward a pure rustc + vendored dependencies. I'm using this for CI/CD at my job right now because the caching is both more effective and far more reliable than `sccache`. It even knows what tests to skip if the inputs didn't change. You can override that if you want but I was very pleased. My team uses Cargo on their local development machines because the default on-disk cache is fine.
   > - ...

   If you're intersted in checking out improving Rust compilation times,
   I recommend reading that comment in its entirety.

1. And finally, the DevOps team at my old company was looking at bazel.

All of this nudged me to checking out bazel for my own learning.
My default Rust project template uses vendored dependencies,
and bazel can sometimes struggle with dependencies in general,
so I wanted to see how easy it was to get bazel working in that type of project setup.

I also wanted to see how bazel and cargo can work together,
since I enjoy using cargo locally.

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
