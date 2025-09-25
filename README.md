# CHIP-8

CHIP-8 emulator written in Rust

<img width="974" height="512" alt="image" src="https://github.com/user-attachments/assets/e6939881-960d-45d6-aa7a-3c1631591af3" />


## Requirements
Install SDL2:<br/>
```
sudo apt-get install libsdl2-dev
```

Or uncomment `"static-link"` and `"use-vcpkg"` in `Cargo.toml` and install with vcpkg:
```
[dependencies.sdl2]
version = "0.38"
default-features = false
features = [ "static-link", "use-vcpkg"]
```

```
cargo install cargo-vcpkg
cargo vcpkg build
```

## Run
```
cargo run rompath
```
or

```
cargo build
./target/debug/chip-8 rompath
```
