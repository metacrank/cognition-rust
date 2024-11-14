#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
#![allow(unused_imports)]
use std::process::ExitCode;
use std::env;
use std::fs;
use std::fs::File;

use std::any::Any;
use std::io::Write;
use std::io::Read;

use cognition::*;
use cognition::macros::*;
use cognition::serde::CognitionDeserialize;
use cognition::builtins::io::ReadWriteCustom;

use serde_json;
use ::serde::{Serialize, Serializer};

const VERSION: &'static str = "0.2.1 alpha";

fn main() -> ExitCode {
  let args: Vec<String> = env::args().collect();
  let argc = args.len();

  let parse = parse_configs(&args, argc);
  let opts = match parse {
    Err(e) => return e,
    Ok(o) => o,
  };

  if opts.help { return help(); }
  if opts.version { return version(); }
  if opts.list_formats { return list_formats(); }
  if opts.fileidx == 0 && opts.sources != 0 {
    println!("{}: missing filename", binary_name());
    return try_help(3)
  }

  if let Some(ref format) = opts.format {
    if !DATA_FORMATS.iter().any(|x| x == format) {
      println!("{}: invalid format -- '{}'", binary_name(), format);
      return try_help(2)
    }
  }

  let mut logfile = if let Some(ref s) = opts.logfile {
    match File::options().write(true).truncate(true).create(true).open(s) {
      Ok(f) => Some(f),
      Err(e) => {
        println!("{}: could not open logfile: {}: {e}", binary_name(), opts.logfile.as_ref().unwrap());
        return ExitCode::from(4);
      }
    }
  } else { None };

  // Initialize state
  let mut state = match opts.load {
    Some(ref loadfile) => {
      match load(loadfile) {
        Ok(state) => state,
        Err(e) => return e
      }
    },
    None => {
      let metastack = Stack::with_capacity(DEFAULT_STACK_SIZE);
      let mut state = CognitionState::new(metastack);
      let mut vstack = Box::new(VStack::with_capacity(DEFAULT_STACK_SIZE));
      vstack.container.faliases = Container::default_faliases();
      state.stack.push(Value::Stack(vstack));
      for arg in args[(opts.fileidx + opts.sources)..].iter() {
        state.args.push(Value::Word(Box::new(VWord{ str_word: arg.clone() })));
      }
      builtins::add_builtins(&mut state);
      state
    }
  };

  if state.parser.is_none() {
    state.parser = Some(Parser::new(None, None));
  }

  for i in 0..opts.sources {
    // Read code from file
    let filename = &args[opts.fileidx + i];
    let mut fs_result = fs::read_to_string(filename);
    if let Err(_) = fs_result {
      if let Some(ref dir) = opts.coglib {
        fs_result = fs::read_to_string(format!("{dir}/{filename}"));
      } else if let Ok(dir) = env::var("COGLIB_DIR") {
        fs_result = fs::read_to_string(format!("{dir}/{filename}"));
      }
    }
    if let Err(e) = fs_result {
      println!("{}: could not open file for reading: {filename}: {e}", binary_name());
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
        Some(v) => {
          if let Some(f) = &mut logfile { v.fprint(f, "\n") }
          state = state.eval(v)
        },
        None => break,
      }
      if state.exited { break }
    }
  }

  if !opts.quiet { print_end(&state); }

  ExitCode::SUCCESS
}

struct Config {
  help: bool,
  coglib: Option<String>,
  format: Option<String>,
  logfile: Option<String>,
  load: Option<String>,
  list_formats: bool,
  quiet: bool,
  version: bool,
  sources: usize,
  fileidx: usize,
}

fn parse_configs(args: &Vec<String>, argc: usize) -> Result<Config, ExitCode> {
  if args.len() < 2 {
    return Err(usage(1));
  }

  let (mut h, mut q, mut v, mut lf) = (false, false, false, false);
  let (mut c, mut f, mut l, mut ll) = (None, None, None, None);
  let mut s: i32 = -1;
  let mut fileidx = 0;

  let mut i = 1;
  while i < argc {
    let slice = args[i].as_str();
    match slice {
      "-h" | "--help" => {
        if h { return Err(usage(1)); }
        else { h = true; }
      }
      "-c" | "--coglib-dir" => {
        if c.is_some() { return Err(usage(1)); }
        else if i + 1 == argc { return Err(usage(3)); }
        i += 1;
        c = Some(args[i].clone());
      }
      "-f" | "--format" => {
        if f.is_some() { return Err(usage(1)); }
        else if i + 1 == argc { return Err(usage(3)); }
        i += 1;
        f = Some(args[i].clone());
      }
      "--list-formats" => {
        if lf { return Err(usage(1)); }
        else { lf = true; }
      }
      "-l" | "--log-file" => {
        if l.is_some() { return Err(usage(1)); }
        else if i + 1 == argc { return Err(usage(3)); }
        i += 1;
        l = Some(args[i].clone());
      }
      "-L" | "--load" => {
        if ll.is_some() { return Err(usage(1)); }
        else if i + 1 == argc { return Err(usage(3)); }
        i += 1;
        ll = Some(args[i].clone());
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
        match args[i].parse::<i32>() {
          Ok(i) => if i < 0 {
            println!("{}: index out of range", binary_name());
            return Err(try_help(2))
          } else { s = i },
          Err(_) => {
            println!("{}: '{}': invalid argument", binary_name(), slice);
            return Err(try_help(3))
          }
        }
      }
      _ => {
        fileidx = i;
        let s: usize = if s < 0 { 1 } else { s as usize };

        for int in i..(i + s.max(0) as usize) {
          if int >= argc {
            println!("{}: missing filename", binary_name());
            return Err(try_help(3))
          }
          if let Some(c) = args[int].bytes().next() {
            if c == b'-' {
              println!("{}: invalid option -- '{}'", binary_name(), &args[int].get(1..).unwrap());
              return Err(try_help(2))
            }
          }
        }
        break;
      }
    }
    i += 1;
  }

  let s: usize = if s < 0 { 1 } else { s as usize };

  Ok(Config{ help: h, coglib: c, format: f, logfile: l, load: ll, list_formats: lf, quiet: q, version: v, sources: s, fileidx })
}

fn binary_name() -> String { if let Some(n) = env::args().next() { n } else { "crank".to_string() } }

fn usage(code: u8) -> ExitCode {
  println!("usage: {} [-hqv] [--list-formats | -f FORMAT] [-l FILE] [-L FILE] [-s N] [file...] [arg...]", binary_name());
  try_help(code)
}

fn try_help(code: u8) -> ExitCode {
  println!("Try '{} --help' for more information.", binary_name());
  ExitCode::from(code)
}

fn help() -> ExitCode {
  usage(0);
  println!(" -h, --help            print this help message");
  println!(" -c, --coglib-dir DIR  use DIR as a secondary source directory");
  println!(" -f, --format FORMAT   use FORMAT as the load format, see '--load'");
  println!("     --list-formats    print a list of supported load formats");
  println!(" -l, --log-file FILE   enable token logging to FILE");
  println!(" -L, --load FILE       load cognition state from FILE (default format is JSON)");
  println!("                         for a list of supported formats, see '--list-formats'");
  println!(" -q, --quiet           don't show state information at program end");
  println!(" -s, --sources N       specify N source files to be composed (default is N=1)");
  println!(" -v, --version         print version information");
  ExitCode::SUCCESS
}

fn version() -> ExitCode {
  println!("cognition {VERSION}, written by Matthew Hinton and Preston Pan, MIT License 2024");
  ExitCode::SUCCESS
}

fn list_formats() -> ExitCode {
  println!("Currently supported data formats:");
  for fmt in DATA_FORMATS { println!("{fmt}"); }
  ExitCode::SUCCESS
}

fn load(filename: &String) -> Result<CognitionState, ExitCode> {
  let Ok(file) = std::fs::File::options().read(true).open(filename) else {
    return Err(ExitCode::from(4))
  };
  let mut bufreader = std::io::BufReader::new(file);
  let mut string = String::new();
  let _ = bufreader.read_to_string(&mut string);
  println!("loading... \"{}\"", string);

  let mut deserializer = serde_json::Deserializer::from_str(&string);
  match crate::serde::deserialize_cognition_state(&mut deserializer) {
    Ok(state) => Ok(state),
    Err(e) => {
      println!("{e}");
      Err(ExitCode::from(5))
    }
  }
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

  if let Some(ref fllibs) = state.fllibs {
    if fllibs.len() > 0 {
      print!("fllibs: ");
      let mut names = fllibs.keys();
      print!("'{}'", names.next().unwrap());
      for name in names {
        print!(", '{}'", name);
      }
      println!("\n");
    }
  }

  println!("Pool:");
  state.pool.print();
  println!("");

  if let Some(ref code) = state.exit_code {
    print!("Exit code: '");
    code.print_pretty();
    println!("'");
  } else {
    println!("Exit code: (none)");
  }
}
