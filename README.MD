# Space Invaders Arcade emulator.

Uses [r8080](https://github.com/solomonarul/r8080) as the base CPU.

The main thread handles drawing to the screen and the rest is offset to a separate thread for the CPU.

## Controls:

- W - Shoot.
- A, D - Move left / right.
- Space - Insert coin.
- 1 - First player button.

## Status:

- TODO: figure out the insane 20% CPU usage.

<p align="center">
    <img src=".github/latest.gif"/>
</p>

---

<a href="https://brainmade.org/">
    Made with <3 by a human.
    <img src="https://brainmade.org/88x31-light.png" align="right">
</a>