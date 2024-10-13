#![allow(dead_code)]

use std::process::ExitCode;
use std::env;
use std::fs;

use crate::cognition::*;
use crate::cognition::macros::*;

#[macro_use]
mod cognition;
mod pool;

// to be reimplemented properly
fn isint(_n: &String) -> bool { true }
fn string_to_i32(_n: &String) -> Result<i32, &'static str> { Ok(2) }

// figure out a better solution
// macro_rules! cognition_destroy_parser {
//   ($state:ident) => {
//     {
//       $state.args.clear();
//       CognitionState{ stack: $state.stack,
//                       parser: None,
//                       exited: false,
//                       exit_code: None,
//                       args: $state.args,
//                       pool: $state.pool,
//                       i: $state.i }
//     }
//   }
// }

fn main() -> ExitCode {
  let args: Vec<String> = env::args().collect();
  let argc = args.len();

  let parse = parse_configs(&args, argc);
  let opts = match parse {
    Err(e) => return e,
    Ok(o) => o,
  };

  if opts.h { return help(); }
  if opts.v { return version(); }
  if opts.fileidx == 0 && opts.s != 0 {
    return usage(3);
  }

  // Initialize state
  let metastack = Stack::with_capacity(DEFAULT_STACK_SIZE);
  let mut state = CognitionState::new(metastack);
  let mut initial_stack = Value::Stack(Box::new(VStack::with_capacity(DEFAULT_STACK_SIZE)));
  let Value::Stack(vstack) = &mut initial_stack else { panic!("fatal error") };
  vstack.container.faliases = Container::default_faliases();
  state.stack.push(initial_stack);

  for i in 0..opts.s {
    // Read code from file
    let filename = &args[opts.fileidx + i];
    let fs_result = fs::read_to_string(filename);
    if let Err(e) = fs_result {
      println!("Could not open file for reading: {filename}: {e}");
      return ExitCode::from(4);
    }
    let buffer: String = fs_result.unwrap();
    let mut parser = Parser::new(&buffer);

    // Parse and eval loop
    loop {
      let val = parser.get_next(&mut state);
      match val {
        Some(v) => state = state.eval(v),
        None => break,
      }
      if state.exited { break; }
    }
  }

  if !opts.q { print_end(&state); }

  ExitCode::SUCCESS
}

struct Config {
  h: bool,
  q: bool,
  v: bool,
  s: usize,
  fileidx: usize,
}

fn parse_configs(args: &Vec<String>, argc: usize) -> Result<Config, ExitCode> {
  if args.len() < 2 {
    return Err(usage(1));
  }

  let (mut h, mut q, mut v) = (false, false, false);
  let mut s: i32 = -1;
  let mut fileidx = 0;

  let mut i = 1;
  while i < argc {
    match args[i].as_str() {
      "-h" | "--help" => {
        if h { return Err(usage(1)); }
        else { h = true; }
      }
      "-q" | "--quit" => {
        if q { return Err(usage(1)); }
        else { q = true; }
      }
      "-v" | "--version" => {
        if v { return Err(usage(1)); }
        else { v = true; }
      }
      "-s" | "--sources" => {
        if s >= 0 {
          return Err(usage(1));
        } else if i + 1 == argc {
          return Err(usage(3));
        }
        i += 1;
        let arg = &args[i];
        match string_to_i32(arg) {
          Ok(i) => s = i,
          Err(_) => return Err(usage(2)),
        }
      }
      _ => {
        fileidx = i;
        if i as i64 + s as i64 > argc as i64 {
          println!("Missing filename");
          return Err(ExitCode::from(3));
        }
        break;
      }
    }
    i += 1;
  }

  let s: usize = if s < 0 { 1 }
                  else { s as usize };

  Ok(Config{h, q, v, s, fileidx})
}

fn usage(code: u8) -> ExitCode {
  println!("Usage: crank [-hqsv] [file...] [arg...]");
  ExitCode::from(code)
}

fn help() -> ExitCode {
  usage(0);
  println!(" -h    --help            print this help message");
  println!(" -q    --quiet           don't show stack information at program end");
  println!(" -s N  --sources N       specify N source files to be composed (default is N=1)");
  println!(" -v    --version         print version information");
  ExitCode::from(0)
}

fn version() -> ExitCode {
  println!("Authors: Preston Pan, Matthew Hinton, MIT License 2024");
  println!("cognition, version 1.0 alpha");
  ExitCode::from(0)
}

fn print_end(state: &CognitionState) {
  println!("");
  println!("Stack at end:");
  let cur = state.current_ref();
  for v in cur.stack.iter() { v.print("\n"); }
  println!("");
  println!("Error stack:");
  if let Some(errors) = &cur.err_stack {
    for verror in errors.iter() { verror.print("\n"); }
  }
  if let Some(faliases) = &cur.faliases {
    print!("\nFaliases:");
    for alias in faliases.iter() {
      b" '".print_pretty();
      alias.print_pretty();
      b"'".print_pretty();
    }
  }
  println!("");
  print!("delims: '");
  if let Some(delims) = &cur.delims { delims.print_pretty(); }
  if cur.dflag { println!("' (whitelist)"); }
  else         { println!("' (blacklist)"); }
  print!("ignored: '");
  if let Some(ignored) = &cur.ignored { ignored.print_pretty(); }
  if cur.iflag { println!("' (whitelist)"); }
  else         { println!("' (blacklist)"); }
  print!("singlets: '");
  if let Some(singlets) = &cur.singlets { singlets.print_pretty(); }
  if cur.sflag { println!("' (whitelist)\n"); }
  else         { println!("' (blacklist)\n"); }

  if let Some(cranks) = &cur.cranks {
    if cranks.len() == 0 {
      println!("crank 0");
    } else {
      print!("cranks:");
      for (i, crank) in cranks.iter().enumerate() {
        print!(" {i}:({},{})", crank.modulo, crank.base);
      }
      println!("");
    }
  } else {
    println!("uninitialized crank");
  }
  println!("");
  if let Some(code) = &state.exit_code {
    print!("Exit code: '");
    code.print_pretty();
    println!("'");
  } else {
    println!("Exit code: (none)");
  }
}
