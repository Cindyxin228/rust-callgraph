mod a; 
use crate::a::fun_method::{S1, S, R, T}; 

fn main() {
    let s = S {};
    let r = R {};

    // // 测试 if 表达式
    // s.met();

    // for loop
    // for i in 0..100{
    //     s.bla();
    // }
    let i = 0;
    // while(i < 100){
    //     s.bla();
    // }
    
    // loop{
    //     if i < 100 {
    //         s.bla();
    //     }
        
    // }
    // s.met();
    // println!("lla");
    // let a = true && false;
    // // 测试嵌套 if 和函数调用
    if S1::should_call_bla(1) && S1::should_call_bla1(2) && S1::should_call_bla2(3){
        s.bla();
    }

    let a = String::new();


    // // 测试复杂条件
    // let mut count = 0;
    // while count < 5 {
    //     if S1::should_call_bla2(count) {
    //         println!("Count matches condition: {}", count);
    //     }
    //     count += 1;
    // }
    // let mut a = 0;
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


    // let t: &dyn T = &s;
    // let r: &dyn T = &r;

    // //dynamic call
    // if  t.bla() && r.bla() {
    //     println!("both true");
    // } else {
    //     println!("both false");
    // }
}


// struct S;

// impl S {
//     fn test_a(&self, x: i32) -> bool {
//         return self.a(x);
//     }

//     #[inline(always)]
//     fn a(&self,x: i32) -> bool{
//         return x == 1;
//     }
// }



// fn main() {
//     // let ascii = vec!['a', 'b', 'c', 'd', 'e'];
//     // let deny_list = vec!['a', 'c']; // 假设这是我们的 deny_list

//     // // 使用 map 和闭包，模拟你的例子
//     // let result: Vec<_> = ascii.iter()
//     //     .map(|&c| {
//     //         a::fun_method::apply_ascii_deny_list_to_potentially_upper_case_ascii(c, &deny_list)
//     //     })
//     //     .collect();
//     let s = S {};
//     let a = s.test_a(1);
//     // // 输出处理后的结果
//     // println!("{:?}", result); // 应该打印 ['*', 'b', '*', 'd', 'e']
// }

// 引入 rand crate
// use rand::Rng;

// fn main() {
//     // 创建一个随机数生成器
//     let mut rng = rand::thread_rng();
    
//     // 生成一个 1 到 100 的随机数
//     let random_number: u32 = rng.gen_range(1..=100);
    
//     // 打印随机数
//     println!("Generated random number: {}", random_number);
// }

// fn for_a()-> bool {
//     return true;
// }

// fn a(b: bool)->i32{
//     if b{
//         return 1;
//     }
//     else{
//         return 0;
//     }
// }

// fn main() {
//     a(for_a());
// }


//测试函数参数是函数
// fn write_char(c: char){
//     println!("{}", c);
// }

// fn main(){
//     let c = 'A';
//     write_char(char::from(c.to_ascii_lowercase()));
// }


// use std::io::{self, Write};

// fn main() -> io::Result<()> {
//     // 创建一个 `Vec<u8>` 作为 sink
//     let mut sink: Vec<u8> = Vec::new();

//     let c = 'A';
//     // 将字符 'A' 转换为字节并写入 sink
//     sink.write(&[char::from(c.to_ascii_lowercase()) as u8])?;

//     // 输出结果，查看 sink 中的字节内容
//     println!("Written data: {:?}", sink);
    
//     Ok(())
// }


// struct S;

// impl S {
//     fn a(&self, b: bool) -> i32 {
//         return 1;
//     }
    
//     fn test(&self, c: bool) -> i32 {
//         if self.a(c) == 1{
//             return 1   
//         }
//         else{
//             return 0;
//         }
//     }
// }

// fn a()->bool{
//     false
// }

// fn main() {
//     if test_assert(){
//         a();
//     }
//     // debug_assert!(test_assert());
// }

// fn test_assert() -> bool{
//     return true;
// }

// fn b()->bool{
//     true
// }

// fn c()->bool{
//     true
// }

// fn main(){
//     for i in 0..1{
//         let a = if true{
//             a();
//         }else{
//             b();
//         };
//     }
    
// }


// trait Foo {
//     fn do_foo(&self);
// }

// struct A;
// struct B;

// impl Foo for A {
//     fn do_foo(&self) {
//         println!("A does foo");
//     }
// }

// impl Foo for B {
//     fn do_foo(&self) {
//         println!("B does foo");
//     }
// }

// fn call_foo<T: Foo>(x: T) {
//     x.do_foo();
// }

// fn main() {
//     let a1 = A;
//     let a = A;
//     let b = B;
    
//     call_foo(a); // 静态分发
//     call_foo(b); // 静态分发

//     let dynamic_foo: &dyn Foo = &a1;
//     dynamic_foo.do_foo(); // 动态分发
// }

