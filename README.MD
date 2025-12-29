# Piston-MC
A Rust library for interacting with the Minecraft game and its related services, such as asset downloading and version management.

## Features
- **Version Manifest Fetching** - Query all Minecraft versions from Mojang's official API
- **Client & Server Downloads** - Download Minecraft client and server JARs with progress tracking
- **Asset Management** - Download and validate game assets (textures, sounds, etc.)
- **Java Runtime Management** - Fetch and install Java runtimes for any platform
- **SHA1 Validation** - Verify file integrity after downloads
- **Parallel Downloads** - Configurable concurrent downloads for faster asset retrieval
- **Progress Reporting** - Real-time download progress via async channels
- **Async First** - Built on Tokio for easy integration with asynchronous Rust applications

## Installation
You can install `piston-mc` from the git repostitory by adding this to the `Cargo.toml` file:
```toml
[dependencies]
piston-mc = { git = "https://github.com/Drew-Chase/piston-mc.git" }
```

or by running this command:
```bash
cargo add piston-mc --git https://github.com/Drew-Chase/piston-mc.git
```

## Usage
for usage information you can view the [examples](/examples) in the repository or the [wiki](https://github.com/Drew-Chase/piston-mc/wiki).