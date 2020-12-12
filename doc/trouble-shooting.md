# Troube shooting

### Usage when adding items
Please use markdown level 2 headers ->  "## HERE SOMETHING" to start an issue to the trouble shoot and level 3, 4 or 5 to trouble shoot that level 2 issue.

## The cargo install command failed. Any ideas what to do next?
> Distilled from [issue 4](https://github.com/pandoracore/bitcoin-pro/issues/4), case was on Ubuntu 18.04

Suppose this command fails:
`$ cargo install bitcoin-pro --version 0.1.0-beta.1`

At this point:
```
Compiling glade v0.1.0-alpha.2
error[E0554]: #![feature] may not be used on the stable release channel
--> /home/hvancann/.cargo/registry/src/github.com-1ecc6299db9ec823/glade-0.1.0-alpha.2/src/lib.rs:15:1
|
15 | #![feature(try_trait)]
| ^^^^^^^^^^^^^^^^^^^^^^


error: aborting due to previous error
For more information about this error, try rustc --explain E0554.
error: could not compile glade.
```

Then try \
`$rustc --explain E0554`  -  it gives the explanation of how to overcome this issue:

```
Feature attributes are only allowed on the nightly release channel. Stable or
beta compilers will not comply.
...
If you need the feature, make sure to use a nightly release of the compiler
(but be warned that the feature may be removed or altered in the future).
```

### Use the nightly release fo the compiler

`$ rustup install nightly`   -  If you get an error look at this [side step](#side-step-rustup-install)\
and\
`$ rustup default nightly`\
Now continue with\
`$ cargo install bitcoin-pro --version 0.1.0-beta.1`

#### Side step rustup install
if you haven got rustup installed, first this command:\
`sudo snap install rustup`

You might get:
```error: This revision of snap "rustup" was published using classic confinement
       and thus may perform arbitrary system changes outside of the security
       sandbox that snaps are usually confined to, which may put your system at
       risk.

       If you understand and want to proceed, repeat the command including
       --classic.
```
Or perform:
`sudo snap refresh --classic rustup`