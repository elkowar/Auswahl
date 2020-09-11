# Auswahl
Auswahl is a commandline utility that allows you to build simple menus using dmenu or rofi 
without having to write a shellscript for each.

## Usage

- To launch a specific menu, just call `auswahl <yourmenu>`.
- To get a list of all available menus, call `auswahl --list`.
- To use a custom config file path, call `auswahl -c <path-to-your-file> <yourmenu>`.

## Configuration
The `auswahlrc` configuration file has to be placed at `~/.config/auswahl/auswahlrc`.

Example config:
```
use: rofi -dmenu

# set redshift
menu: set-redshift :: -i
  off    => redshift -x
  medium => redshift -P -O 5000
  high   => redshift -P -O 4500

# play and pause music
menu: music
  pause => playerctl pause
  play => playerctl play

# search the web (type: "g <your query>" into the prompt to search google)
menu: search
  g  => firefox https://google.com?q={{}}
  yt => firefox https://youtube.com/results?search_query={{}}

```

When specifying a menu, you can add specific command line flags by adding 
`:: <your flags>` after the menu name. These will be added at the end of the menu command.

## Installation

Auswahl is available as a statically linked binary on the [releases page](https://github.com/elkowar/Auswahl/releases)
Just download it, make it executable (`chmod +x auswahl`), and run it!
-
Auswahl is also avaible as a [aur package](https://aur.archlinux.org/packages/auswahl-git/)  
Install it with your favourite aur helper. And it should be in your path.

## Dependencies

Auswahl depends only on what you configure as your menu! Use what you want!

## Building

To build Auswahl, you need to have the rust toolchain installed (see here: [Rustup](https://rustup.rs/))  
If you have that, clone this repository and run `cargo install --path .` in the project root.

If you only want to compile, run `cargo build --release`. This will generate a binary in `target/release/auswahl`
