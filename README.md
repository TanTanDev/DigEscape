# Dig Escape
Dig Escape is a simple puzzle game written in [Rust](https://www.rust-lang.org/)

### Background
What started out as a small project with the goal of learning [Rust](https://www.rust-lang.org/)
, ended up being released playable on the [web!](https://tantandev.itch.io/digescape)

The progress was recorded on my [Youtube Channel](https://www.youtube.com/channel/UChl_NKOs1qqh_x7yJfaDpDw)

### Building
before you can run using cargo,
The game assets need to be zipped as a .tar and put into the src/ folder

To automatically zip the /resources there is a script in utils/wasm/zip_resources.sh you can run using git bash:
```
# first cd into the utils/wasm/ folder
./zip_resources.sh
```
Then we can use cargo to run the project on windows
```bash
cargo run
```
## WebAssembly
There is a script in utils/wasm/build.sh you can run using git bash.
```bash
./build.sh
```
This script compiles the program with cargo, takes the generated dig_escape.wasm file, and the files in utils/wasm/ and
moves them into a new folder called static/.
To run it in the browser I'm, using [basic-http-server](https://crates.io/crates/basic-http-server).
```bash
cargo install basic-http-server
```
start the server by using the correct path
```bash
basic-http-server . # starts server based on current directory
basic-http-server static # start server in the folder /static
```


### Dependencies
Forked Game framework [Good-web-game](https://github.com/TanTanDev/good-web-game)
Note: I'm using the audio branch which is a work in progress
