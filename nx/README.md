# balatro-vs on Nintendo Switch (`nx/`)

The balatro-vs core as a [skyline](https://github.com/skyline-dev/skyline) plugin. It runs
next to [lovely-injector-nx](https://github.com/Fcornaire/lovely-injector-nx), which hooks
the game, runs lovely and hands this plugin the game's Lua state
through its C ABI. This crate is the Switch equivalent of `winmm.dll` (Windows) /
`libwinmm.so` (Android).

Works on hardware (tested latest Balatro 1.1.3, Atmosphere 1.11.2 and switch firmware 22.5.0)
Switch x PC and Switch x Android matches played.

## Requirements

Build machine :

- rustup toolchain `skyline` and `cargo-skyline` (`cargo install cargo-skyline`).

Console: a working lovely-injector-nx install and a copy of
Steamodded `26.829.0` .

## Build

```sh
cd nx
cargo skyline build --release
```

## Install

Grab from the release the zip and extract it onto your SD card, the zip should already have the correct layout :

```
atmosphere\contents\0100CD801CE5E000\romfs\skyline\plugins\
  liblovely_injector_nx.nro                the loader
  libbalatro_vs_nx.nro                     this plugin
Balatro\Mods\
  Steamodded\                              copy of Steamodded 26.829.0
  balatro-vs\lovely\                       copy of ..\patchs (tomls, Lua modules, bvs.json)
```

## Network

The Switch uses the **relay** transport,a WebSocket to the matchbox server, joined with
`?relay=1`. The server pairs it with a PC or Android peer and forwards packets

## Known limitations

- the game's LuaJIT look running interpreted, so it run poorly with others mods , read [lovely-injector-nx performance](https://github.com/Fcornaire/lovely-injector-nx#Performance)
