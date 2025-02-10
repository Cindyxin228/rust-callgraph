// // 引入 fun_method 模块中的内容
// use crate::a::fun_method::{T, S, S1, R, _virt};
// // 定义一个新的结构体
// pub struct MyStruct;

// impl MyStruct {
//     // 自定义方法，调用 S 和 S1 的方法
//     pub fn my_method(&self) {
//         let s = S;
//         let r = R;

//         // 调用 S 的 met 方法
//         s.met();

//         // 调用 S1 的静态方法
//         let should_call = S1::should_call_bla(1);
//         println!("MyStruct: should_call_bla result: {}", should_call);

//         // 调用 S 的 bla 方法
//         s.bla();
//     }

//     // 另一个方法，使用动态分发调用 _virt
//     pub fn use_virt(&self) {
//         let s = S;
//         let r = R;

//         // 调用 _virt 函数
//         _virt(&s);
//         _virt(&r);
//     }

//     pub fn call_test_between_module() {
//         let s = S;
//         let t = R;
        
      
//         crate::a::fun_method::test_between_module::<_, _>(s, t);
    
//     }
    
// }

// // 实现 T trait 给 MyStruct
// impl T for MyStruct {
//     fn bla(&self) -> bool {
//         println!("MyStruct: bla called");
//         true
//     }
// }


