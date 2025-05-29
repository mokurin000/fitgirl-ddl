# fitgirl-ddl

Extract direct download links from fitgirl-repacks.site (must have fuckingfast.co source),
export to aria2 input file.

## Installation

### Windows/MacOS

CLI/GUI version for Windows/MacOS and CLI version for Linux are avaliable in [releases](https://github.com/mokurin000/fitgirl-ddl/releases/latest)

### ArchLinux

CLI version:

```
paru -S fitgirl-ddl
```

GUI version, GTK4+:

```bash
paru -S fitgirl-ddl-gtk4
```

## Screenshots

### Scraping

![image](https://github.com/user-attachments/assets/aff1d175-b8b1-41b4-a2a6-aaf1bb86f2be)

### Export Completed

![image](https://github.com/user-attachments/assets/80856bb8-20f2-48fd-b5d5-b4e3ce045de9)

### Download games

![image](https://github.com/user-attachments/assets/970c6ca7-61b7-4911-aa30-807084796225)

## Build Instructions

Note: To build cli version, replace `fitgirl-ddl_gui` with `fitgirl-ddl`

### Windows/MacOS

Windows/MacOS provide native GUI toolkit, no additional dependency was needed.

```
cargo build --release --bin fitgirl-ddl_gui
```

### Linux

to build GUI version on linux, you should setup latest Rust toolchain and install `gtk4-dev` e.g. (and `qt6-base-dev` for Qt backend) Qt backend is usable, but currently with bad performance at window resizing.

To build fitgirl-ddl-gui with GTK+4:

```bash
cargo build --release --bin fitgirl-ddl_gui
```

To build fitgirl-ddl-gui with Qt6:

```bash
RUSTFLAGS="-C link-args=-flto" cargo build --release --bin fitgirl-ddl_gui --no-default-features -F qt
```
