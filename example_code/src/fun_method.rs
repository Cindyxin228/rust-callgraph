pub trait T {
    fn bla(&self){let _g=3;}
}

pub struct S;

impl S {
    pub fn met(&self) {
        if self.should_call_bla() {
            self.bla(); // 调用 bla
        }
        println!("met called");
    }

    pub fn bla(&self) {
        for _ in 0..1 {
            println!("bla called");
        }
    }

    fn should_call_bla(&self) -> bool {
        // 这里可以返回 true 或 false
        true
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
