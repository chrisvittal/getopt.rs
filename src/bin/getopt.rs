use std::{env, io, path::Path, process};

use getopt::prelude::*;

mod my {
    use std::io;

    pub use self::Result::*;

    pub enum Result<T> {
        Ok(T),
        External(io::Error),
        Internal(io::Error),
    }
}

enum ShellKind {
    Bourne,
    C,
    Fish,
    Rc,
}

fn main() -> ! {
    process::exit(match program() {
        my::Ok(code) => code,
        my::External(error) => {
            eprintln!("{}", error);
            1
        }
        my::Internal(error) => {
            eprintln!("{}", error);
            2
        }
    });
}

#[rustfmt::skip]
fn print_usage(program: &str) -> my::Result<i32> {
    println!("Usage: {} [-h] [-n name] [-s shell] optstring [args ...]", program);
    println!("  -h        display this help");
    println!("  -n name   report errors as 'name' (default '{}')", program);
    println!("  -s shell  use quoting conventions for shell (default 'sh')");

    my::Ok(0)
}

fn program() -> my::Result<i32> {
    let program = program_name("getopt");
    let args = program_args();
    let mut parsed: Vec<String> = Vec::new();

    let mut name = program.clone();
    let mut shell = ShellKind::Bourne;

    // gather our own options
    let mut opts = Parser::new(&args, "hn:s:");
    loop {
        match opts.next() {
            None => break,
            Some(Err(error)) => {
                return my::Internal(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{}: {}", program, error),
                ));
            }
            Some(Ok(opt)) => match opt {
                Opt('h', None) => {
                    print_usage(&program);
                    return my::Ok(0);
                }
                Opt('n', Some(arg)) => name = arg,
                Opt('s', Some(arg)) => {
                    shell = match arg.to_lowercase().trim() {
                        "ash" | "bash" | "dash" | "ksh" | "mksh" | "sh" | "zsh" => {
                            ShellKind::Bourne
                        }
                        "csh" | "tcsh" => ShellKind::C,
                        "fish" => ShellKind::Fish,
                        "plan9" | "rc" => ShellKind::Rc,
                        x => {
                            return my::Internal(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                format!("{}: unknown shell type: {}", program, x),
                            ));
                        }
                    }
                }
                _ => unreachable!(),
            },
        }
    }

    let optstring = match args.get(opts.index()) {
        None => {
            return my::Internal(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{}: missing optstring argument", program),
            ));
        }
        Some(s) => s,
    };
    let index = opts.index() + 1;

    // parse the other options
    let mut opts = Parser::new(&args, optstring);
    opts.set_index(index);
    loop {
        match opts.next() {
            None => break,
            Some(Err(error)) => {
                return my::External(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{}: {}", name, error),
                ));
            }
            Some(Ok(Opt(opt, arg))) => {
                parsed.push(format!("-{}", opt));
                match arg {
                    None => (),
                    Some(s) => parsed.push(quote_for_shell(&s, &shell)),
                }
            }
        }
    }

    parsed.push("--".to_string());

    for arg in args.clone().split_off(opts.index()) {
        parsed.push(quote_for_shell(&arg, &shell));
    }

    println!("{}", parsed.join(" "));

    my::Ok(0)
}

fn quote_for_shell(string: &str, kind: &ShellKind) -> String {
    match kind {
        // most shells (sh, ksh, zsh, bash, (d)ash, etc.) are in this category
        ShellKind::Bourne => {
            let e = '\\'; // escape char
            let q = '\''; // quote char
            let mut new_string = String::new();
            new_string.push(q);
            for c in string.chars() {
                match c {
                    '\'' => {
                        new_string.push(q);
                        new_string.push(e);
                        new_string.push(c);
                        new_string.push(q);
                    }
                    _ => new_string.push(c),
                }
            }
            new_string.push(q);
            new_string
        }

        ShellKind::C => {
            let e = '\\'; // escape char
            let q = '\''; // quote char
            let mut new_string = String::new();
            new_string.push(q);
            for c in string.chars() {
                match c {
                    ' ' | '\'' => {
                        new_string.push(q);
                        new_string.push(e);
                        new_string.push(c);
                        new_string.push(q);
                    }
                    _ => new_string.push(c),
                }
            }
            new_string.push(q);
            new_string
        }

        ShellKind::Fish => {
            let e = '\\'; // escape char
            let q = '\''; // quote char
            let mut new_string = String::new();
            new_string.push(q);
            for c in string.chars() {
                match c {
                    '\'' | '\\' => {
                        new_string.push(e);
                        new_string.push(c);
                    }
                    _ => new_string.push(c),
                }
            }
            new_string.push(q);
            new_string
        }

        ShellKind::Rc => {
            let q = '\''; // quote char
            let mut new_string = String::new();
            new_string.push(q);
            for c in string.chars() {
                match c {
                    '\'' => {
                        new_string.push(q);
                        new_string.push(c);
                    }
                    _ => new_string.push(c),
                }
            }
            new_string.push(q);
            new_string
        }
    }
}

fn program_args() -> Vec<String> {
    env::args_os()
        .map(|a| a.to_string_lossy().into_owned())
        .collect()
}

fn program_name(default: &str) -> String {
    match env::args_os().next() {
        None => default.to_string(),
        Some(os_string) => match Path::new(&os_string).file_name() {
            None => default.to_string(),
            Some(os_str) => os_str.to_string_lossy().into_owned(),
        },
    }
}
