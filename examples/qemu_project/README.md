To build this project, simply run `cargo build`. <br />
To run the project without debugginf, run `cargo run`. <br />
To connect to the project with gdb, in one terminal run:
```
qemu-system-arm -M lm3s6965evb \
    -cpu cortex-m3 \
    -serial stdio \
    -gdb tcp::3333 \
    -S \
    -kernel ../../target/thumbv7m-none-eabi/debug/qemu_project
```
Then in another terminal run:
```
gdb-multiarch -x qemu.gdb -q -tui ../../target/thumbv7m-none-eabi/debug/qemu_project
```
<br/>
Or more simply, in one terminal run:
```
./start_debug.sh
```
To build the project and start qemu with the arguments above.<br/>
Then in another terminal run:
```
./start_gdb.sh
```
To start gdb with the arguments above
