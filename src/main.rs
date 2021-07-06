use std::{
    collections::HashMap,
    fmt::Arguments,
    fs::{self, write},
    ops::Add,
    rc::Rc,
    usize,
};

#[repr(u8)]
enum Instructions {
    STOP = 0,
    PUSH, // push data vector
    POP,
    VREFSTART, // start vref decl
    /*
     * VREFSTART
     * NAMEBYTES
     * VREFNAMEEND
     * location on data vector
     * VREFEND
     */
    VREFNAMEEND,
    VREFEND,
    ADD,
    SUB,
    MUL,
    DIV,
    LOOPN,
    LOOPEND,
    CALL, // call rust fn
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
    vrefs: HashMap<String, usize>,
    imports: Vec<Rc<dyn Fn() + 'static>>,
    counter: usize,
}

impl VM {
    pub fn new(code: Vec<u8>, imports: Vec<Rc<dyn Fn()>>) -> Self {
        Self {
            code,
            data: vec![],
            vrefs: HashMap::new(),
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

    fn executor(&mut self, byte: u8) {
        if byte == Instructions::PUSH as u8 {
            let data = self.step_moved();

            self.data.push(data.into());
        } else if byte == Instructions::ADD as u8 {
            let b = self.data_pop();
            let a = self.data_pop();

            self.data.push(a + b);
        } else if byte == Instructions::SUB as u8 {
            let b = self.data_pop();
            let a = self.data_pop();
            self.data.push(a - b);
        } else if byte == Instructions::MUL as u8 {
            let b = self.data_pop();
            let a = self.data_pop();
            self.data.push(a * b);
        } else if byte == Instructions::DIV as u8 {
            let b = self.data_pop();
            let a = self.data_pop();
            self.data.push(a / b);
        } else if byte == Instructions::CALL as u8 {
            let fid = self.step_moved();

            let imports = self.imports.clone();
            let f = imports.get(fid as usize).unwrap();
            f();
        } else if byte == Instructions::VREFSTART as u8 {
            let mut name = String::new();
            loop {
                let c = self.step_moved();
                if c == Instructions::VREFNAMEEND as u8 {
                    break;
                }

                if !(c as char).is_alphanumeric() {
                    panic!("invalid vname entry");
                }

                name += &format!("{}", c as char);
            }

            let refloc = self.step_moved() as usize;
            self.vrefs.insert(name, refloc);

            if self.step_moved() != Instructions::VREFEND as u8 {
                panic!("var ref addr not provided");
            }
        } else if byte == Instructions::LOOPN as u8 {
            let till = self.step_moved();

            let mut instructions = vec![];

            loop {
                let b = self.current();

                instructions.push(b);
                if b == inst!(LOOPEND) {
                    break;
                }
                self.counter += 1;
            }

            for _ in 0..till {
                for b in instructions.iter() {
                    self.executor(*b);
                }
            }
        }
    }

    pub fn execute(&mut self) {
        while self.counter < self.code.len() {
            self.executor(self.current());
            self.step_counter();
        }
    }

    pub fn peek_var(&self, varname: String) -> u32 {
        let &addr = self.vrefs.get(&varname).expect("variable doesnt exist");

        *self.data.get(addr).expect("var points to empty loc")
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
        inst!(VREFSTART),
        'g' as u8,
        'l' as u8,
        'o' as u8,
        'b' as u8,
        'a' as u8,
        'l' as u8,
        inst!(VREFNAMEEND),
        0,
        inst!(VREFEND),
        inst!(PUSH),
        15,
        add!(),
        inst!(LOOPN),
        5,
        inst!(PUSH),
        5,
        inst!(LOOPEND),
        inst!(STOP),
    ];

    write("binfile", code);

    // RUN FROM FILE
    // let code = fs::read("binfile").unwrap();
    // let code = code.as_slice();

    let exports = vec![];

    let mut mach = VM::new(code.to_vec(), exports);

    mach.execute();

    println!("{:?}", mach.data_peek());
    println!("{:?}", mach.peek_var("global".to_string()));
}
