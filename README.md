# todo-ipc

Todo list over IPC

[![build](https://github.com/mosmeh/todo-ipc/workflows/build/badge.svg)](https://github.com/mosmeh/todo-ipc/actions)

## Usage

### Server

```sh
cargo run --bin server
```

### Client

```sh
cargo run --bin client -- add Do something
cargo run --bin client -- list
cargo run --bin client -- check 1
cargo run --bin client -- rm 1
```
