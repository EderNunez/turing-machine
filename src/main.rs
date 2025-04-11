use std::io::{BufRead, Write};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Step {
    L,
    R,
}

impl TryFrom<&str> for Step {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "L" => Ok(Self::L),
            "R" => Ok(Self::R),
            _ => Err(format!("{value} is not a valid step. Expected 'L' or 'R'",)),
        }
    }
}

type State<'a> = &'a str;
type Symbol<'a> = &'a str;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Turd<'a> {
    current: State<'a>,
    read: Symbol<'a>,
    write: Symbol<'a>,
    step: Step,
    next: State<'a>,
}

impl<'a> Turd<'a> {
    const fn new(
        current: State<'a>,
        read: Symbol<'a>,
        write: Symbol<'a>,
        step: Step,
        next: State<'a>,
    ) -> Self {
        Self {
            current,
            read,
            write,
            step,
            next,
        }
    }

    fn parse_turd(filepath: &str, s: (usize, &'a str)) -> Result<Self, String> {
        let tokens = s.1.split_whitespace().map(str::trim).collect::<Vec<_>>();
        if tokens.len() != 5 {
            return Err(format!(
                "{filepath}:{}: A single turd is expected to have 5 tokens",
                s.0 + 1
            ));
        }
        let current = 0;
        let read = 1;
        let write = 2;
        let step = 3;
        let next = 4;
        Ok(Self::new(
            tokens[current],
            tokens[read],
            tokens[write],
            tokens[step].try_into()?,
            tokens[next],
        ))
    }
    fn states_of_turds(turds: &[Self]) -> Box<[State]> {
        let mut states = std::collections::BTreeSet::new();
        for turd in turds {
            states.insert(turd.current);
            states.insert(turd.next);
        }
        states.iter().copied().collect()
    }
}

#[derive(Debug)]
struct Machine<'a> {
    tape: Box<[Symbol<'a>]>,
    head: usize,
    state: State<'a>,
}

impl<'a> Machine<'a> {
    const fn new(tape: Box<[Symbol<'a>]>, head: usize, state: State<'a>) -> Self {
        Self { tape, head, state }
    }

    fn next(&mut self, program: &'a [Turd]) -> bool {
        for turd in program {
            if turd.current == self.state && turd.read == self.tape[self.head] {
                self.tape[self.head] = turd.write;
                match turd.step {
                    Step::L if self.head == 0 => self.head = self.tape.len() - 1,
                    Step::L => self.head -= 1,
                    Step::R => self.head = (self.head + 1) % self.tape.len(),
                }
                self.state = turd.next;
                return true;
            }
        }
        false
    }

    fn dump(&self) {
        println!("STATE: {}", self.state);
        self.tape.iter().for_each(|cell| print!("{cell} "));
        println!();
        self.tape.iter().enumerate().for_each(|(i, cell)| {
            if i == self.head {
                print!("^");
            }
            (0..cell.len()).for_each(|_| print!(" "));
            if i != self.head {
                print!(" ");
            }
        });
        println!();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Box<[_]>>();
    if args.len() < 3 {
        eprintln!("Error: input file is not provided");
        return Err("Usage: .\turing-machine <input.turd> <input.tape>".into());
    }
    let turd_filepath = &args[1];
    let tape_filepath = &args[2];

    let content = std::fs::read_to_string(turd_filepath)?;
    let turds = content
        .lines()
        .map(str::trim)
        .enumerate()
        .filter(|x| !x.1.is_empty())
        .map(|x| Turd::parse_turd(turd_filepath, x).unwrap())
        .collect::<Box<[_]>>();

    let states = Turd::states_of_turds(&turds);

    println!("Possible states:");
    states.iter().for_each(|state| println!("{state}"));
    print!("Initial_state: ");
    std::io::stdout().flush()?;
    let initial_state = std::io::stdin().lock().lines().next().unwrap()?;
    println!();

    let binding = std::fs::read_to_string(tape_filepath)?;
    let mut machine = Machine::new(
        binding
            .split_whitespace()
            .map(str::trim)
            .collect::<Box<[_]>>(),
        0,
        &initial_state,
    );
    loop {
        machine.dump();
        std::thread::sleep(std::time::Duration::from_millis(100));
        if !machine.next(&turds) {
            break;
        }
    }

    Ok(())
}
