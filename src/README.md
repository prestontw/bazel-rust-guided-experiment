# bazel-rust-guided-experiment

This article is about incorporating the [Bazel build tool](https://bazel.build) into a
Rust project so that both `cargo` and `bazel`-based builds, tests, and vendoring
work. Specifically, this is a Rust project that makes use of vendored
dependencies. This integration works out really nicely, but there were some
painful details that I ran into. I'm writing this article to

1. Highlight those pain points and what I should have done instead,
1. Give a quick demo on using `bazel` and how `cargo` and `bazel` work together to vendor dependencies, and
1. Point out opportunities to improve the `rules_rust` documentation.

I'm trying to keep this article focused on relevant issues that might come up
with integrating `bazel`. There were separate issues that came up due to
"incorrect" directory structure, using the wrong `bazel` rules, etc. Where I
tried something that didn't work, I will point it out as a kind of warning sign.

> 👀 And I will point out places that might be problems in the future, or,
> in general, interesting things that you should think about to keep reading
> interactive.

> 🤦‍♂️ And I will respond like this when I made the wrong decision or ran
> into a sharp edge.

## Motivation---Why Bazel?

Rust compilation times are a common complaint. There have been significant
improvements here in raw Rust compiler performance,
but we can achieve even bigger
savings by caching intermediate results between compilation runs. We can see the
value of this strategy in the push for turning on incremental mode for the Rust
compiler. Unfortunately, this isn't available everywhere that we might want to
build Rust code: local Docker, as part of a CI pipeline, etc.

My path to getting to Bazel has been somewhat circuitous. Here are some tools,
blogs, and comments that got me here:

1. [cargo-chef](https://github.com/LukeMathWalker/cargo-chef) is a tool to
   improve caching between Docker builds for Rust projects. This formalizes a
   process of turning a Rust project into a hollow skeleton just including its
   remote dependencies. This stems from the observation that the remote dependencies don't change
   frequently, so compiling them once as a Docker layer could be a good source of caching.

   [cargo-chef](https://github.com/LukeMathWalker/cargo-chef#limitations-and-caveats)
   lists some limitations, but they all really boil down to a simple caching
   strategy of one mega cache that only includes external dependencies. Local
   dependencies inside of the project are rebuilt from scratch each time,
   which is a major pain point when they dominate compilation times. This is
   something that I ran into at my previous company: there were some other
   issues that using `cargo-chef` presented (mostly around patched dependencies)
   but the main one was that we really wanted to cache local dependencies.

1. [fasterthanlime's blog on his ideal Rust setup](https://fasterthanli.me/articles/my-ideal-rust-workflow).
   I really enjoy Amos's articles. I really enjoy the depth, the narrative, the
   content, and the characters...

   > 👀 What do you mean?

   > 🤦‍♂️ You don't... see it?

   > 👀 Ha ha... No.

   I really enjoyed that article in particular because it had a lot of stuff I could
   ~~steal~~ look into for my own Rust development. He has a specific section on
   [CI](https://fasterthanli.me/articles/my-ideal-rust-workflow#circleci) which includes:

   > ... but
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

1. I saw a [comment on Reddit](https://www.reddit.com/r/rust/comments/ua09tc/comment/i5w7n6g/?utm_source=share&utm_medium=web2x&context=3) for an alternative to `sccache`.
   The parent post was about a Rust build tool, Fleet, that claims to lead to 5x faster build times.
   `bitemyapp` comments on various ways the tool achieves its speedups
   and offers some of their own thoughts and advice on optimizing Rust build times.
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
and bazel has a reputation of struggling with remote dependencies,
so I wanted to see how easy it was to get bazel working in that type of project setup.

I also wanted to see how bazel and cargo can work together,
since I enjoy using cargo locally.

## Structure of this repo

Each child directory is the entire completed state of the project for that particular goal.
The readme for each child
will go into detail on what we want to accomplish,
the steps from the previous completed goal to the current target goal,
and any errors we encounter along the way.
The general flow is

0. [The initial rust project](stage-0.html),
1. [Building with bazel](stage-1.html),
1. [Vendoring with bazel](stage-2.html), and finally
1. [Wrapping up](stage-3.html).

## TL;DR

If you want to vendor your dependencies locally,
start from the [vendor local manifests (Cargo.toml files) example](https://github.com/bazelbuild/rules_rust/tree/main/examples/crate_universe/vendor_local_manifests).

If you are migrating an existing project, make sure that
your Rust code is not at the root level of the repository.
(This is likely if you are in a multi-lingual project
where using bazel would lead to cross-language builds,
and potentially less likely if you are working in a Rust-only project.)

Also make sure that you name your local dependency `rust_library` targets
the same as their crate name.

The bazel slack rust channel seems to be active and I found
people there to be very helpful! Thank you so much to them!

### Is it worth the complexity?

If you're working just in one language, I don't know.
If you spend a lot of time building, maybe!
If your project is kind of small and build times are't significant,
maybe stick with language tooling until that becomes painful.

> 🤷‍♂️ But I've also heard the advice that you should switch to bazel before this point.

If you work in a project that has different languages all working together,
bazel is a nice unified build tool that tries (depending on support level)
to work for all of them.
In this case, I think the smart builds and caching is worth the extra work
for large enough projects.

For using Rust and Bazel together specifically,
the experience falls somewhere in between using `cargo`
and using `bazel` with other languages.
Rust and Bazel have benefits that some other bazel ecosystems really struggle with.
On the other hand, bazel (or `rules_rust`) is more opinionated than `cargo`
when it comes to directory structures.
Most of my difficulty in this experiment was making my directory structure
match what bazel (or `rules_rust`) expected.
But after that struggle, I got a bazel environment that
still cooperates with native rust tooling and IDE integration and
handles transitive dependencies nicely.
Both of these are problems that people have with bazel
(watch some Bazel talks on Youtube and see how many of them talk about
getting bazel to work with IntelliJ).
Now that I've gone through this struggle,
hopefully other people can get the cool benefits
without all of the flailing I did.

This guide is really only possible because we are standing on the shoulders of giants---`rules_rust`
developers really put in a lot of work to make these
typically painful `bazel` issues into non-issues.
But I think there still could be some work on building
projects that work with `cargo` in `bazel`.

> 🤦‍♂️ Maybe `cargo-raze`, another tool for integrating `cargo` and `bazel`
> works better for this.
> But I also ran into problems using `cargo-raze`...
> Maybe I'll look into it again another time.
>
> I think, really, the tradeoff comes down to the work needed to adapt
> your directory structure to what `bazel` is expecting
> vs the caching that `bazel` gives you.

There is something else to keep in mind regarding using `bazel`:
we have had to add these build files ourselves to get this level of caching.
We are taking on a maintenance cost.
Bazel is also not completely stable---it is under active development.
Some of their plans (besides general improvements) include
getting rid of WORKSPACE files
and replacing them with mod files or something
(see https://bazel.build/docs/bzlmod),
so there might be some pretty signficant organizational changes.
That being said, this is a nice tool for coordinating with other languages,
and there's a certain sense of coordination between companies too.
Having a bunch of companies use this tool means that everyone can benefit
(but that could also mean that there are a lot of changes to keep up with).

Separately, bazel can also be used to [minimise the number of potential dependencies](https://youtu.be/5OjqD-ow8GE?t=2089),
providing a road bump from your code base from turning into a ball of mud.
In this scenario, bazel is actually a tool against unbounded complexity of dependencies.
