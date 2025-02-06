pub trait T {
    fn bla(&self) -> bool { let _g = 3; return true ; }
}

pub struct S;

impl S {
    pub fn met(&self) {
        if S1::should_call_bla(1) && S1::should_call_bla1(2) && S1::should_call_bla2(3) {
            self.bla(); // 调用 bla
        }
        println!("met called");
    }

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

    pub fn test_while(&self, a: i32) -> i32 {
        self.bla();
        return a + 1;
    }

    pub fn test_while1(&self, a: i32) -> i32 {
        self.bla();
        return a + 2;
    }
}

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

impl T for S {
    fn bla(&self) -> bool {
        let _i = 6;
        return true;
    }
}

pub struct R;

impl T for R {
    fn bla(&self) -> bool {
        let _x = 4;
        return true;
    }
}

pub fn _virt(ob: &dyn T) {
    ob.bla();
}
