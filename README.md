# Hanachan

A reimplementation of the physics engine of Mario Kart Wii, aiming for perfectly accurate ghost replay. The initial goal is to reach 100% coverage of the CTGP database on Nintendo tracks.

## Status

See [STATUS.md](STATUS.md).

## May 2021 roadmap

- [x] More vehicles
  - [x] Karts
  - [x] Outside-drifting bikes
  - [x] Quacker
  - [x] Blue Falcon
- [ ] RKRD improvements
  - [x] Rewrite as a GCT
  - [ ] Support regions other than PAL
  - [ ] Automation options
- [ ] KMP features
  - [ ] KTPT
  - [ ] CKPT/CKPH
  - [ ] POTI
  - [ ] AREA
  - [ ] JGPT
  - [ ] CNPT
- [ ] KCL features
  - [x] Floor collision
  - [ ] Wall collision
  - [x] Off-road properties
  - [x] Boost panels
  - [ ] Ramps
  - [ ] OOB
  - [ ] Cannons
  - [ ] Moving road
- [ ] Driving mechanics
  - [ ] SSMTs
  - [ ] Tricks
  - [ ] Respawn boost
  - [ ] Nosediving/taildiving
  - [x] Slipdrifts
- [ ] CLI overhaul
  - [ ] Make RKRD optional
  - [ ] Multiple runs
  - [ ] Configurable output
- [ ] 3D renderer
  - [ ] Collision (KCL/BSP) view
  - [ ] BRRES view
    - [ ] Basic MDL0
    - [ ] Advanced MDL0
    - [ ] TEX0
    - [ ] CHR0

## Future plans

* 200cc
* Custom tracks
* Multiplayer mechanics
* Advanced ghost analysis and statistics
* Graphics reimplementation using Vulkan
* Full game reimplementation (maybe)

## Building

Install the latest version of Rust, then run:

```bash
cargo build --release
```

## Running

Make a copy of your Mario Kart Wii disc with e.g. [CleanRip](https://wiibrew.org/wiki/CleanRip).

Extract `Common.szs` and the `Course` directory with e.g. [Dolphin](https://github.com/dolphin-emu/dolphin) or [WIT](https://wit.wiimm.de/).

Then run:

```bash
./target/release/hanachan Common.szs Course samples
```

Every ghost in the `samples` directory will be replayed and compared to a dump of the most important physics variables. The number of accurate frames will then be printed.

It is also possible to supply a single ghost file for more detailed output, or a single track file.

## Contributing

At the moment, the codebase is evolving very fast and there is still some code I haven't integrated yet, so if you want to contribute non-trivial features, please tell me about it so we can properly coordinate.

## Resources

Note: the vast majority of research and development happens on the PAL version of the game.

[Ghidra project](https://drive.google.com/drive/folders/1I1VRfeut3NtPeddePutfAaZhduVdKhhc?usp=sharing)

[mkw-structures](https://github.com/SeekyCt/mkw-structures)

[riidefi's decompilation](https://github.com/riidefi/mkw)

[Symbol Map](https://docs.google.com/spreadsheets/d/1gA5WmnEbPAeA1Lq4XUJg9qDwawky9hpNUv2n1wWRwno/)

[Tockdom Wiki](http://wiki.tockdom.com/wiki/Main_Page)

## License

Copyright 2003-2021 Dolphin Emulator Project

Copyright 2020-2021 Pablo Stebler

This software is available under the terms of the GNU General Public License, either version 2 of the License, or (at your option) any later version.
