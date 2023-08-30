# rhokell
A very simple functional programming language I made to not get scared about FP just because of Haskell's scary symbols.
Currently it is only a REPL, however I plan to add I/O and turn it into a "proper" language.

# get started
```shell
cargo install --git https://github.com/pro465/rhokell
echo "hello() = world();" > hello_world.rhk
echo "hello()" | rhokell hello_world.rhk
```
You should see `world()` getting output on the REPL.
