mod fun_method;

use fun_method::S;

fn main() {
    let s = S {};
    s.met();
    if true {
        s.bla();
    }
}
