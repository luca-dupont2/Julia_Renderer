# Julia Set Renderer
> Plot and explore Julia Sets!

## Installation

```bash
git clone https://github.com/luca-dupont2/Julia_Renderer.git
cd Julia_Renderer
```

## Usage :
* First install cargo (mentioned in [requirements](#Requirements) below).
* Within the directory, either run :
  ```console
  cargo run --release
  ```
* Or create an application bundle which you'll find in `target/release/bundle/` by running :
  ```console
  cargo bundle --release
  ```
* Change functions to graph in the code at line 30

## Controls :
> Move the mouse around to change the value of c
---
* \- : Zoom out
* \+ : Zoom in
* ← : Scroll left
* → : Scroll right
* ↑ : Scroll up
* ↓ : Scroll down
* Space : Freeze the screen

## Requirements
The following installs are required for this app to work.
- Cargo

To install this :
* Linux or MacOS systems : run
  ```console
  curl https://sh.rustup.rs -sSf | sh
  ```
* Windows : Download and run [rustup-init.exe](https://win.rustup.rs/) 

## Sample images
<div style="display: flex; gap: 10px;">
  <img src="/images/Plot1.png" width="400">
  <img src="/images/Plot2.png" width="400">
  <img src="/images/Plot3.png" width="400">
  <img src="/images/Plot4.png" width="400">
</div>

