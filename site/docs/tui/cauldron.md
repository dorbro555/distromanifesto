---
sidebar_position: 1
title: The Dashboard (Cauldron)
---

# The Dashboard (Cauldron)

The **Cauldron** is the central interactive dashboard for Distromanifesto. It provides a real-time view of your container ecosystem, allowing you to manage the lifecycle of containers, manifests, and home directories without memorizing CLI commands.

To launch the dashboard:

```bash
dimo cauldron
```

# Interface Overview

The interface is divided into three main sections:

1. Left Sidebar: Stacked lists for Containers, Manifests, and Homes.
2. Right Pane: A detail view with two tabs: [ Info ] and [ Actions ].
3. Footer: A quick reference for navigation keys.

# Navigation & Controls

You can navigate the TUI using standard Vim motions or Arrow keys.

## Global Navigation
| Action	| Keybinding |
|---------- |----------- |
|Cycle Focus	| `Tab` (Cycles: Containers → Manifests → Homes → Content) |
|Move Selection |	`j` / `k` (or `↓` / `↑`) |
|Switch Detail Tab |	`h` / `l` (or `←` / `→`) (Toggles between Info and Actions) |
|Refresh Data |	`r` |
|Toggle Help |	`?` |
|Quit |	`q` |

# Pane Actions

Different actions are available depending on which list is currently focused.

1. Containers

Manage your running Distrobox instances.

| Key | Action | Description |
|-----|--------|-------------|
| e	| Enter |	Opens a shell inside the selected container.|
|s |	Stop |	Stops the container (distrobox stop).|
|u |	Upgrade |	Runs the package manager upgrade inside the container.|
|d |	Delete |	Prompts to forcibly remove the container.|

2. Manifests

Manage your .ini configuration files stored in ~/.distromanifesto/manifests/.

|Key |	Action |	Description |
|----|---------|----------------|
|c |	Create |	Assembles (builds) the container defined in the manifest.|
|m |	Modify |	Opens the manifest in the TUI Editor.|
|n |	New	Launches the "Wizard" to create a brand new manifest.|
|v |	Verify	Checks the syntax of the selected file.|
|d |	Delete	Deletes the manifest file.|

3. Homes

Manage the isolated home directories stored in ~/.distromanifesto/homes/.

|Key |	Action |	Description |
|----|---------|----------------|
|i |	Inspect |	Calculates disk usage (runs du -sh) for the directory. |
|d |	Delete |	Prompts to permanently delete the home directory. |

# Status Indicators

The dashboard uses color coding to indicate container health:

- <span style={{color: 'green'}}>Green (●)</span>: Container is Up (Running).
- <span style={{color: 'grey'}}>Grey (○)</span>: Container is Stopped or Exited.
- <span style={{color: 'yellow'}}>Yellow (○)</span>: Container is Created but has unknown status.