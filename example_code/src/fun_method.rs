pub trait T {
    fn bla(&self) { let _g = 3; }
}

pub struct S;

impl S {
    // pub fn met(&self) {
    //     if S::should_call_bla(1) && S::should_call_bla1(2) && S::should_call_bla2(3) {
    //         self.bla(); // 调用 bla
    //     }
    //     println!("met called");
    // }

    pub fn met(&self, a: i32, b: i32, c: i32) {
        if a == 1 && b == 2 && c == 3 {
            self.bla(); // 调用 bla
        }
        println!("met called");
    }

    pub fn bla(&self) {
        for _ in 0..1 {
            println!("bla called");
        }
    }

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

impl T for S {
    fn bla(&self) {
        let _i = 6;
    }
}

pub struct R;

impl T for R {
    fn bla(&self) {
        let _x = 4;
    }
}

pub fn _virt(ob: &dyn T) {
    ob.bla();
}
