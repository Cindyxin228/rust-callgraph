mod a; 

use crate::a::fun_method::{S, S1, R, T}; 

fn main() {
    let s = S {};
    let r = R {};

    // // 测试 if 表达式
    // s.met();

    // // for loop
    // for i in 0..3 {
    //     println!("Loop iteration: {}", i);
    //     if S1::should_call_bla(i) {
    //         s.bla();
    //     }
    // }

    // // 测试嵌套 if 和函数调用
    // if S1::should_call_bla(1) && S1::should_call_bla1(2) {
    //     s.bla();
    // }

    // // 测试复杂条件
    // let mut count = 0;
    // while count < 5 {
    //     if S1::should_call_bla2(count) {
    //         println!("Count matches condition: {}", count);
    //     }
    //     count += 1;
    // }
    let mut a = 0;
    //just loop
    // loop {
    //     a += 1;
    //     if s.test_while(a) >= 10 {
    //         break;
    //     }
    // }

    //while loop
    // while (s.test_while(a) < 10 || s.test_while1(a) < 10) {
    //     a += 1;
    // }

    let t: &dyn T = &s;
    let r: &dyn T = &r;

    //dynamic call
    if  t.bla() && r.bla() {
        println!("both true");
    } else {
        println!("both false");
    }
}
