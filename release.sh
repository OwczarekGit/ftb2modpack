#!/bin/bash

cargo build --release
cross build --release --target x86_64-pc-windows-gnu

mv ./target/release/ftb2modpack ./ftb2modpack_linux
mv ./target/x86_64-pc-windows-gnu/release/ftb2modpack.exe ./ftb2modpack.exe
