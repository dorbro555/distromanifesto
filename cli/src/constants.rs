// In cli/src/constants.rs

pub const CUSTOM_OCI_IMAGE_OPTION: &str = "[ Type Custom OCI Image ]";

// A list of common images from the distrobox compatibility list
pub const DISTROBOX_IMAGES: &[&str] = &[
    "registry.fedoraproject.org/fedora-toolbox:latest",
    "registry.fedoraproject.org/fedora-toolbox:38",
    "registry.fedoraproject.org/fedora-toolbox:39",
    "registry.fedoraproject.org/fedora-toolbox:40",
    "docker.io/library/alpine:latest",
    "docker.io/library/ubuntu:latest",
    "docker.io/library/ubuntu:22.04",
    "docker.io/library/ubuntu:24.04",
    "docker.io/library/debian:latest",
    "docker.io/library/debian:12",
    "registry.opensuse.org/opensuse/tumbleweed:latest",
    "ghcr.io/ublue-os/bluefin-cli:latest",
    "ghcr.io/ublue-os/aurora-cli:latest",
];