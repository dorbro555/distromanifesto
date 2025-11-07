// In cli/src/constants.rs

pub const CUSTOM_OCI_IMAGE_OPTION: &str = "[ Type Custom OCI Image ]";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageItem {
    pub display_name: &'static str, // e.g., "40", "latest"
    pub full_url: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageGroup {
    pub name: &'static str,
    pub images: &'static [ImageItem],
}

#[derive(Debug, Clone, Copy)]
pub struct ImageTab {
    pub title: &'static str,
    pub groups: &'static [ImageGroup],
}

// (shortname, full_url)

// --- Custom Option ---
const CUSTOM_IMAGE: &[ImageItem] = &[
    ImageItem { display_name: CUSTOM_OCI_IMAGE_OPTION, full_url: "" }
];

// --- Data for TOOLBOX Tab ---
const TOOLBOX_ALMALINUX: &[ImageItem] = &[
    ImageItem { display_name: "8", full_url: "quay.io/toolbx-images/almalinux-toolbox:8" },
    ImageItem { display_name: "9", full_url: "quay.io/toolbx-images/almalinux-toolbox:9" },
    ImageItem { display_name: "latest", full_url: "quay.io/toolbx-images/almalinux-toolbox:latest" },
];
const TOOLBOX_ALPINE: &[ImageItem] = &[
    ImageItem { display_name: "3.16", full_url: "quay.io/toolbx-images/alpine-toolbox:3.16" },
    ImageItem { display_name: "3.17", full_url: "quay.io/toolbx-images/alpine-toolbox:3.17" },
    ImageItem { display_name: "3.18", full_url: "quay.io/toolbx-images/alpine-toolbox:3.18" },
    ImageItem { display_name: "3.19", full_url: "quay.io/toolbx-images/alpine-toolbox:3.19" },
    ImageItem { display_name: "3.20", full_url: "quay.io/toolbx-images/alpine-toolbox:3.20" },
    ImageItem { display_name: "edge", full_url: "quay.io/toolbx-images/alpine-toolbox:edge" },
    ImageItem { display_name: "latest", full_url: "quay.io/toolbx-images/alpine-toolbox:latest" },
];
const TOOLBOX_AMAZONLINUX: &[ImageItem] = &[
    ImageItem { display_name: "2", full_url: "quay.io/toolbx-images/amazonlinux-toolbox:2" },
    ImageItem { display_name: "2023", full_url: "quay.io/toolbx-images/amazonlinux-toolbox:2023" },
    ImageItem { display_name: "latest", full_url: "quay.io/toolbx-images/amazonlinux-toolbox:latest" },
];
const TOOLBOX_ARCHLINUX: &[ImageItem] = &[
    ImageItem { display_name: "latest", full_url: "quay.io/toolbx/arch-toolbox:latest" },
];
const TOOLBOX_BAZZITE: &[ImageItem] = &[
    ImageItem { display_name: "arch", full_url: "ghcr.io/ublue-os/bazzite-arch:latest" },
    ImageItem { display_name: "arch-gnome", full_url: "ghcr.io/ublue-os/bazzite-arch-gnome:latest" },
];
const TOOLBOX_CENTOS: &[ImageItem] = &[
    ImageItem { display_name: "stream8", full_url: "quay.io/toolbx-images/centos-toolbox:stream8" },
    ImageItem { display_name: "stream9", full_url: "quay.io/toolbx-images/centos-toolbox:stream9" },
    ImageItem { display_name: "latest", full_url: "quay.io/toolbx-images/centos-toolbox:latest" },
];
const TOOLBOX_DEBIAN: &[ImageItem] = &[
    ImageItem { display_name: "10", full_url: "quay.io/toolbx-images/debian-toolbox:10" },
    ImageItem { display_name: "11", full_url: "quay.io/toolbx-images/debian-toolbox:11" },
    ImageItem { display_name: "12", full_url: "quay.io/toolbx-images/debian-toolbox:12" },
    ImageItem { display_name: "testing", full_url: "quay.io/toolbx-images/debian-toolbox:testing" },
    ImageItem { display_name: "unstable", full_url: "quay.io/toolbx-images/debian-toolbox:unstable" },
    ImageItem { display_name: "latest", full_url: "quay.io/toolbx-images/debian-toolbox:latest" },
];
const TOOLBOX_FEDORA: &[ImageItem] = &[
    ImageItem { display_name: "37", full_url: "registry.fedoraproject.org/fedora-toolbox:37" },
    ImageItem { display_name: "38", full_url: "registry.fedoraproject.org/fedora-toolbox:38" },
    ImageItem { display_name: "39", full_url: "registry.fedoraproject.org/fedora-toolbox:39" },
    ImageItem { display_name: "40", full_url: "registry.fedoraproject.org/fedora-toolbox:40" },
    ImageItem { display_name: "41", full_url: "quay.io/fedora/fedora-toolbox:41" },
    ImageItem { display_name: "rawhide", full_url: "quay.io/fedora/fedora-toolbox:rawhide" },
];
const TOOLBOX_OPENSUSE: &[ImageItem] = &[
    ImageItem { display_name: "latest", full_url: "registry.opensuse.org/opensuse/distrobox:latest" },
];
const TOOLBOX_REDHAT: &[ImageItem] = &[
    ImageItem { display_name: "8", full_url: "registry.access.redhat.com/ubi8/toolbox" },
    ImageItem { display_name: "9", full_url: "registry.access.redhat.com/ubi9/toolbox" },
];
const TOOLBOX_ROCKYLINUX: &[ImageItem] = &[
    ImageItem { display_name: "8", full_url: "quay.io/toolbx-images/rockylinux-toolbox:8" },
    ImageItem { display_name: "9", full_url: "quay.io/toolbx-images/rockylinux-toolbox:9" },
    ImageItem { display_name: "latest", full_url: "quay.io/toolbx-images/rockylinux-toolbox:latest" },
];
const TOOLBOX_UBUNTU: &[ImageItem] = &[
    ImageItem { display_name: "16.04", full_url: "quay.io/toolbx/ubuntu-toolbox:16.04" },
    ImageItem { display_name: "18.04", full_url: "quay.io/toolbx/ubuntu-toolbox:18.04" },
    ImageItem { display_name: "20.04", full_url: "quay.io/toolbx/ubuntu-toolbox:20.04" },
    ImageItem { display_name: "22.04", full_url: "quay.io/toolbx/ubuntu-toolbox:22.04" },
    ImageItem { display_name: "24.04", full_url: "quay.io/toolbx/ubuntu-toolbox:24.04" },
    ImageItem { display_name: "latest", full_url: "quay.io/toolbx/ubuntu-toolbox:latest" },
];
const TOOLBOX_WOLFI: &[ImageItem] = &[
    ImageItem { display_name: "latest", full_url: "quay.io/toolbx-images/wolfi-toolbox:latest" },
];
const TOOLBOX_UBLUE: &[ImageItem] = &[
    ImageItem { display_name: "bluefin-cli", full_url: "ghcr.io/ublue-os/bluefin-cli" },
    ImageItem { display_name: "ubuntu-toolbox", full_url: "ghcr.io/ublue-os/ubuntu-toolbox" },
    ImageItem { display_name: "fedora-toolbox", full_url: "ghcr.io/ublue-os/fedora-toolbox" },
    ImageItem { display_name: "wolfi-toolbox", full_url: "ghcr.io/ublue-os/wolfi-toolbox" },
    ImageItem { display_name: "arch-distrobox", full_url: "ghcr.io/ublue-os/arch-distrobox" },
    ImageItem { display_name: "powershell-toolbox", full_url: "ghcr.io/ublue-os/powershell-toolbox" },
];

// --- Data for STANDARD Tab ---
const STANDARD_ALMALINUX: &[ImageItem] = &[
    ImageItem { display_name: "8", full_url: "docker.io/library/almalinux:8" },
    ImageItem { display_name: "9", full_url: "docker.io/library/almalinux:9" },
];
const STANDARD_ALPINE: &[ImageItem] = &[
    ImageItem { display_name: "3.15", full_url: "docker.io/library/alpine:3.15" },
    ImageItem { display_name: "3.16", full_url: "docker.io/library/alpine:3.16" },
    ImageItem { display_name: "3.17", full_url: "docker.io/library/alpine:3.17" },
    ImageItem { display_name: "3.18", full_url: "docker.io/library/alpine:3.18" },
    ImageItem { display_name: "3.19", full_url: "docker.io/library/alpine:3.19" },
    ImageItem { display_name: "3.20", full_url: "docker.io/library/alpine:3.20" },
    ImageItem { display_name: "edge", full_url: "docker.io/library/alpine:edge" },
    ImageItem { display_name: "latest", full_url: "docker.io/library/alpine:latest" },
];
const STANDARD_AMAZONLINUX: &[ImageItem] = &[
    ImageItem { display_name: "1", full_url: "public.ecr.aws/amazonlinux/amazonlinux:1" },
    ImageItem { display_name: "2", full_url: "public.ecr.aws/amazonlinux/amazonlinux:2" },
    ImageItem { display_name: "2023", full_url: "public.ecr.aws/amazonlinux/amazonlinux:2023" },
];
const STANDARD_ARCHLINUX: &[ImageItem] = &[
    ImageItem { display_name: "latest", full_url: "docker.io/library/archlinux:latest" },
];
const STANDARD_BLACKARCH: &[ImageItem] = &[
    ImageItem { display_name: "latest", full_url: "docker.io/blackarchlinux/blackarch:latest" },
];
const STANDARD_CENTOS: &[ImageItem] = &[
    ImageItem { display_name: "stream8", full_url: "quay.io/centos/centos:stream8" },
    ImageItem { display_name: "stream9", full_url: "quay.io/centos/centos:stream9" },
];
const STANDARD_WOLFI: &[ImageItem] = &[
    ImageItem { display_name: "latest", full_url: "cgr.dev/chainguard/wolfi-base:latest" },
];
const STANDARD_CLEARLINUX: &[ImageItem] = &[
    ImageItem { display_name: "latest", full_url: "docker.io/library/clearlinux:latest" },
    ImageItem { display_name: "base", full_url: "docker.io/library/clearlinux:base" },
];
const STANDARD_CRYSTAL: &[ImageItem] = &[
    ImageItem { display_name: "latest", full_url: "registry.gitlab.com/crystal-linux/misc/docker:latest" },
];
const STANDARD_DEBIAN: &[ImageItem] = &[
    ImageItem { display_name: "7 (wheezy)", full_url: "docker.io/debian/eol:wheezy" },
    ImageItem { display_name: "10 (buster)", full_url: "docker.io/library/debian:buster" },
    ImageItem { display_name: "11-backports", full_url: "docker.io/library/debian:bullseye-backports" },
    ImageItem { display_name: "12-backports", full_url: "docker.io/library/debian:bookworm-backports" },
    ImageItem { display_name: "stable-backports", full_url: "docker.io/library/debian:stable-backports" },
    ImageItem { display_name: "testing", full_url: "docker.io/library/debian:testing" },
    ImageItem { display_name: "testing-backports", full_url: "docker.io/library/debian:testing-backports" },
    ImageItem { display_name: "unstable", full_url: "docker.io/library/debian:unstable" },
];
const STANDARD_DEEPIN: &[ImageItem] = &[
    ImageItem { display_name: "20 (apricot)", full_url: "docker.io/linuxdeepin/apricot" },
    ImageItem { display_name: "23 (beige)", full_url: "docker.io/linuxdeepin/deepin:beige" },
];
const STANDARD_FEDORA: &[ImageItem] = &[
    ImageItem { display_name: "36", full_url: "quay.io/fedora/fedora:36" },
    ImageItem { display_name: "37", full_url: "quay.io/fedora/fedora:37" },
    ImageItem { display_name: "38", full_url: "quay.io/fedora/fedora:38" },
    ImageItem { display_name: "39", full_url: "quay.io/fedora/fedora:39" },
    ImageItem { display_name: "40", full_url: "quay.io/fedora/fedora:40" },
    ImageItem { display_name: "41", full_url: "quay.io/fedora/fedora:41" },
    ImageItem { display_name: "rawhide", full_url: "quay.io/fedora/fedora:rawhide" },
];
const STANDARD_GENTOO: &[ImageItem] = &[
    ImageItem { display_name: "latest", full_url: "docker.io/gentoo/stage3:latest" },
];
const STANDARD_KDENEON: &[ImageItem] = &[
    ImageItem { display_name: "latest", full_url: "invent-registry.kde.org/neon/docker-images/plasma:latest" },
];
const STANDARD_KALI: &[ImageItem] = &[
    ImageItem { display_name: "rolling", full_url: "docker.io/kalilinux/kali-rolling:latest" },
];
const STANDARD_MINT: &[ImageItem] = &[
    ImageItem { display_name: "21.1", full_url: "docker.io/linuxmintd/mint21.1-amd64" },
];
const STANDARD_NEURODEBIAN: &[ImageItem] = &[
    ImageItem { display_name: "nd100", full_url: "docker.io/library/neurodebian:nd100" },
];
const STANDARD_OPENSUSE: &[ImageItem] = &[
    ImageItem { display_name: "Leap", full_url: "registry.opensuse.org/opensuse/leap:latest" },
    ImageItem { display_name: "Tumbleweed", full_url: "registry.opensuse.org/opensuse/tumbleweed:latest" },
    ImageItem { display_name: "Toolbox", full_url: "registry.opensuse.org/opensuse/toolbox:latest" },
];
const STANDARD_ORACLE: &[ImageItem] = &[
    ImageItem { display_name: "7", full_url: "container-registry.oracle.com/os/oraclelinux:7" },
    ImageItem { display_name: "7-slim", full_url: "container-registry.oracle.com/os/oraclelinux:7-slim" },
    ImageItem { display_name: "8", full_url: "container-registry.oracle.com/os/oraclelinux:8" },
    ImageItem { display_name: "8-slim", full_url: "container-registry.oracle.com/os/oraclelinux:8-slim" },
    ImageItem { display_name: "9", full_url: "container-registry.oracle.com/os/oraclelinux:9" },
    ImageItem { display_name: "9-slim", full_url: "container-registry.oracle.com/os/oraclelinux:9-slim" },
];
const STANDARD_RHEL_UBI: &[ImageItem] = &[
    ImageItem { display_name: "7", full_url: "registry.access.redhat.com/ubi7/ubi" },
    ImageItem { display_name: "8", full_url: "registry.access.redhat.com/ubi8/ubi" },
    ImageItem { display_name: "8-init", full_url: "registry.access.redhat.com/ubi8/ubi-init" },
    ImageItem { display_name: "8-minimal", full_url: "registry.access.redhat.com/ubi8/ubi-minimal" },
    ImageItem { display_name: "9", full_url: "registry.access.redhat.com/ubi9/ubi" },
    ImageItem { display_name: "9-init", full_url: "registry.access.redhat.com/ubi9/ubi-init" },
    ImageItem { display_name: "9-minimal", full_url: "registry.access.redhat.com/ubi9/ubi-minimal" },
];
const STANDARD_ROCKYLINUX: &[ImageItem] = &[
    ImageItem { display_name: "8", full_url: "quay.io/rockylinux/rockylinux:8" },
    ImageItem { display_name: "8-minimal", full_url: "quay.io/rockylinux/rockylinux:8-minimal" },
    ImageItem { display_name: "9", full_url: "quay.io/rockylinux/rockylinux:9" },
    ImageItem { display_name: "latest", full_url: "quay.io/rockylinux/rockylinux:latest" },
];
const STANDARD_SLACKWARE: &[ImageItem] = &[
    ImageItem { display_name: "current", full_url: "docker.io/vbatts/slackware:current" },
];
const STANDARD_STEAMOS: &[ImageItem] = &[
    ImageItem { display_name: "latest", full_url: "ghcr.io/linuxserver/steamos:latest" },
];
const STANDARD_UBUNTU: &[ImageItem] = &[
    ImageItem { display_name: "14.04", full_url: "docker.io/library/ubuntu:14.04" },
    ImageItem { display_name: "16.04", full_url: "docker.io/library/ubuntu:16.04" },
    ImageItem { display_name: "18.04", full_url: "docker.io/library/ubuntu:18.04" },
    ImageItem { display_name: "20.04", full_url: "docker.io/library/ubuntu:20.04" },
    ImageItem { display_name: "22.04", full_url: "docker.io/library/ubuntu:22.04" },
    ImageItem { display_name: "24.04", full_url: "docker.io/library/ubuntu:24.04" },
];
const STANDARD_VANILLAOS: &[ImageItem] = &[
    ImageItem { display_name: "vso", full_url: "ghcr.io/vanilla-os/vso:main" },
];
const STANDARD_VOIDLINUX: &[ImageItem] = &[
    ImageItem { display_name: "glibc", full_url: "ghcr.io/void-linux/void-glibc-full:latest" },
    ImageItem { display_name: "musl", full_url: "ghcr.io/void-linux/void-musl-full:latest" },
];

// --- ALL TOOLBOX GROUPS ---
const TOOLBOX_GROUPS: &[ImageGroup] = &[
    ImageGroup { name: "AlmaLinux", images: TOOLBOX_ALMALINUX },
    ImageGroup { name: "Alpine", images: TOOLBOX_ALPINE },
    ImageGroup { name: "AmazonLinux", images: TOOLBOX_AMAZONLINUX },
    ImageGroup { name: "Archlinux", images: TOOLBOX_ARCHLINUX },
    ImageGroup { name: "Bazzite Arch", images: TOOLBOX_BAZZITE },
    ImageGroup { name: "Centos", images: TOOLBOX_CENTOS },
    ImageGroup { name: "Debian", images: TOOLBOX_DEBIAN },
    ImageGroup { name: "Fedora", images: TOOLBOX_FEDORA },
    ImageGroup { name: "openSUSE", images: TOOLBOX_OPENSUSE },
    ImageGroup { name: "RedHat", images: TOOLBOX_REDHAT },
    ImageGroup { name: "Rocky Linux", images: TOOLBOX_ROCKYLINUX },
    ImageGroup { name: "Ublue", images: TOOLBOX_UBLUE },
    ImageGroup { name: "Ubuntu", images: TOOLBOX_UBUNTU },
    ImageGroup { name: "Wolfi", images: TOOLBOX_WOLFI },
    ImageGroup { name: "Custom", images: CUSTOM_IMAGE },
];

// --- ALL STANDARD GROUPS ---
const STANDARD_GROUPS: &[ImageGroup] = &[
    ImageGroup { name: "AlmaLinux", images: STANDARD_ALMALINUX },
    ImageGroup { name: "Alpine Linux", images: STANDARD_ALPINE },
    ImageGroup { name: "AmazonLinux", images: STANDARD_AMAZONLINUX },
    ImageGroup { name: "Archlinux", images: STANDARD_ARCHLINUX },
    ImageGroup { name: "Blackarch", images: STANDARD_BLACKARCH },
    ImageGroup { name: "CentOS Stream", images: STANDARD_CENTOS },
    ImageGroup { name: "Chainguard Wolfi", images: STANDARD_WOLFI },
    ImageGroup { name: "ClearLinux", images: STANDARD_CLEARLINUX },
    ImageGroup { name: "Crystal Linux", images: STANDARD_CRYSTAL },
    ImageGroup { name: "Debian", images: STANDARD_DEBIAN },
    ImageGroup { name: "deepin", images: STANDARD_DEEPIN },
    ImageGroup { name: "Fedora", images: STANDARD_FEDORA },
    ImageGroup { name: "Gentoo Linux", images: STANDARD_GENTOO },
    ImageGroup { name: "KDE neon", images: STANDARD_KDENEON },
    ImageGroup { name: "Kali Linux", images: STANDARD_KALI },
    ImageGroup { name: "Mint", images: STANDARD_MINT },
    ImageGroup { name: "Neurodebian", images: STANDARD_NEURODEBIAN },
    ImageGroup { name: "openSUSE", images: STANDARD_OPENSUSE },
    ImageGroup { name: "Oracle Linux", images: STANDARD_ORACLE },
    ImageGroup { name: "RedHat (UBI)", images: STANDARD_RHEL_UBI },
    ImageGroup { name: "Rocky Linux", images: STANDARD_ROCKYLINUX },
    ImageGroup { name: "Slackware", images: STANDARD_SLACKWARE },
    ImageGroup { name: "SteamOS", images: STANDARD_STEAMOS },
    ImageGroup { name: "Ubuntu", images: STANDARD_UBUNTU },
    ImageGroup { name: "Vanilla OS", images: STANDARD_VANILLAOS },
    ImageGroup { name: "Void Linux", images: STANDARD_VOIDLINUX },
    ImageGroup { name: "Custom", images: CUSTOM_IMAGE },
];

// --- THE NEW MAIN CONSTANT ---
pub const DISTROBOX_IMAGE_DATA: &[ImageTab] = &[
    ImageTab { title: "Toolbox Images", groups: TOOLBOX_GROUPS },
    ImageTab { title: "Standard Images", groups: STANDARD_GROUPS },
];