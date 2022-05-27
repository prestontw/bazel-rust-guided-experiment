## Stage 2: Let's vendor our dependencies through bazel!

If you want to follow along through a pull request,
all of this work is availagble at
https://github.com/prestontw/bazel-rust-guided-experiment/pull/1/.

To recap, we can build our project through bazel! Whoopee!
But it isn't making use of our vendored dependencies that we got
through `cargo vendor`! Let's try to remedy this.
Concretely, this is our goal for this stage.

Watch out, there is a little bit of flailing here.
Maybe not narratively (hopefully),
but this is something that I struggled with.
Some of the narrative might be light as I cover errors I ran into
and how I tried to get around them.

## How did we get here?

As alluded to before, there is a rule for vendoring dependencies:
https://bazelbuild.github.io/rules_rust/crate_universe.html#crates_vendor.
However, I don't really get its example.
It specifies an annotation---is this related to a dependency?
Does this supplant `Cargo.toml` information?

> :eyes: I think there is a difference between minimal examples
> to help beginners and more full-featured examples to show what
> is possible. One thing that could help a beginner with filtering
> out some of the more advanced options is: documentation!
> But in the context of code in examples, this most likely lives
> as comments. Having comments in this example would have been
> really helpful---it seemed so different from what I wanted
> that I ended up getting scared away.

This is too much. But, there are other worked examples!
Let's try copying [an example](https://github.com/bazelbuild/rules_rust/blob/0265c293f195a59da45f83aafcfca78eaf43a4c5/examples/crate_universe/vendor_local_manifests/BUILD.bazel) from the `rules_rust` repo.
We will tweak it slightly becase we want to reuse the vendored
dependencies we set up in `cargo vendor`:
```python
load("@rules_rust//crate_universe:defs.bzl", "crates_vendor")

crates_vendor(
    name = "crates_vendor",
    manifests = [":Cargo.toml"],
    vendor_path = "vendor",
    mode = "local",
)

load("@rules_rust//rust:defs.bzl", "rust_binary")
# load("@crate_index//:defs.bzl", "all_crate_deps")
load("//vendor:defs.bzl", "all_crate_deps")
```

This leads to error messages saying that
`vendor` is not a package. Hmm.
Maybe it's due to a kind of cyclic issue?
`crates_vendor` would create the `//vendor` package,
but then we try to use that same package potentially
before it's created since it's in the same file.
That's weird. Let's try upgrading our version of `rules_rust`
and see if that solves it for us.

> :eyes: `rules_rust` is now on version `0.5.0`
> while the documentation mentions `0.2.0`.
> Keeping documentation up to date with code is hard.
> I don't feel like this is a complaint because I can go
> and improve the situation through pull requests!

After updating our version (and syncing our lockfile),
we see the same error.
Maybe we can break the cycle by removing the code that relies
on `//vendor`.
Let's also specify the specific `backend` manifest.
Now our `backend` `BUILD` file looks like
```python

```

This actually works after one small correction.
There's an issue with some versions of `tokio` that leads
to errors on uses with `bazel`.
The [`vendor_local_manifests` example](https://github.com/bazelbuild/rules_rust/blob/0265c293f195a59da45f83aafcfca78eaf43a4c5/examples/crate_universe/vendor_local_manifests/Cargo.toml#L8) has a fix---let's
borrow other people's workarounds.

There's a bigger philosophical issue here, though.
This places the dependencies inside of `backend`.
If we want some of our utilities to use the same
version as we do in our backend, this nested directory
structure isn't condusive to that.

> :eyes: If the utilities dependency issue isn't a motiving issue
> to you, imagine that this monorepo also has some microservices
> in a `services` directory.

Trying `/vendor` as the `vendor_path` gives me permissions issues,
which makes me think that it's not actually true that
> Absolute paths will be treated as relative to the workspace root

Just as an experiment, let's try specifying the full path to `vendor`.
This "works" as in it's able to vendor the dependencies.
This reveals that `bazel` vendors dependencies slightly
differently than how we have been, and more in line with
```
cargo vendor --versioned-dirs
```
So, let's replace our unversioned directories with versioned directories
before we continue on our merry way.

The full path won't work in general, so let's try a relative path!
Let's try `../vendor` and see if it works.
I think at this point,
https://bazelbuild.github.io/rules_rust/crate_universe.html#crates_vendor
is starting to make some more sense.
We have a specific target for vendoring the dependencies,
and, for some reason,
the example that we've been following combined that target with
the general rust library target.

Anyway, let's try loading from it. Our `BUILD` file now ends with
```python
load("//vendor:defs.bzl", "all_crate_deps")

rust_binary(
    name = "hello_world",
    srcs = ["src/main.rs"],
    deps = all_crate_deps(normal = True,),
)
```
Now that we aren't using `crate_index`,
let's remove it from our `WORKSPACE` file.
It now ends with
```python
load("@rules_rust//crate_universe:crates_deps.bzl", "crate_repositories")

crate_repositories()
```
Finally, let's build just to make sure that everything works.

```
ERROR: /Users/preston/git/bazel-rust-guided-experiment/stage-2-crates-vendor/backend/BUILD.bazel:13:12: //backend:hello_world: invalid label '//backend/../vendor/axum-0.5.6:axum' in element 0 of attribute 'deps' in 'rust_binary' rule: invalid package name 'backend/../vendor/axum-0.5.6': package name component contains only '.' characters
```

So it looks like relative paths didn't work.

> :eyes: So time for the file actually mentioned in the documentation, right?

> :facepalm: Yes. It still feels weird to me that the path
> being relative to the workspace root didn't work, but oh well!

Copying the vendoring part of our `backend/BUILDFILE` to
another directory, we see
```
ERROR: /Users/preston/git/bazel-rust-guided-experiment/stage-2-crates-vendor/3rd-party/BUILD.bazel:6:14: no such target '//:Cargo.toml': target 'Cargo.toml' not declared in package ''; however, a source file of this name exists.  (Perhaps add 'exports_files(["Cargo.toml"])' to /BUILD?)
```
when running `bazel run //3rd-party:crates_vendor`.
Following this suggestion works!

> :eyes: Note that I skipped over undoing some earlier steps
> to get this target to work.
> I made this decision because it was a temporary measure to dig
> ourselves out of a hole that we got into:
> as soon as we were out, I removed it again.

The final version of our `backend/BUILD.bazel` and `3rd-party/BUILD.bazel` are:
```python
exports_files(["Cargo.toml"])

load("@rules_rust//rust:defs.bzl", "rust_binary")
load("//3rd-party/crates:defs.bzl", "all_crate_deps")

rust_binary(
    name = "hello_world",
    srcs = ["src/main.rs"],
    deps = all_crate_deps(normal = True,),
)
```
```python
load("@rules_rust//crate_universe:defs.bzl", "crates_vendor")

crates_vendor(
    name = "crates_vendor",
    manifests = ["//backend:Cargo.toml"],
    mode = "local",
    vendor_path = "crates",
    tags = ["manual"],
)
```

Building our backend target again works!
And uses our vendored dependencies.

## What did we do?

After a little bit of struggling with mixed `BUILD` files,
we added a new `BUILD` file just for vendoring.
If we vendor front-end dependencies, they can go here too!
With this new separation, we got bazel building the project
with our vendored dependencies.

https://github.com/prestontw/bazel-rust-guided-experiment/pull/2
is all of the steps we did but condensed down to avoid
the experimentation and flailing:
- we picked going with the `3rd-party` approach from the beginning,
- we updated out `.cargo/config.toml` file to point to the new directory,
- we revendored with `cargo vendor --versioned-dirs` to the new location
to mimimize noise of actually vendoring.

## What's next?

Let's go through the process of upgrading `rust_rules`
since a new version came out since I started writing this!
