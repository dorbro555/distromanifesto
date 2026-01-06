---
sidebar_position: 1
---

# The Manifest (.ini)

The core of Distromanifesto is the **Manifest File**. Instead of running long, complex CLI commands every time you want to create a container, you define your desired state in a simple configuration file.

`dimo` uses the standard INI format, where **Sections** represent containers and **Keys** represent configuration options.

## Syntax Rules

To ensure your manifest is valid, adhere to these strict syntax rules enforced by `dimo verify`:

1.  **Sections are Containers**: Each header like `[my-box]` defines a new container named "my-box".
2.  **No Spaces**: Keys and values must **not** have spaces around the equals sign.
    * ✅ `image=archlinux:latest`
    * ❌ `image = archlinux:latest`
3.  **Required Keys**: Every section must contain at least an `image` (to create new) or `clone` (to duplicate existing).

## Supported Keys

Here is the complete list of keys supported by `dimo`.

### Core Configuration

| Key | Type | Description |
| :-- | :--- | :---------- |
| `image` | String | The OCI image to use (e.g., `ubuntu:22.04`, `archlinux:latest`). |
| `home` | String | Custom path for the container's home directory. Defaults to your host home. |
| `pull` | Bool | If `true`, always pull the latest image before creating. |
| `root` | Bool | If `true`, the container will run as root. |
| `clone` | String | Name of an existing container to clone from. |

### Integration & Hardware

| Key | Type | Description |
| :-- | :--- | :---------- |
| `nvidia` | Bool | Enable NVIDIA GPU integration. |
| `init` | Bool | Use a specialized init system (useful for systemd support). |
| `volume` | List | Mount additional volumes. Format: `/host/path:/container/path`. |

### Hooks & customization

| Key | Type | Description |
| :-- | :--- | :---------- |
| `additional_packages` | List | Space-separated list of packages to install during creation. |
| `init_hooks` | List | Shell commands to run **inside** the container after initialization. |
| `pre_init_hooks` | List | Shell commands to run **inside** the container before initialization. |
| `additional_flags` | List | Any extra flags to pass directly to the underlying `distrobox create` command. |

### Isolation (Unshare)

Advanced users can control namespace sharing with these boolean flags:
* `unshare_ipc`
* `unshare_netns`
* `unshare_process`
* `unshare_devsys`
* `unshare_all`

## Example Manifest

Here is a robust example defining two different containers in a single file:

```ini
# A generic development environment
[dev-box]
image=archlinux:latest
pull=true
init=true
additional_packages=git vim neofetch
init_hooks=echo "Welcome to your dev box!"

# A secluded gaming container with GPU support
[gaming-box]
image=ubuntu:22.04
nvidia=true
# Use a custom home directory managed by dimo
home=~/.distromanifesto/homes/gaming-box
```

## Verifying Your Manifest
Before assembling, you can check your syntax using the verify command:

```bash
dimo verify ./my-manifest.ini
```