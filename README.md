# chip8.rs
Chip 8 emulator implemented in Rust
graphics done using SDL2 https://github.com/AngryLawyer/rust-sdl2

Huge credit goes out to Thomas P. Greene a.k.a. cowgod, who has an excellent document that contains pretty much everything you could
possibly need to know about the CHIP8 here: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

Also to mikezaby who previously did the same thing, and who's code I learned a lot from. Probably would have never figured out SDl
if it weren't for reading through your stuff. See his work here: https://github.com/mikezaby/chip-8.rs

Things left TODO:
1. break up into modules so it's no longer one monstrous main.rs
2. optimize it some more
3. if it's still slow, implement dynamic instruction compilation.
