---
sidebar_position: 2
---

# Quickstart

Let's create your first Distrobox container using the Interactive Wizard.

## 1. The Wizard

Run the `create` command (or just `dimo` by itself) to launch the TUI Wizard:

```bash
dimo create
```

You will see an interactive form. Use your keyboard to navigate:

- Arrow Keys: Move between fields.
- Enter: Confirm a selection.

Fill in the basics:

- Name: my-first-box
- Image: archlinux:latest (or your preferred distro)

When you save, dimo will generate a manifest file for you.

## 2. Assemble the Container

Once the manifest is created, you can build the container immediately:

```bash
dimo assemble my-first-box.ini
```

This will trigger distrobox create under the hood and pull the necessary images.

## 3. Storage Locations

dimo keeps your environment organized in a hidden directory in your home folder (~/.distromanifesto).

- Manifests: Stored in ~/.distromanifesto/manifests/. This is where your .ini configurations live.
- Homes: Stored in ~/.distromanifesto/homes/. If you define a custom home for your container, dimo manages it here to keep your real $HOME clean.

## Next Steps
Now that you have a running container, check out the Dashboard to manage it:

```bash
dimo cauldron
```