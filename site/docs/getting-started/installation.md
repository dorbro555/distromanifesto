---
sidebar_position: 1
---

# Installation

`distromanifesto` (alias: `dimo`) is a Rust-based CLI tool. It requires a container runtime and Distrobox to function.

## Prerequisites

Before installing `dimo`, ensure you have the following installed:

1.  **Distrobox**: The core container manager.
    * [Installation Guide](https://distrobox.it/#installation)
2.  **Container Runtime**: Either **Podman** (recommended) or **Docker**.
    * *Tip: If using Docker, ensure you can run it without sudo.*

## Install via Cargo

The easiest way to install `dimo` is via Rust's package manager.

```bash
cargo install distromanifesto
```

## Build from Source

If you prefer the bleeding edge, you can build directly from the repository:

```bash
# Clone the repository
git clone [https://github.com/dorbro555/distromanifesto.git](https://github.com/dorbro555/distromanifesto.git)
cd distromanifesto

# Build and install
cargo install --path cli
```

## Verification

Once installed, verify that the dimo alias is accessible:

```bash
dimo --help
```