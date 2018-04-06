# chip8-rs

CHIP-8 interpreter. Needs SDL2 (`brew install sdl2` / `apt install libsdl2-dev`).

    cargo run --release -- roms/pong.ch8

Keys (original keypad on the left, keyboard on the right):

    1 2 3 C      1 2 3 4
    4 5 6 D      Q W E R
    7 8 9 E      A S D F
    A 0 B F      Z X C V

Escape quits. Display wait quirks are not implemented, so some ROMs flicker.
