### Basic Control Scheme for ASV Using Mavlink Protocol

Basic Mavlink-based control system for the Aqualink ASV. 

Can cross-compile for Jetson Nano using
```bash
$ rustup target add aarch64-unknown-linux-gnu # Adds target
$ cargo build --target=aarch64-unknown-linux-gnu
```

The linker in the `.cargo/config` file must be 
```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

which can be installed on Ubuntu using 
```bash
$ sudo apt-get install gcc-aarch64-linux-gnu
```

Please note that naming conventions can be confusing; the `aarch64` platform is occasionally also known as `armv8`, which is why it doesn't follow the same naming convention as that used by the Raspberry Pi 3 that is included on the BlueROV2, which uses the `armv7` target. 