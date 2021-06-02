# Hanachan

A reimplementation of the physics engine of Mario Kart Wii, aiming for perfectly accurate ghost replay. The initial goal is to reach 100% coverage of the CTGP database on Nintendo tracks.

## Status

At the moment, it is possible to replay the rMC3 WR for 5080 frames (until the end of the race but without the final slowdown after the finish line).

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

## Running the program

Two files have to be supplied as cli arguments: a ghost file in the rkg format, and a race dump file in the custom rkrd format, which contains important physics variables for each frame. The program replays the ghost and compares it to the dump, and as soon as there is a difference for any of the variables, it is printed and the execution is stopped. Sample files are provided in the data directory.

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
