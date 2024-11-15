#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
use std::process::ExitCode;
use std::env;
use std::fs::{self, File};

use cognition::*;
use cognition::macros::*;

const VERSION: &'static str = "0.2.2 alpha";

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

  let mut logfile = if let Some(ref s) = opts.logfile {
    match File::options().write(true).truncate(true).create(true).open(s) {
      Ok(f) => Some(f),
      Err(e) => {
        println!("{}: could not open logfile: {}: {e}", binary_name(), opts.logfile.as_ref().unwrap());
        return ExitCode::from(4);
      }
    }
  } else { None };

  let save_fn = if let Some(ref savefile) = opts.save {
    match get_save_fn(savefile, opts.save_format.as_ref()) {
      Ok(func) => Some(func),
      Err(e) => return e
    }
  } else { None };

  // Initialize state
  let mut state = match opts.load {
    Some(ref loadfile) => {
      match load(loadfile, opts.format.as_ref(), opts.fllibs.as_ref(), opts.suppress_fllibs) {
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
  if let Some(ref save_fn) = save_fn {
    return cogsave(&state, opts.save.as_ref().unwrap(), *save_fn)
  }

  ExitCode::SUCCESS
}

struct Config {
  help: bool,
  coglib: Option<String>,
  format: Option<String>,
  list_formats: bool,
  fllibs: Option<String>,
  suppress_fllibs: bool,
  logfile: Option<String>,
  load: Option<String>,
  quiet: bool,
  sources: usize,
  save: Option<String>,
  save_format: Option<String>,
  version: bool,
  fileidx: usize,
}

fn parse_configs(args: &Vec<String>, argc: usize) -> Result<Config, ExitCode> {
  if args.len() < 2 {
    return Err(usage(1));
  }

  let (mut help, mut quiet, mut version, mut list_formats, mut suppress_fllibs) = (false, false, false, false, false);
  let (mut coglib, mut format, mut fllibs, mut logfile, mut load, mut save, mut save_format) = (None, None, None, None, None, None, None);
  let mut sources: i32 = -1;
  let mut fileidx = 0;

  let mut i = 1;
  while i < argc {
    let slice = args[i].as_str();
    match slice {
      "-h" | "--help" => {
        if help { return Err(usage(1)); }
        else { help = true; }
      }
      "-c" | "--coglib-dir" => {
        if coglib.is_some() { return Err(usage(1)); }
        else if i + 1 == argc { return Err(usage(3)); }
        i += 1;
        coglib = Some(args[i].clone());
      }
      "-f" | "--format" => {
        if format.is_some() { return Err(usage(1)); }
        else if i + 1 == argc { return Err(usage(3)); }
        i += 1;
        format = Some(args[i].clone());
      }
      "--list-formats" => {
        if list_formats { return Err(usage(1)); }
        else { list_formats = true; }
      }
      "-F" | "--fllibs" => {
        if fllibs.is_some() { return Err(usage(1)); }
        else if i + 1 == argc { return Err(usage(3)); }
        i += 1;
        fllibs = Some(args[i].clone());
      }
      "--suppress_fllibs" => {
        if suppress_fllibs { return Err(usage(1)); }
        else { suppress_fllibs = true; }
      }
      "-l" | "--log-file" => {
        if logfile.is_some() { return Err(usage(1)); }
        else if i + 1 == argc { return Err(usage(3)); }
        i += 1;
        logfile = Some(args[i].clone());
      }
      "-L" | "--load" => {
        if load.is_some() { return Err(usage(1)); }
        else if i + 1 == argc { return Err(usage(3)); }
        i += 1;
        load = Some(args[i].clone());
      }
      "-q" | "--quit" => {
        if quiet { return Err(usage(1)); }
        else { quiet = true; }
      }
      "-v" | "--version" => {
        if version { return Err(usage(1)); }
        else { version = true; }
      }
      "-s" | "--sources" => {
        if sources >= 0 {
          return Err(usage(1));
        } else if i + 1 == argc {
          return Err(usage(3));
        }
        i += 1;
        match args[i].parse::<i32>() {
          Ok(i) => if i < 0 {
            println!("{}: index out of range", binary_name());
            return Err(try_help(2))
          } else { sources = i },
          Err(_) => {
            println!("{}: '{}': invalid argument", binary_name(), slice);
            return Err(try_help(3))
          }
        }
      }
      "-S" | "--save" => {
        if save.is_some() { return Err(usage(1)); }
        else if i + 1 == argc { return Err(usage(3)); }
        i += 1;
        save = Some(args[i].clone());
      }
      "--save-format" => {
        if save_format.is_some() { return Err(usage(1)); }
        else if i + 1 == argc { return Err(usage(3)); }
        i += 1;
        save_format = Some(args[i].clone());
      }
      _ => {
        fileidx = i;
        let s: usize = if sources < 0 { 1 } else { sources as usize };

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

  if load.is_none() && (format.is_some() || fllibs.is_some() || suppress_fllibs) {
    return Err(usage(1))
  }
  if load.is_some() && list_formats {
    return Err(usage(1))
  }
  if save.is_none() && save_format.is_some() {
    return Err(usage(1))
  }

  let sources: usize = if sources < 0 { 1 } else { sources as usize };

  Ok(Config{ help, coglib, format, list_formats, fllibs, suppress_fllibs, logfile, load, quiet, sources, save, save_format, version, fileidx })
}

fn binary_name() -> String { if let Some(n) = env::args().next() { n } else { "crank".to_string() } }

const OPTIONS: &str = "[-hqv] [-l FILE] [-L FILE [-f FORMAT] [-F FILE] [--suppress-fllibs] -S FILE [--save-format FORMAT] | --list-formats] [-s N]";

fn usage(code: u8) -> ExitCode {
  println!("Usage: {} {OPTIONS} [file...] [arg...]", binary_name());
  try_help(code)
}

fn try_help(code: u8) -> ExitCode {
  println!("Try '{} --help' for more information.", binary_name());
  ExitCode::from(code)
}

fn help() -> ExitCode {
  println!("Usage: {} [options] [file...] [arg...]", binary_name());
  println!("");
  println!("Options: {OPTIONS}");
  println!(" -h, --help                print this help message");
  println!(" -c, --coglib-dir DIR      use DIR as a secondary source directory");
  println!(" -f, --format FORMATS      use FORMAT as the load format, see '--load'");
  println!("                             FORMAT can optionally be a comma-separated pair as in:");
  println!("                               '-f|--format LOAD_FORMAT,FLLIBS_FORMAT'");
  println!("                             to specify a different format for each file");
  println!("     --list-formats        print a list of supported load formats");
  println!(" -F, --fllibs FILE         supplement cognition load file with fllibs (typically generated with describe-fllibs)");
  println!("                             (only used in combination with '--load')");
  println!("     --suppress-fllibs     suppress automatic interpretation of fllibs with '--load'");
  println!(" -l, --log-file FILE       enable token logging to FILE");
  println!(" -L, --load FILE           load cognition state from FILE, attempting to infer format from file extension");
  println!("                             for a list of supported formats and extensions, see '--list-formats'");
  println!(" -q, --quiet               don't show state information at program end");
  println!(" -s, --sources N           specify N source files to be composed (default is N=1)");
  println!(" -S, --save FILE           save cognition state to FILE on program exit");
  println!("     --save-format FORMAT  explicitly set save format");
  println!(" -v, --version             print version information");
  ExitCode::SUCCESS
}

fn version() -> ExitCode {
  println!("cognition {VERSION}, written by Matthew Hinton and Preston Pan, MIT License 2024");
  ExitCode::SUCCESS
}

fn list_formats() -> ExitCode {
  println!("Currently supported data formats:");
  for (k, e, _, _, _, _, _) in DATA_FORMATS { println!("{k} \"{e}\""); }
  ExitCode::SUCCESS
}

fn load_result(result: CognitionDeserializeResult) -> Result<CognitionState, ExitCode> {
  match result {
    Ok(state) => Ok(state),
    Err(e) => {
      println!("{}: load: {}", binary_name(), e.1);
      Err(ExitCode::from(5))
    }
  }
}

fn load(loadfile: &String, format: Option<&String>, fllibs: Option<&String>, suppress_fllibs: bool) -> Result<CognitionState, ExitCode> {

  let formats = match format {
    Some(fmt) => {
      let v: Vec<&str> = fmt.split(',').collect();
      if v.len() > 2 || v.len() == 2 && fllibs.is_none() {
        println!("{}: incorrect number of formats: {}", binary_name(), v.len());
        return Err(try_help(3))
      }
      Some(v)
    },
    None => None,
  };

  let mut state = cognition::serde::cogstate_init();
  let mut ignore_fllibs = false;

  if let Some(fllibs) = fllibs {
    let fllibs_fmt = match formats {
      Some(ref fmts) => match fmts.last() { Some(f) => Some(*f), None => None },
      None => None
    };

    let file = match fs::read_to_string(fllibs) {
      Ok(f) => f,
      Err(e) => {
        println!("{}: could not open file for reading: {}: {}", binary_name(), fllibs, e);
        return Err(ExitCode::from(4))
      }
    };

    let deserialize_fn = get_from_data_formats!(fllibs, fllibs_fmt, 3, MAIN);
    state = load_result(deserialize_fn(&file, state))?;

    ignore_fllibs = true;
  }

  let load_fmt = match formats {
    Some(ref fmts) => match fmts.first() { Some(f) => Some(*f), None => None },
    None => None
  };

  let file = match fs::read_to_string(loadfile) {
    Ok(f) => f,
    Err(e) => {
      println!("{}: could not open file for reading: {}: {}", binary_name(), loadfile, e);
      return Err(ExitCode::from(4))
    }
  };

  let deserialize_fn = get_from_data_formats!(loadfile, load_fmt, 2, MAIN);
  load_result(deserialize_fn(&file, ignore_fllibs || suppress_fllibs, state))
}

fn get_save_fn(savefile: &String, format: Option<&String>) -> Result<CogStateSerializeFn, ExitCode> {
  Ok(get_from_data_formats!(savefile, format, 4, MAIN))
}

fn cogsave(state: &CognitionState, savefile: &String, serialize_fn: CogStateSerializeFn) -> ExitCode {
  let mut file = match File::create(savefile) {
    Ok(f) => f,
    Err(e) => {
      println!("{}: could not open save file: {}: {}", binary_name(), savefile, e);
      return ExitCode::from(4)
    }
  };
  match serialize_fn(state, &mut file) {
    Ok(_) => ExitCode::SUCCESS,
    Err(e) => {
      println!("{}: save: {e}", binary_name());
      return ExitCode::from(5)
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
