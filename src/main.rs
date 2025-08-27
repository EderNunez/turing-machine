use std::{
    collections::BTreeSet,
    error::Error,
    fmt::Display,
    io::{BufRead, Write},
};

#[derive(Debug)]
enum TuringMachineError {
    Parse(String),
    Transformation(String),
    Args(String),
    Io(std::io::Error),
}

impl From<std::io::Error> for TuringMachineError {
    fn from(v: std::io::Error) -> Self {
        Self::Io(v)
    }
}

impl Error for TuringMachineError {}

impl Display for TuringMachineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(s) | Self::Transformation(s) | Self::Args(s) => {
                write!(f, "{s}")
            }
            Self::Io(error) => error.fmt(f),
        }
    }
}

enum Step {
    L,
    R,
}

impl TryFrom<&str> for Step {
    type Error = TuringMachineError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "L" => Ok(Self::L),
            "R" => Ok(Self::R),
            _ => Err(TuringMachineError::Transformation(format!(
                "{value} is not a valid step. Expected 'L' or 'R'"
            ))),
        }
    }
}

type State<'a> = &'a str;
type Symbol<'a> = &'a str;

struct Turd<'a> {
    current: State<'a>,
    read: Symbol<'a>,
    write: Symbol<'a>,
    step: Step,
    next: State<'a>,
}

impl<'a> Turd<'a> {
    fn parse_turd(filepath: &str, s: (usize, &'a str)) -> Result<Self, TuringMachineError> {
        const CURRENT: usize = 0;
        const READ: usize = 1;
        const WRITE: usize = 2;
        const STEP: usize = 3;
        const NEXT: usize = 4;
        let tokens = s.1.split_whitespace().map(str::trim).collect::<Vec<_>>();
        if tokens.len() != 5 {
            return Err(TuringMachineError::Parse(format!(
                "{filepath}:{}: A single turd is expected to have 5 tokens",
                s.0 + 1
            )));
        }

        Ok(Self {
            current: tokens[CURRENT],
            read: tokens[READ],
            write: tokens[WRITE],
            step: tokens[STEP].try_into()?,
            next: tokens[NEXT],
        })
    }

    fn states_of_turds(turds: &[Self]) -> impl Iterator<Item = State<'_>> {
        turds
            .iter()
            .flat_map(|t| [t.current, t.next])
            .collect::<BTreeSet<_>>()
            .into_iter()
    }
}

struct Machine<'a> {
    tape: Vec<Symbol<'a>>,
    head: usize,
    state: State<'a>,
}

impl Display for Machine<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "STATE: {}", self.state)?;
        self.tape.iter().try_for_each(|cell| write!(f, "{cell} "))?;
        writeln!(f)?;
        for (i, cell) in self.tape.iter().enumerate() {
            if i == self.head {
                write!(f, "^")?;
            }
            (0..cell.len()).try_for_each(|_| write!(f, " "))?;
            if i != self.head {
                write!(f, " ")?;
            }
        }
        writeln!(f)
    }
}

impl<'a> Machine<'a> {
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
}

fn main() -> Result<(), TuringMachineError> {
    let mut args = std::env::args();
    let program = args.next().unwrap();
    if args.len() < 2 {
        eprintln!("Error: input file is not provided");
        return Err(TuringMachineError::Args(format!(
            "Usage: {program} <input.turd> <input.tape>"
        )));
    }
    let turd_filepath = &args.next().unwrap();
    let tape_filepath = &args.next().unwrap();

    let content = std::fs::read_to_string(turd_filepath)?;
    let turds = content
        .lines()
        .map(str::trim)
        .enumerate()
        .filter_map(|x| {
            if x.1.is_empty() {
                None
            } else {
                Some(Turd::parse_turd(turd_filepath, x))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let states = Turd::states_of_turds(&turds);

    println!("Possible states:");
    states.for_each(|state| println!("{state}"));
    print!("Initial_state: ");
    std::io::stdout().flush()?;
    let initial_state = std::io::stdin().lock().lines().next().unwrap()?;
    println!();

    let binding = std::fs::read_to_string(tape_filepath)?;
    let mut machine = Machine {
        tape: binding.split_whitespace().map(str::trim).collect(),
        head: 0,
        state: &initial_state,
    };
    loop {
        print!("{machine}");
        std::thread::sleep(std::time::Duration::from_millis(100));
        if !machine.next(&turds) {
            break;
        }
    }
    Ok(())
}
