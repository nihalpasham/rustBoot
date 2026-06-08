# Formal Models for rustBoot

## Overview
Two formal models of the rustBoot boot state machine:

1. **TLA+** (`tla_plus/boot_state_machine.tla`) — Industrial-grade formal
   specification language by Leslie Lamport. Supports model checking with TLC.
   
2. **Alloy** (`alloy/boot_state_machine.als`) — Lightweight formal modeling
   language by Daniel Jackson. Supports automated analysis with the Alloy Analyzer.

## What's Modeled
- Boot state machine (5 states, 4 valid transitions)
- Partition state invariants
- Invalid transition rejection

## Running

### TLA+
1. Install TLA+ Toolbox: https://tla.msr-inria.inria.fr/tlatoolbox/
2. Open `boot_state_machine.tla` in TLC model checker
3. Model check with default parameters

### Alloy
1. Install Alloy Analyzer: https://alloytools.org/
2. Open `boot_state_machine.als`
3. Execute the `check` and `run` commands

## Verification Results

### TLA+
[To be filled after model checking]

### Alloy
[To be filled after analysis]

## Relationship to Code
The formal models correspond to:
- `rustBoot/src/image/image.rs` — `ImageType` enum and transition methods
- `rustBoot/src/image/image.rs` — `SectFlags` encoding