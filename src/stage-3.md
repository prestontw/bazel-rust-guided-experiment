# Stage 3: Wrapping up

Wow, we've done it! We've built our project with bazel,
we are getting bazel to use our vendored dependencies,
things are looking up for us!

There's actually one part that I forgot,
something that is pretty important in a realistic Rust project.
And that's local dependencies.

> :eyes: There's also some small things that we can do,
> like using a more recent version of `rules_rust`.

## How did we get here?

Let's go through the upgrade first,
since it's pretty quick!
`rules_rust` has its releases available on
https://github.com/bazelbuild/rules_rust/releases.
Let's update it:

```python
{{ #include ./stage-3-upgrade-version/WORKSPACE.bazel:5:12 }}
```

> :eyes: Boom, done, next!

### Local Dependency

This requires a little project restructuring.
Let's move what we currently have in `backend` to a subdirectory
so we can add a sibling project to contain the dependency.
Let's call the new directory for the server, well, `server`.
For simplicity's sake, let's call the new library `math`
to contain the classic `double` function!

Let's work on making this project build on the cargo side,
then make the necessary changes on the bazel side.

First, let's update our workspace `Cargo.toml` file:

```toml
{{ #include ./stage-3-upgrade-version/.Cargo1.toml }}
```

And let's include a dependency on `math` and use the `double` function in `server`.

```toml
{{ #include ./stage-3-upgrade-version/backend/.Cargo1.toml:7:8}}
```

Running

```
cargo run
```

is available at `localhost:3000`!

> :facepalm: Sweet, seems good to me!

### Using a local dependency from `bazel`'s side

We will follow
https://bazelbuild.github.io/rules_rust/defs.html#rust_binary
for this.

Specifically,

![the library BUILD file](./.images/hello-lib-build.png)

and

![the binary BUILD file](./.images/hello-world-build.png)

Similar to how we exported our `backend`'s `Cargo.toml`,
we specify that `math` is a package.
If we wanted to restrict access to this library,
in case we wanted to reinforce architectural directory structure
(like in https://youtu.be/5OjqD-ow8GE?t=2089),
we could specify visibility other than `public`,
but let's do that now for the sake of getting up and running quickly.

```python
{{ #include ./stage-3-upgrade-version/backend/math/.build1.bazel }}
```

Before we update our `backend/server/BUILD.bazel`
to list this library as a dependency,
let's see if `deps = all_crate_deps(),` does this for us automatically.
Let's run

```
bazel build //backend/server:hello_world
```

> :facepalm: Hmm, I'm running into a bunch of errors like
>
> ```
> error[E0405]: cannot find trait `IntoResponse` in this scope
>   --> backend/server/src/main.rs:41:11
>    |
> 41 | ) -> impl IntoResponse {
>    |           ^^^^^^^^^^^^ not found in this scope
> ```
>
> I'll try running our vendoring command again and see if that works:
>
> ```
> bazel run //3rd-party:crates_vendor
> ```
>
> ```
> /Users/preston/git/bazel-rust-guided-experiment/src/stage-3-upgrade-version/3rd-party/BUILD.bazel:6:14: no such package 'backend': BUILD file not found in any of the following directories. Add a BUILD file to a directory to mark it as a package.
>
> - /Users/preston/git/bazel-rust-guided-experiment/src/stage-3-upgrade-version/backend and referenced by '//3rd-party:crates_vendor'
> ```
>
> Ahh, yep, let's update our paths for our vendoring `BUILD.bazel`.

> :eyes: If `math` needed dependencies to function,
> we would probably add it's `Cargo.toml` at this point
> in the same location.

Trying to re-vendor, I get

```
INFO: Build completed successfully, 539 total actions
thread 'main' panicked at 'called `Option::unwrap()` on a `None` value', external/rules_rust/crate_universe/src/splicing/splicer.rs:78:86
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

> :facepalm: Let's try adding `math`'s `Cargo.toml` to this list.

> :eyes: Should this work? If we just add the `Cargo.toml` here,
> do we expect an error or a successful result?
> If it's an error, what error do we expect?

Before bazel will let us do this, we need to export `math`'s `Cargo.toml`.
But, I'm still running into the same error.
This might be evidence that we need to add our
dependency on `math` to our `server/BUILD.bazel`,
or it could be that we are off the beaten path.

> :facepalm: I actually took a beat here to take a break.
> I checked to see if there were any examples that were doing what we are trying to do.
> I couldn't find any.
> After some more flailing, running lots of `bazel clean`'s,
> trying to revendor and running into errors,
> I decided to try another approach.
>
> I started from `stage-2`, but instead of going straight to adding a local dependency,
> I moved the folders to a structure that would support this.
> I noticed two things:
>
> 1. I was running into the same error, and
> 1. The directory structure didn't make much sense.
>
> Why is there this random nested folder?
> This pointed me to think that I was too quick
> to copy the server functionality into a sub-folder.
> Let's try again, but with `server`'s files inside of `backend`.

That gives us

```toml
{{ #include ./stage-3-upgrade-version/Cargo.toml }}
```

and

```toml
{{ #include ./stage-3-upgrade-version/backend/Cargo.toml::8 }}
```

for our root-level and "`server`" `Cargo.toml`'s.
Let's try from this point and see if we run into the same errors.

> :eyes: Let's check after all of this flailing that we are actually using
> `math` and that everything is setup correctly, including `Cargo.toml`'s
> as well as `BUILD.bazel`'s.
> This includes incorporating our dependency on `math` in our `BUILD.bazel`:
>
> ```
> {{ #include ./stage-3-upgrade-version/backend/.build1.bazel:9}}
> ```

Unexpectedly, this doesn't work:

```
ERROR: /Users/preston/git/bazel-rust-guided-experiment/src/stage-3-upgrade-version/backend/BUILD.bazel:6:12: Compiling Rust bin hello_world (1 files) failed: (Exit 1): process_wrapper failed: error executing command bazel-out/darwin_arm64-opt-exec-2B5CBBC6/bin/external/rules_rust/util/process_wrapper/process_wrapper --arg-file ... (remaining 130 arguments skipped)

Use --sandbox_debug to see verbose messages from the sandbox
error[E0433]: failed to resolve: use of undeclared crate or module `math`
  --> backend/src/main.rs:44:13
   |
44 |         id: math::double(1337),
   |             ^^^^ use of undeclared crate or module `math`

error: aborting due to previous error
```

This works with `cargo`, let's play around with our `bazel` rules a little.
Looking at [an example](https://github.com/bazelbuild/rules_rust/blob/59fab4e79f62bfa13551ac851a40696c24c0c3a4/examples/crate_universe/cargo_workspace/num_printer/BUILD.bazel#L11),
they specify the local dependency without the `:<name>` format.
Let's make the necessary changes to our `math` library
and reference it as such:

```python
{{ #include ./stage-3-upgrade-version/backend/math/BUILD.bazel:7 }}
```

```python
{{ #include ./stage-3-upgrade-version/backend/BUILD.bazel:9 }}
```

Running `bazel build //...`

```
INFO: Analyzed 3 targets (2 packages loaded, 4 targets configured).
INFO: Found 3 targets...
INFO: Elapsed time: 0.898s, Critical Path: 0.78s
INFO: 2 processes: 1 internal, 1 darwin-sandbox.
INFO: Build completed successfully, 2 total actions
```

> :facepalm: I can't belive that works!

> :eyes: _Most_ of the pain points we've run into
> seem to be assumptions around directory structure that aren't immediately clear.
> We faced this for dependencies, original repo layout,
> and now for integrating local dependencies.
> But there seems to be some kind of weird coupling between local dependency crate names
> and how we label them as `rust_library`'s!?
>
> A lot of the directory issues made a lot of sense
> and would probably be how a more robust, more mature
> codebase would be structured,
> but all of those project structures worked fine just with Cargo.
> Bazel is more opinionated in its folder structure.
> One of the reasons why I'm writing this is to show
> a directory structure that works.

## What did we do?

We made our project a little more realistic!
We used a more up to date version of `rules_rust` and added a local dependency to our project!
We also found another assumption bazel makes for source folders
and an unexpected assumption around local dependency names.

## What's next?

It's up to you!

> :tada: Doo-do-de-do do-do-do-doo!

Some ideas I have are:

- Running tests through `bazel`.
  I've done this before rather quickly,
  it shouldn't be too bad.
  They have nice utility rules we can use to enumerate all of the tests automatically.
- Building a Docker container containing our Rust application.
- Actually using this in a CI pipeline, such as CircleCI!

As far as what I want to do now that this is done:

- Give documentation feedback to `rules_rust`.
  - Some of the naming is confusing for a beginner,
    such as `crates_repository` vs `crate_repositories`,
    but it seems like it is with the rest of the bazel ecosystem.
    Maybe improved documentation on what these nouns mean would be helpful?
  - Add an example of vendoring with manifests in the `rules_rust` documentation.
    Both the examples for **Cargo Workspaces** and **Direct Packages** use `crates_repository`.
    I think that would make it clearer that there is a matrix of decisions between
    expressing dependencies (`Cargo.toml` or directly) and
    how those are made available for use (downloaded or vendored).
  - Add more comments in the sample files!
    This can help beginners decipher what is necessary,
    why we are specifying something, etc.
    There's a tension between making things beginner friendly
    and being complete in documentation,
    and I think code comments can help with this.
- Think about how much of this can be automatic.
  One easy one would be using the `rust-toolchain` file
  instead of duplicating that information in the `WORKSPACE.bazel` file.

  There have been several times in this experience where `cargo build` works fine,
  but `bazel` is confused and errors.
  A more difficult, but more amazing ask,
  is if a lot of this `bazel` infrastructure could be automatically generated.
  `crates_universe` does that for dependencies, but I'm imagining something like
  having something that we can run as
  `cargo run --bin <x>` being available as
  `bazel run //<x>` without a specific `rust_binary` `BUILD.bazel`
  file being written by the user.
  There are some other rules that accomplish this to varying degrees,
  like [npm packages with `bin` entries](https://bazelbuild.github.io/rules_nodejs/repositories.html#generated-macros-for-npm-packages-with-bin-entries).

> :facepalm: This has been fun, y'all.
> Thank you for sticking through this with me.
> I hope that I've shown you that it is possible to integrate `bazel`
> and `cargo` and that there are many opportunities for improvements here.

## Document links:

- https://bazelbuild.github.io/rules_rust/
- https://bazelbuild.github.io/rules_rust/defs.html
- https://bazelbuild.github.io/rules_rust/crate_universe.html
- https://github.com/bazelbuild/rules_rust/tree/main/examples/crate_universe
- https://bazelbuild.slack.com/archives/CSV56UT0F
- https://github.com/tokio-rs/axum/tree/19fe93262fc14862f828b1db8b434fd8608a2a87/examples/readme
