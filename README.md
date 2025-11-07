# hsh — A Minimal Shell in Rust

**hsh** is a lightweight command-line shell implemented in Rust.  
It is designed to be a minimal, extensible, and educational shell, similar in spirit to traditional Unix shells like `sh` and `zsh`.  
The project is currently under active development.

---

## Overview

The shell provides a simple read–evaluate–print loop (REPL) that reads commands from standard input, tokenizes the input into commands and arguments, and executes basic built-in operations.  

At this stage, `hsh` supports simple text commands and basic error handling. It serves as a foundation for future expansion into a more feature-complete shell environment.

---

## Features

- Interactive shell prompt (`~$ `)
- Input tokenization into commands and arguments
- Command execution through pattern matching
- Built-in commands:
  - `echo`: Prints the provided arguments to standard output
  - `exit`: Terminates the shell session gracefully
- Basic error reporting for unsupported commands

---
