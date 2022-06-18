# Stage 2: Let's vendor our dependencies through bazel!

To recap, we can build our project through bazel! Whoopee!
But it isn't making use of our vendored dependencies that we got
through `cargo vendor`! Let's try to remedy this.
This is our goal for this stage.

Watch out, there is a little bit of flailing here.
Maybe not narratively

> :facepalm: Hopefully!

but this is something that I struggled with.
While trying to get things working,
I might move from attempt to attempt quickly.

## How did we get here?

As alluded to before, there is a rule for vendoring dependencies:
https://bazelbuild.github.io/rules_rust/crate_universe.html#crates_vendor{:target="_blank"}.
However, I don't really get its example.
It specifies an annotation---is this related to a dependency?
Does this supplant `Cargo.toml` information?

> :eyes: I think there is a difference between minimal examples
> to help beginners and more full-featured examples to show what
> is possible. One thing that could help a beginner with filtering
> out some of the more advanced options is: documentation!
> In the context of code in examples, this most likely lives
> as comments. Having comments in this example would have been
> really helpful---it seemed so different from what I wanted
> that I ended up getting scared away.

This is too much. But, there are other worked examples!
Let's try copying [an example](https://github.com/bazelbuild/rules_rust/blob/0265c293f195a59da45f83aafcfca78eaf43a4c5/examples/crate_universe/vendor_local_manifests/BUILD.bazel) from the `rules_rust` repo.
This is for our `BUILD` file in our `backend` directory.
We will tweak it slightly becase we want to reuse the vendored
dependencies we set up in `cargo vendor`:

```python
{{ #include ./stage-2-crates-vendor/backend/.build1.bazel }}
```

This leads to error messages saying that
`3rd-party/crates` is not a package. Hmm.
Maybe it's due to a kind of cyclic issue?
`crates_vendor` would create the `//3rd-party/crates` package,
but then we try to use that same package potentially
before it's created since it's in the same file.
That's weird. Let's try upgrading our version of `rules_rust`
and see if that solves it for us.

> :eyes: `rules_rust` is now on version `0.5.0`
> while the documentation mentions `0.2.0`.
> To further complicate things, while I was writing this example,
> the most recent release was version `0.4.0`.
>
> Keeping documentation up to date with code is hard,
> as evinced by the fact that this guide is immediately behind!
> I don't feel like this is a complaint regarding `rust_rules`'s documentation
> because I can go and improve the situation through pull requests!

After updating our version (and syncing our lockfile),
we see the same error.
Maybe we can break the cycle by removing the code that relies
on `//3rd-party/crates`.
Now our `backend` `BUILD` file looks like

```python
{{ #include ./stage-2-crates-vendor/backend/.build2.bazel }}
```

Note that we need to use `bazel run` instead of `bazel build`.

This actually works after one small correction.
There's an issue with some versions of `tokio` that leads
to errors on usage with `bazel`.
The [`vendor_local_manifests` example](https://github.com/bazelbuild/rules_rust/blob/0265c293f195a59da45f83aafcfca78eaf43a4c5/examples/crate_universe/vendor_local_manifests/Cargo.toml#L8) has a fix---let's
borrow other people's workarounds.

There's a bigger philosophical issue here, though.
This places the dependencies inside of `backend`.
Whoops! We originally placed dependencies in the root level `3rd-party`
to enable sharing between different rust projects,
potentially outside of the `backend` directory.
If we want some of our utilities to use the same
version as we do in our backend, this nested directory
structure isn't condusive to that.

> :eyes: The utilities dependency issue is just an example.
> Having other smaller services
> in a `services` directory would also motivate this sharing.

Trying `/3rd-party/crates` as the `vendor_path` gives me permissions issues,
which makes me think that it's not actually true that

> Absolute paths will be treated as relative to the workspace root

I've reported some of this difficulty in [this GitHub issue](https://github.com/bazelbuild/rules_rust/issues/1341).

Just as an experiment, let's try specifying the full path to `3rd-party/crates`.
This "works" as in it's able to vendor the dependencies.

The full path won't work in general as soon as we go to another machine,
so let's try a relative path!
Let's try `../3rd-party/crates` and see if it works.

> :facepalm: I think at this point,
> https://bazelbuild.github.io/rules_rust/crate_universe.html#crates_vendor
> is starting to make some more sense.
> We have a specific target for vendoring the dependencies,
> and, for some reason,
> the example that we've been following combined that target with
> the general rust library target.

Anyway, let's try loading from it. Filling back in code that we removed,
our `BUILD` file now ends with

```python
{{ #include ./stage-2-crates-vendor/backend/.build3.bazel:10: }}
```

Finally, let's build just to make sure that everything works.

```
ERROR: /Users/preston/git/bazel-rust-guided-experiment/stage-2-crates-vendor/backend/BUILD.bazel:13:12: //backend:hello_world: invalid label '//backend/../3rd-party/crates/axum-0.5.6:axum' in element 0 of attribute 'deps' in 'rust_binary' rule: invalid package name 'backend/../3rd-party/crates/axum-0.5.6': package name component contains only '.' characters
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

Now that we aren't using `crate_index`,
let's remove it from our `WORKSPACE` file.
It now ends with

```python
{{ #include ./stage-2-crates-vendor/WORKSPACE.bazel:24:26}}
```

The final version of our `backend/BUILD.bazel` and `3rd-party/BUILD.bazel` are:

```python
{{ #include ./stage-2-crates-vendor/backend/BUILD.bazel }}
```

```python
{{ #include ./stage-2-crates-vendor/3rd-party/BUILD.bazel:4: }}
```

Building our backend target again works!
And uses our vendored dependencies.

### Cleaning up previous steps

> :eyes: Hmm, look at that, we don't need some of these files anymore!

> :facepalm: Yes, this actually took me [reaching out on slack](https://bazelbuild.slack.com/archives/CSV56UT0F) to realize.
> Because we aren't using `crate_index` anymore,
> we can also remove our old `BUILD.bazel` at the root level
> and the `Cargo.Bazel.lock`.

## What did we do?

After a little bit of struggling with mixed `BUILD` files,
we added a new `BUILD` file just for vendoring.
If we vendor front-end dependencies, they can go here too!
With this new separation, we got bazel building the project
with our vendored dependencies.

https://github.com/prestontw/bazel-rust-guided-experiment/pull/2
is all of the steps we did but condensed down to avoid
the experimentation and flailing:

- we picked going with the `3rd-party` `BUILD` file from the beginning,
- we exposed `backend`'s `Cargo.toml`, and
- we updated out `.cargo/config.toml` file to point to the new directory.

## What's next?

Let's go through the process of upgrading `rust_rules`
since a new version came out since I started writing this!
