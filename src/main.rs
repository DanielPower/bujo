use std::cmp::max;
use std::io::{stdin, stdout, BufRead, Stdout, Write};
use termion::cursor::{DetectCursorPos, Goto};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

enum Mode {
    Normal,
    Todo,
    Note,
    Journal,
}

#[derive(Debug)]
struct Todo {
    text: String,
    complete: bool,
}

#[derive(Debug)]
struct Journal {
    text: String,
}

#[derive(Debug)]
struct Note {
    text: String,
}

#[derive(Debug)]
enum Node {
    Todo(Todo),
    Journal(Journal),
    Note(Note),
}

struct State {
    mode: Mode,
    terminal: RawTerminal<Stdout>,
    close: bool,
    nodes: Vec<Node>,
}

fn normal(mut state: State, evt: Event) -> State {
    let (x, y) = state.terminal.cursor_pos().unwrap();
    match evt {
        Event::Key(Key::Char('h')) => {
            print!("{}", Goto(max(1, x - 1), y));
        }
        Event::Key(Key::Char('j')) => {
            print!("{}", Goto(x, max(1, y + 1)));
        }
        Event::Key(Key::Char('k')) => {
            print!("{}", Goto(x, max(1, y - 1)));
        }
        Event::Key(Key::Char('l')) => {
            print!("{}", Goto(max(1, x + 1), y));
        }
        Event::Key(Key::Char('.')) => {
            print!("• ");
            state.mode = Mode::Todo;
        }
        Event::Key(Key::Char('-')) => {
            state.mode = Mode::Note;
        }
        Event::Key(Key::Char('\'')) => {
            print!("| ");
            state.mode = Mode::Journal;
        }
        _ => {}
    };
    state
}

fn journal(mut state: State, evt: Event) -> State {
    match evt {
        Event::Key(Key::Esc) => {
            print!("\r\n");
            state.mode = Mode::Normal
        }
        Event::Key(Key::Char('\n')) => print!("\r\n|  "),
        Event::Key(Key::Char(key)) => {
            print!("{}", key);
            state.mode = Mode::Journal
        }
        _ => {}
    };
    state
}

fn todo(mut state: State, evt: Event) -> State {
    match evt {
        Event::Key(Key::Esc) => {
            print!("\r\n");
            state.mode = Mode::Normal
        }
        Event::Key(Key::Char('\n')) => {
            print!("\r\n");
            state.mode = Mode::Normal
        }
        Event::Key(Key::Char(key)) => {
            print!("{}", key);
            state.mode = Mode::Todo
        }
        _ => {}
    };
    state
}

fn load_file() -> Vec<Node> {
    let mut nodes = Vec::new();
    let file = std::fs::File::open("test.bujo").unwrap();
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        print!("{:?}", line);
        let line = line.unwrap();
        if line.starts_with(". ") {
            nodes.push(Node::Todo(Todo {
                text: line[2..].to_string(),
                complete: false,
            }));
        } else if line.starts_with("x ") {
            nodes.push(Node::Todo(Todo {
                text: line[2..].to_string(),
                complete: true,
            }));
        } else if line.starts_with("| ") {
            nodes.push(Node::Journal(Journal {
                text: line[2..].to_string(),
            }));
        } else if line.starts_with("- ") {
            nodes.push(Node::Note(Note {
                text: line[2..].to_string(),
            }));
        }
    }
    nodes
}

fn render_statusbar(state: &State) {
    print!(
        "{}",
        match state.mode {
            Mode::Normal => "Normal  ",
            Mode::Todo => "Todo    ",
            Mode::Note => "Note    ",
            Mode::Journal => "Journal ",
        },
    );
}

fn render_node(node: &Node) {
    match node {
        Node::Todo(todo) => {
            if todo.complete {
                print!("x ");
            } else {
                print!("• ");
            }
            print!("{}", todo.text);
        }
        Node::Journal(journal) => {
            print!("| ");
            print!("{}", journal.text);
        }
        Node::Note(note) => {
            print!("- ");
            print!("{}", note.text);
        }
    }
}

fn update(mut state: State, event: Event) -> State {
    if event == Event::Key(Key::Ctrl('c')) {
        state.close = true;
        return state;
    }
    state = match state.mode {
        Mode::Normal => normal(state, event),
        Mode::Todo => todo(state, event),
        // Mode::Note => state,
        Mode::Journal => journal(state, event),
        _ => state,
    };
    let (_width, height) = termion::terminal_size().unwrap();
    let (x, y) = state.terminal.cursor_pos().unwrap();
    print!("{}", Goto(1, height));
    render_statusbar(&state);
    print!("{}", Goto(x, y));
    for node in &state.nodes {
        render_node(node);
        print!("\r\n");
    }

    state.terminal.flush().unwrap();
    state
}

fn main() {
    let stdin = stdin();
    let mut state = State {
        mode: Mode::Normal,
        terminal: stdout().into_raw_mode().unwrap(),
        nodes: load_file(),
        close: false,
    };
    print!("{}{}", termion::clear::All, Goto(1, 1));
    state.terminal.flush().unwrap();

    state = update(state, Event::Unsupported(vec![0]));
    for c in stdin.events() {
        state = update(state, c.unwrap());
        if state.close {
            print!("{}{}", Goto(1, 1), termion::clear::All);
            state.terminal.flush().unwrap();
            break;
        }
    }
}
