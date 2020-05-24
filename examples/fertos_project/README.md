The default target for this project is the TM4C1294, however it can also target
the TM4C123. <br />
In order to build this example for the TM4C1294, simply run `cargo build` <br />
To build this example for the TM4C123, you must run
`cargo build --features tm4c123 --no-default-features`

To load the binary onto a board, plug the board into your computer and in one
terminal run `openocd -f openoc.cfg`
Then in another terminal run:
* `cargo run` if you are targeting the TM4C1294
* `cargo run --features tm4c123 --no-default-features` if you are targeting the TM4C123
