use std::process::ExitCode;
use std::env;
use std::fs;

use cognition::*;
use cognition::macros::*;
use cognition::math::Math;

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
  let mut vstack = Box::new(VStack::with_capacity(DEFAULT_STACK_SIZE));
  vstack.container.faliases = Container::default_faliases();
  state.stack.push(Value::Stack(vstack));
  state.parser = Some(Parser::new(None, None));
  for arg in args[opts.fileidx+opts.s..].iter() {
    state.args.push(Value::Word(Box::new(VWord{ str_word: arg.clone() })));
  }
  builtins::add_builtins(&mut state);

  for i in 0..opts.s {
    // Read code from file
    let filename = &args[opts.fileidx + i];
    let mut fs_result = fs::read_to_string(filename);
    if let Err(_) = fs_result {
      if let Ok(dir) = env::var("COGLIB_DIR") {
        fs_result = fs::read_to_string(format!("{dir}/{filename}")); }}
    if let Err(e) = fs_result {
      println!("Could not open file for reading: {filename}: {e}");
      return ExitCode::from(4);
    }
    let source: String = fs_result.unwrap();
    let mut parser = state.parser.take().unwrap();
    if let Some(s) = parser.source() { state.pool.add_string(s) }
    parser.reset(source, Some(state.string_copy(filename)));
    state.parser = Some(parser);

    // Parse and eval loop
    loop {
      let w = state.parser_get_next();
      match w {
        Some(v) => state = state.eval(v),
        None => break,
      }
      if state.exited { break }
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

  let mut math = Math::new();
  let digits = String::from("0123456789↊↋🜘");
  math.set_digits(&digits);
  math.set_negc('\u{0305}');
  math.set_radix('.');
  math.set_delim(',');
  math.set_meta_radix(';');
  math.set_meta_delim(':');
  math.set_base(24);

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
        match math.stoi(arg) {
          Ok(i) => if i < 0 || i > i32::MAX as isize {
            println!("Index (s) out of range");
            return Err(ExitCode::from(2))
          } else {
            s = i as i32;
          },
          Err(_) => return Err(usage(3)),
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

  let s: usize = if s < 0 { 1 } else { s as usize };

  Ok(Config{h, q, v, s, fileidx})
}

fn usage(code: u8) -> ExitCode {
  println!("Usage: crank [-hfqsv] [file...] [arg...]");
  ExitCode::from(code)
}

fn help() -> ExitCode {
  usage(0);
  println!(" -h    --help            print this help message");
  println!(" -q    --quiet           don't show state information at program end");
  println!(" -s N  --sources N       specify N source files to be composed (default is N=1)");
  println!(" -v    --version         print version information");
  ExitCode::from(0)
}

fn version() -> ExitCode {
  println!("Authors: Matthew Hinton, Preston Pan, MIT License 2024");
  println!("cognition, version 0.1.2 alpha");
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
      print!(" '");
      alias.print_pretty();
      print!("'");
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

  if let Some(ref math) = cur.math {
    println!("Math:");
    println!("base: {}", math.base());
    print!("digits: ");
    for d in math.get_digits() {
      print!("{d}");
    }
    println!("");
    print!("negc: ");
    match math.get_negc() {
      Some(c) => println!("'{}\u{00A0}'", c),
      None => println!("(none)"),
    }
    print!("radix: ");
    match math.get_radix() {
      Some(c) => println!("'{c}'"),
      None => println!("(none)"),
    }
    print!("delim: ");
    match math.get_delim() {
      Some(c) => println!("'{c}'"),
      None => println!("(none)"),
    }
    print!("meta-radix: ");
    match math.get_meta_radix() {
      Some(c) => println!("'{c}'"),
      None => println!("(none)"),
    }
    print!("meta-delim: ");
    match math.get_meta_delim() {
      Some(c) => println!("'{c}'"),
      None => println!("(none)"),
    }
    println!("");
  } else {
    println!("uninitialized math\n");
  }

  println!("Pool:");
  state.pool.print();
  println!("");

  if let Some(code) = &state.exit_code {
    print!("Exit code: '");
    code.print_pretty();
    println!("'");
  } else {
    println!("Exit code: (none)");
  }
}
