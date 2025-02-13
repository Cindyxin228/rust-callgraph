

pub trait T {
    fn bla(&self) -> bool { let _g = 3; return true ; }
}

pub struct S;

impl S {
    // pub fn met(&self) {
    //     if S1::should_call_bla(1) && S1::should_call_bla1(2) && S1::should_call_bla2(3) {
    //         self.bla(); // 调用 bla
    //     }
    //     println!("met called");
    // }

    // pub fn met(&self, a: i32, b: i32, c: i32) {
    //     if a == 1 && b == 2 && c == 3 {
    //         self.bla(); // 调用 bla
    //     }
    //     println!("met called");
    // }

    pub fn bla(&self) -> bool {
        for _ in 0..1 {
            println!("bla called");
        }
        return true;
    }

    // pub fn test_while(&self, a: i32) -> i32 {
    //     self.bla();
    //     return a + 1;
    // }

    // pub fn test_while1(&self, a: i32) -> i32 {
    //     self.bla();
    //     return a + 2;
    // }
}

// 实现 IntoIterator trait，使 S 成为一个迭代器
// impl IntoIterator for S {
//     type Item = char;
//     type IntoIter = std::vec::IntoIter<char>; // 返回一个迭代器

//     fn into_iter(self) -> Self::IntoIter {
//         // 假设 S 能生成一个字符的 Vec
//         vec!['a', 'b', 'c'].into_iter() // 示例：S 迭代 'a', 'b', 'c'
//     }
// }


pub struct S1;

impl S1 {
    pub fn should_call_bla(a: i32) -> bool {
        a == 1
    }

    pub fn should_call_bla1(a: i32) -> bool {
        a == 2
    }

    pub fn should_call_bla2(a: i32) -> bool {
        a == 3
    }
}

// impl T for S {
//     fn bla(&self) -> bool {
//         let _i = 6;
//         return true;
//     }
// }

// pub struct R;

// impl T for R {
//     fn bla(&self) -> bool {
//         let _x = 4;
//         return true;
//     }
// }

// pub fn _virt(ob: &dyn T) {
//     ob.bla();
// }

// // 假设你想让 test_between_module 也能支持迭代器的类型推导
// pub fn test_between_module<I, W>(_input: I, _output: W) -> bool
// where
//     I: IntoIterator<Item = char>, // 约束 I 必须是一个迭代器
//     W: T,                     
// {
//     // 示例，假设我们在这里有一些逻辑
//     println!("test_between_module called");
//     true
// }

// pub fn apply_ascii_deny_list_to_potentially_upper_case_ascii(c: char, deny_list: &[char]) -> char {
//     // 假设这个函数对字符做一些处理
//     if deny_list.contains(&c) {
//         return '*'; // 如果字符在 deny_list 中，返回 '*'
//     }
//     c // 否则返回原字符
// }



