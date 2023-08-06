use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
struct CoverageState {
    lines: std::collections::HashSet<u32>,
    new: bool,
}

struct Corpus {
    db: Vec<Vec<u8>>
}

impl basic_mutator::InputDatabase for Corpus {
    fn num_inputs(&self) -> usize {
        self.db.len()
    }

    fn input(&self, idx: usize) -> Option<&[u8]> {
        Some(&self.db.get(idx)?[..])
    }
}

impl Corpus {
    fn new() -> Self {
        Self {
            db: Vec::new(),
        }
    }

    fn push(&mut self, input: &[u8]) {
        self.db.push(Vec::from(input));
    }
}

impl CoverageState {
    pub fn new() -> Rc<RefCell<Self>> {
        let state = CoverageState {
            lines: Default::default(),
            new: false,
        };

        Rc::new(RefCell::new(state))
    }

    pub fn insert(&mut self, line: u32) {
        self.new |= self.lines.insert(line);
    }

    pub fn resetnew(&mut self) {
        self.new = false;
    }

    pub fn hasnew(&self) -> bool {
        self.new
    }
}

fn main() {
    let cs = CoverageState::new();
    let mut db = Corpus::new();

    let vm = mlua::Lua::new();
    let src = std::fs::read_to_string("./tests/crashme.lua").unwrap();
    let chunk = vm.load(&src).exec();

    let cb = {
        let cs = cs.clone();
        move |_: &mlua::Lua, debug: mlua::Debug| {
            let mut cs = cs.borrow_mut();
            cs.insert(debug.curr_line() as u32);
            Ok(())
        }
    };

    vm.set_hook(mlua::HookTriggers::every_line(), cb).unwrap();
    let crashme = vm.globals().get::<_,mlua::Function>("crashme").unwrap();
    let mut mutator = basic_mutator::Mutator::new()
        .printable(true)
        .max_input_size(64);

    let mut iter = 0;

    loop {
        mutator.mutate(1, &db);
        let input = std::str::from_utf8(&mutator.input).unwrap();

        match crashme.call::<_,()>(input) {
            Ok(_) => {
                let mut cs = cs.borrow_mut();
                if cs.hasnew() {
                    println!("cov: {} input: {:?}", cs.lines.len(), input);
                    db.push(&mutator.input);
                }
            }
            Err(_) => {
                println!("found crash with: {:?}", input);
                return;
            }
        }

        iter += 1;

        let mut cs = cs.borrow_mut();
        cs.resetnew();
    }
}
