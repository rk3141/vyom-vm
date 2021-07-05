use std::{ops::Add, rc::Rc, usize};

#[repr(u8)]
enum Instructions {
    STOP = 0,
    PUSH,
    POP,
    ADD,
    SUB,
    MUL,
    DIV,
    CALL,
}

macro_rules! add {
    () => {
        Instructions::ADD as u8
    };
}

macro_rules! sub {
    () => {
        Instructions::SUB as u8
    };
}

macro_rules! mul {
    () => {
        Instructions::MUL as u8
    };
}

macro_rules! div {
    () => {
        Instructions::DIV as u8
    };
}

macro_rules! inst {
    ($val:ident) => {
        Instructions::$val as u8
    };
}

struct VM {
    code: Vec<u8>,
    data: Vec<u32>,
    imports: Vec<Rc<dyn Fn() + 'static>>,
    counter: usize,
}

impl VM {
    pub fn new(code: Vec<u8>, imports: Vec<Rc<dyn Fn()>>) -> Self {
        Self {
            code,
            data: vec![],
            imports,
            counter: 0,
        }
    }

    fn peek(&mut self) -> u8 {
        self.code[self.counter + 1]
    }

    fn step(&mut self) -> u8 {
        self.counter += 1;

        self.code[self.counter - 1]
    }

    fn step_moved(&mut self) -> u8 {
        self.counter += 1;

        self.code[self.counter]
    }

    fn step_counter(&mut self) {
        self.counter += 1;
    }

    fn current(&self) -> u8 {
        self.code[self.counter]
    }

    pub fn execute(&mut self) {
        while self.counter < self.code.len() {
            if self.current() == Instructions::PUSH as u8 {
                let data = self.step_moved();

                self.data.push(data.into());
            } else if self.current() == Instructions::ADD as u8 {
                let b = self.data_pop();
                let a = self.data_pop();

                self.data.push(a + b);
            } else if self.current() == Instructions::SUB as u8 {
                let b = self.data_pop();
                let a = self.data_pop();
                self.data.push(a - b);
            } else if self.current() == Instructions::MUL as u8 {
                let b = self.data_pop();
                let a = self.data_pop();
                self.data.push(a * b);
            } else if self.current() == Instructions::DIV as u8 {
                let b = self.data_pop();
                let a = self.data_pop();
                self.data.push(a / b);
            } else if self.current() == Instructions::CALL as u8 {
                let fid = self.step_moved();

                let imports = self.imports.clone();
                let f = imports.get(fid as usize).unwrap();
                f();
            }

            self.step_counter();
        }
    }

    pub fn data_peek(&self) -> Vec<u32> {
        self.data.clone()
    }

    pub fn data_pop(&mut self) -> u32 {
        self.data.pop().unwrap()
    }
}

fn main() {
    let code: &[u8] = &[
        inst!(PUSH),
        6,
        inst!(PUSH),
        8,
        inst!(PUSH),
        2,
        div!(),
        add!(),
        inst!(CALL),
        0,
        inst!(STOP),
    ];

    let hello: Rc<dyn Fn()> = Rc::new(|| {
        println!("lol");
    });

    let exports = vec![hello];

    let mut mach = VM::new(code.to_vec(), exports);

    mach.execute();

    println!("{:?}", mach.data_peek());
}
