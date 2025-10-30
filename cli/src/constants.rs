// In cli/src/constants.rs

pub const CUSTOM_OCI_IMAGE_OPTION: &str = "[ Type Custom OCI Image ]";

// A list of common images from the distrobox compatibility list
// (shortname, full_url)
pub const DISTROBOX_IMAGES: &[(&str, &str)] = &[
    ("fedora-toolbox:latest", "registry.fedoraproject.org/fedora-toolbox:latest"),
    ("fedora-toolbox:39", "registry.fedoraproject.org/fedora-toolbox:39"),
    ("fedora-toolbox:40", "registry.fedoraproject.org/fedora-toolbox:40"),
    ("alpine:latest", "docker.io/library/alpine:latest"),
    ("ubuntu:latest", "docker.io/library/ubuntu:latest"),
    ("ubuntu:22.04", "docker.io/library/ubuntu:22.04"),
    ("ubuntu:24.04", "docker.io/library/ubuntu:24.04"),
    ("debian:latest", "docker.io/library/debian:latest"),
    ("debian:12", "docker.io/library/debian:12"),
    ("opensuse-tumbleweed", "registry.opensuse.org/opensuse/tumbleweed:latest"),
    ("bluefin-cli:latest", "ghcr.io/ublue-os/bluefin-cli:latest"),
    ("aurora-cli:latest", "ghcr.io/ublue-os/aurora-cli:latest"),
];