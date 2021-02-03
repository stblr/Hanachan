# Hanachan

A reimplementation of the physics engine of Mario Kart Wii, aiming for perfectly accurate ghost replay. The initial goal is to reach 100% coverage of the CTGP database on Nintendo tracks.

## Status

At the moment, it is possible to replay the rMC3 Flame Runner BKT for 411 frames (that is, until the actual start of the race).

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

Ghidra project: ETA first week of February 2021

[mkw-structures](https://github.com/SeekyCt/mkw-structures)

[riidefi's decompilation](https://github.com/riidefi/mkw)

[Symbol Map](https://docs.google.com/spreadsheets/d/1gA5WmnEbPAeA1Lq4XUJg9qDwawky9hpNUv2n1wWRwno/)

[Tockdom Wiki](http://wiki.tockdom.com/wiki/Main_Page)

## License

Copyright 2003-2021 Dolphin Emulator Project

Copyright 2020-2021 Pablo Stebler

This software is available under the terms of the GNU General Public License, either version 2 of the License, or (at your option) any later version.
