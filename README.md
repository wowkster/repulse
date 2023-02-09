# Repulse

A next generation (blazingly fast) ransomware for Windows written in Rust. Encrypt files and steal user data at speeds never before thought possible, while ensuring memory safety.

## Akagi

This program uses Akagi ([UACME](https://github.com/hfiref0x/UACME)) to elevate itself to local administrator and bypass the windows UAC prompt. This repository does not contain a compiled Akagi64 binary, but one is required for compilation. Building a binary is trivial by following the directions in the [UACME](https://github.com/hfiref0x/UACME) repository. After building, place the binary in the root directory of this project and rename it `Akagi64.bin`