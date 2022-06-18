# Stage 1: Let's build with bazel!

Alright, we have a Rust project that somewhat represents a case
that you might want to bazel-ify: we have a bunch of different languages
all living in the same repository, and we are interested in caching
our build steps. Let's go through incorporating bazel into the Rust parts of this project.
Our goal for this stage will just be to get the project building with bazel---let's hold
off on vendoring with bazel for a little later.
We are all about those quick iteration times and getting feedback!

## How did we get here?

Well, we have a project that currently builds with cargo.
We have four main pieces of documentation for Rust-specific bazel rules
that will be helpful:

- https://bazelbuild.github.io/rules_rust/,
- https://bazelbuild.github.io/rules_rust/defs.html,
- https://bazelbuild.github.io/rules_rust/crate_universe.html, and
- https://bazelbuild.github.io/rules_rust/flatten.html, which is all of the documentation
  in one flattened location.

Let's get started! First, we will setup our project.
The root-level documentation helpfully has such a subsection:
https://bazelbuild.github.io/rules_rust/#setup.
Let's copy those contents to a `WORKSPACE.bazel` file
we put at the root level of our project:

```python
{{ #include ./stage-1-bazel-from-rules-rust-example/.workspace1.bazel }}
```

[Later in that page](https://bazelbuild.github.io/rules_rust/#specifying-rust-version)
they mention how we can specify the Rust version we want to use:

```python
{{ #include ./stage-1-bazel-from-rules-rust-example/WORKSPACE.bazel:18 }}
```

Let's do that as well!

> 👀 I really like specifying the Rust version for a project
> in a [`rust-toolchain` file](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file).
> It would be nice if I could specify a path to one
> instead of duplicating version information in bazel.
> This might be a nice feature to request!

Alright, this is enough for our first attempt at our `WORKSPACE` file.
In that same page, it mentions `crate_universe`, an experimental set of rules
for generating bazel targets for external dependencies. We want this eventually,
let's see if it's a good starting point for our first `BUILD` file!

> :eyes: There are other options for working with dependencies, such as
> [`cargo-raze`](https://github.com/google/cargo-raze).
> I went with `crate_universe` because I heard rumors that `cargo-raze` (or its functionality)
> was being merged into the main `rules_rust` rule set.
> I also [tried making it work](https://github.com/prestontw/bazel-rust-vendored-experiment/commit/eef1ff99496f570fa224fcacab60d7f39f6940e3)
> when using `crate_universe` ran into some snags,
> but ended up giving up and trying `crate_universe` again.

Slow down, eager beaver! Looking at the setup (https://bazelbuild.github.io/rules_rust/crate_universe.html#setup),
there are actually some changes that we need to make to our `WORKSPACE` file:

```python
{{ #include ./stage-1-bazel-from-rules-rust-example/WORKSPACE.bazel:20:22 }}
```

One of the ways that we can handle our dependencies is through a cargo workspace:
https://bazelbuild.github.io/rules_rust/crate_universe.html#cargo-workspaces.
This matches our project structure already, yay!
Let's copy that code:

```python
{{ #include ./stage-1-bazel-from-rules-rust-example/.workspace2.bazel:24:36}}
```

There's another code example for a `BUILD` file for a library,
but we are building a binary.
Let's copy a barebones version of the [`rust_binary`](https://bazelbuild.github.io/rules_rust/defs.html#rust_binary)
example to build our backend service:

```python
{{ #include ./stage-1-bazel-from-rules-rust-example/backend/.build1.bazel }}
```

Sweet! Fast bazel-cached builds here we come! Let's try building all targets with

```
bazel build //...
```

and watch bazel fill in that initial cache!

...

Wait.

I'm getting an error about `Cargo.Bazel.lock` not existing.
That makes sense---it's mentioned in https://bazelbuild.github.io/rules_rust/crate_universe.html#crates_repository.
Let's add it with

```
touch Cargo.Bazel.lock
```

and try again!

Alright, I see another error:

```
ERROR: An error occurred during the fetch of repository 'crate_index':
   Traceback (most recent call last):
        File "/private/var/tmp/_bazel_preston/8649a483de3e22846dc6c9def4d70061/external/rules_rust/crate_universe/private/crates_repository.bzl", line 33, column 28, in _crates_repository_impl
                lockfile = get_lockfile(repository_ctx)
        File "/private/var/tmp/_bazel_preston/8649a483de3e22846dc6c9def4d70061/external/rules_rust/crate_universe/private/generate_utils.bzl", line 259, column 35, in get_lockfile
                path = repository_ctx.path(repository_ctx.attr.lockfile),
Error in path: Unable to load package for //:Cargo.Bazel.lock: BUILD file not found in any of the following directories. Add a BUILD file to a directory to mark it as a package.
 - /Users/preston/git/bazel-rust-guided-experiment/stage-1-bazel-from-rules-rust-example
```

Looking at that documentation, it does have a `BUILD` file at the same level
as the `Cargo.Bazel.lock` file---let's add an empty one and see if it works!

Alright, we are getting closer. New error:

```
An error occurred during the fetch of repository 'crate_index':
   Traceback (most recent call last):
        File "/private/var/tmp/_bazel_preston/8649a483de3e22846dc6c9def4d70061/external/rules_rust/crate_universe/private/crates_repository.bzl", line 44, column 28, in _crates_repository_impl
                repin = determine_repin(
        File "/private/var/tmp/_bazel_preston/8649a483de3e22846dc6c9def4d70061/external/rules_rust/crate_universe/private/generate_utils.bzl", line 326, column 13, in determine_repin
                fail((
Error in fail: The current `lockfile` is out of date for 'crate_index'. Please re-run bazel using `CARGO_BAZEL_REPIN=true` if this is expected and the lockfile should be updated.
```

Again, this is somewhat expected---the subsection
https://bazelbuild.github.io/rules_rust/crate_universe.html#repinning--updating-dependencies
points it out.

We can either run `bazel sync` as that section specifies, or
follow that error message and pass that specific flag to the command
we ran above.
Whichever we do, we end up with

```
no such package '@crate_index//': Command [/private/var/tmp/_bazel_preston/8649a483de3e22846dc6c9def4d70061/external/crate_index/cargo-bazel, "splice", "--output-dir", /private/var/tmp/_bazel_preston/8649a483de3e22846dc6c9def4d70061/external/crate_index/splicing-output, "--splicing-manifest", /private/var/tmp/_bazel_preston/8649a483de3e22846dc6c9def4d70061/external/crate_index/splicing_manifest.json, "--extra-manifests-manifest", /private/var/tmp/_bazel_preston/8649a483de3e22846dc6c9def4d70061/external/crate_index/extra_manifests_manifest.json, "--cargo", /private/var/tmp/_bazel_preston/8649a483de3e22846dc6c9def4d70061/external/rust_darwin_aarch64/bin/cargo, "--rustc", /private/var/tmp/_bazel_preston/8649a483de3e22846dc6c9def4d70061/external/rust_darwin_aarch64/bin/rustc] failed with exit code 1.
STDOUT ------------------------------------------------------------------------

STDERR ------------------------------------------------------------------------
Error: Some manifests are not being tracked. Please add the following labels to the `manifests` key: {
    "//backend:Cargo.toml",
}
```

This makes sense, and was something that I was wondering about
when we filled in the manifests originally.

> :eyes: ... Were you?

> :facepalm: Yes, actually, part of the point of this exercise is to
>
> 1. go through and record errors that I run into,
> 1. provide an example of how someone went through the `rules_rust` documentation, and
> 1. do the minimum amount of work needed to get things in a functioning state.
>
> If I do something extra at some early point, it might be unnecessary after all!

> :eyes: Is this, perhaps, foreshadowing..?

Let's quickly add this manifest to our `WORKSPACE.bazel` file so that
`crates_repository` looks like

```python
{{ #include ./stage-1-bazel-from-rules-rust-example/WORKSPACE.bazel:26:33 }}
```

Let's try building now! If we run `bazel build //...`,
we get... Rust compilation errors?

Ahh, it's all for our dependencies.
Let's specify that the deps for our `hello_world` target
are `all_crate_deps()` and let's see if that works:

```python
{{ #include ./stage-1-bazel-from-rules-rust-example/backend/BUILD.bazel:4:8 }}
```

Oh. My. Goodness. It works!
Running

```
bazel run //backend:hello_world
```

seems to work fine.
And we can verify that it's working by going to
`localhost:3000` and seeing that classic greeting.

## What did we do?

Whew! We went through several different pages of documentation
and got `bazel` to build our project. Hooray!
It's not using our cached dependencies, though...

## What's next?

Let's get bazel to use our dependencies on-disk.
Strap in, because the next stage is a little bumpy.

> :eyes: Did anyone else see https://bazelbuild.github.io/rules_rust/crate_universe.html#crates_vendor?
> Just me?

> :facepalm: I saw it, but it wasn't in the top-level example
> for one of the two ways to support a cargo workflow.

> :eyes: It's used in
> https://github.com/bazelbuild/rules_rust/blob/0265c293f195a59da45f83aafcfca78eaf43a4c5/examples/crate_universe/vendor_local_manifests/BUILD.bazel#L5...

> :facepalm: Again, an example of someone going through the documentation as written.
> Maybe chalk it up to... foreshadowing? ⛈
