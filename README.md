# 2.5d Maze Renderer

Generates a 2d maze and renders it in 3d, so you can explore in first person. 
Has basic lighting using the Phong reflection model, and also you can place portals. 
All the logic for faking the 3d effect is done from scratch without any dependencies. 

This was my first project learning Rust and also my first experimentation with low level computer graphics. 
Expect the code to be terrible and enjoy perusing many commits that struggle to appease the borrow checker. 

Originally used SDL for window/input and used only their line drawing function (see [branch:desktop-sdl](https://github.com/LukeGrahamLandry/2.5d-maze-renderer/tree/desktop-sdl)). 
Targeting Wasm with SDL was fragile, so I updated it to use winit & softbuffer and directly set individual pixel colour values. 

[Try it online!](https://lukegrahamlandry.ca/maze/)

https://user-images.githubusercontent.com/40009893/229319149-fa7562c5-7852-4e8d-850a-fde13d2dbafd.mov

> Controls: WASD to move, right/left click to place portal, space to toggle between 2d and 3d rendering

## Build 

`cargo run --release` will build and run the native binary for your operating system (runs much faster than the Wasm version).   

To build for Wasm, use: 

```
cargo build --target=wasm32-unknown-unknown --release && wasm-bindgen --out-dir=web/pkg --target=web ./target/wasm32-unknown-unknown/release/mazerender2d.wasm
```

Then serve the contents of the `web` directory. 

## Things To Improve

- Wasm: correct canvas size and be independent of physical vs logical resolution. 
- Break the mazes into multiple regions for more efficient rendering. 
- Figure out how to update lighting without updating it everywhere at once. 
- Exits in the mazes and generate new mazes to fill an infinite world.
  - Make sure light caching doesn't use too much memory.
- Be able to see yourself through portals again. 
- Vecs instead of hashmaps
- Finish the mazes book and let you switch between generation algorithms
