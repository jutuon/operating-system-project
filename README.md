# Operating system project

## Features

* 32-bit x86
* Identity mapped PAE paging
* IDT and GDT
* Programmable interrupt controller (Intel 8259A)
* VGA text mode

## Building and running

1. Install Rust

2. Install other dependencies

* <https://github.com/rust-osdev/cargo-xbuild>
* <https://github.com/casey/just>

```
rustup component add rust-src
cargo install --vers=0.5.1 cargo-xbuild
cargo install just
sudo apt install qemu-system-x86 xorriso grub2-common
```
4. `git clone https://github.com/jutuon/operating-system-project`

5. `cd operating-system-project`

6. Run Justfile with `just`. By default this builds the
operating system and starts it in QEMU.

### Bochs x86 emulator

1. Install Bochs.

```
sudo apt install bochs bochs-sdl
```

2. `just run-bochs`

## License

This project is licensed under terms of

* Apache 2.0 license or
* MIT license

at your opinion.
