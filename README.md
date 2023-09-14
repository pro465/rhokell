# Rhokell
A very simple functional programming language I made to not get scared about FP just because of Haskell's scary symbols.
It also has I/O, see the smallfuck example for a demo.

# Get started
```shell
cargo install --git https://github.com/pro465/rhokell
echo "(hello) = (world);" > hello_world.rhk
echo "(hello)" | rhokell -r hello_world.rhk
```
You should see `(world)` getting output on the REPL.

# License 
all files in [./examples](./examples) are available under [public domain](https://creativecommons.org/publicdomain/zero/1.0/), while the interpreter is licensed under [GPLv3.0](/LICENSE)
