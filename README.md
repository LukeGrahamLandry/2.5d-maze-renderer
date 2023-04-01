# 2.5d Maze Renderer

Generates a 2d maze and renders it in 3d so you can explore in first person. 
Has basic lighting using the Phong reflection model and also you can place portals. 
Uses SDL to create a window and get user input but the only drawing function used is making a line between two points on a 2d canvas. 
All the logic for faking the 3d effect is done from scratch without any dependencies. Mostly targeting native but also kinda works in the browser with Web Assembly. 

This is my first project learning Rust. Expect the code to be terrible and enjoy perusing many commits that struggle to appease the borrow checker. 

https://user-images.githubusercontent.com/40009893/229319149-fa7562c5-7852-4e8d-850a-fde13d2dbafd.mov

> Controls: WASD to move, right/left click to place portal, space to toggle between 2d and 3d rendering
