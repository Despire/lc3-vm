use lc3_vm::VM;
use std::env;

fn main() {
    VM::new(env::args().collect()).run();
}
