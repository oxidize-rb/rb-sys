# Background and Concepts

## What is a native extension?

Typically, Ruby code is compiled to a special instruction set which executes on a stack-based virtual machine. You can
see what these instructions look like by running:

```sh
$ ruby --dump=insns -e '2 + 3'
== disasm: #<ISeq:<main>@-e:1 (1,0)-(1,5)> (catch: FALSE)
0000 putobject         2           (   1)[Li]
0002 putobject         3
0004 opt_plus          <calldata!mid:+, argc:1, ARGS_SIMPLE>[CcCr]
0006 leave
```

In this example, `2` and `3` are pushed onto the stack, and then `opt_plus` performs the addition.

For a native gem, we bypass this mechanism entirely and instead exposes native machine code to Ruby. In our native code,
we can use the [Ruby C API] to interact with the Ruby VM.

## How are native Gems loaded?

Under the hood, native extensions are compiled as shared libraries (`.so`, `.bundle`, etc.). When you
`require 'some_gem'`, if Ruby finds a `some_gem.(so|bundle|lib)`, the shared library loaded on demand using [dlopen] (or
the system equivalent). After that, Ruby will call `Init_some_gem` so the native library can do its magic.

## Why does it work with Rust and not other languages?

C is often referred to as the "lingua franca" of the programming language world, and Rust is fluent. Rust can compile
functions to be compatible with the C calling conventions, and align items in memory in a way that C understands. Rust
also does not have a garbage collector, which makes integration signifcantly easier.

When Ruby loads a gem extension written in Rust, it has no idea the gem is actually written in Rust. Due to Rust's
robust C FFI, you can code anything in Rust that you could with C.

[vincius stock's excellent guide]: https://dev.to/vinistock/creating-ruby-native-extensions-kg1
[ruby c api]: https://docs.ruby-lang.org/en/3.1/extension_rdoc.html
[dlopen]: https://man7.org/linux/man-pages/man3/dlopen.3.html
[rustonomicon docs]: https://doc.rust-lang.org/nomicon/ffi.html#calling-rust-code-from-c
