#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
use std::process::ExitCode;
use std::env;
use std::fs::{self, File};
use std::io::stdin;

use cognition::*;

fn main() -> ExitCode {
  std::panic::set_hook(Box::new(|panic_info| {
    println!("Internal Cognition or FLLib Error: {}", panic_info);
    println!("This is a bug. If you can replicate this error, please help improve Cognition");
    println!("and its foreign language libraries by re-running crank with RUST_BACKTRACE=full,");
    println!("and posting the output in an issue on https://github.com/metacrank/cognition-rust");
  }));

  let args: Vec<String> = env::args().collect();
  let argc = args.len();

  let parse = parse_configs(&args, argc);
  let opts = match parse {
    Err(e) => return e,
    Ok(o) => o,
  };

  if opts.help { return help(); }
  if opts.usage { return usage(); }
  if opts.version { return version(); }
  if opts.list_formats { return list_formats(); }

  let sources = opts.sources as usize;
  let no_input = opts.fileidx == 0 && opts.sources != 0;
  let too_few_filenames = opts.fileidx + sources > argc;

  if no_input || too_few_filenames {
    println!("{}: missing filename", binary_name());
    return try_help(1)
  }

  let mut logfile = match log(opts.logfile.as_ref()) {
    Ok(f) => f,
    Err(e) => return e
  };

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
      for arg in args[(opts.fileidx + sources)..].iter() {
        state.args.push(Value::Word(Box::new(VWord{ str_word: arg.clone() })));
      }
      builtins::add_builtins(&mut state);
      state
    }
  };

  if state.parser.is_none() {
    state.parser = Some(Parser::new(None, None));
  }

  let mut idx = 0;
  'inputs: loop {
    if idx >= sources { break 'inputs }
    // Read code from file
    let file = &args[opts.fileidx + idx];
    let (source, filename, stdin) = if *file == "-" {
      let Some(Ok(s)) = stdin().lines().next() else { break 'inputs };
      (s, None, true)
    } else {
      let mut filename = None;
      let mut fs_result = fs::read_to_string(file);
      if let Err(_) = fs_result {
        if let Some(ref dir) = opts.coglib {
          let f = format!("{dir}/{file}");
          fs_result = fs::read_to_string(&f);
          filename = Some(f);
        } else if let Ok(dir) = env::var("COGLIB_DIR") {
          let f = format!("{dir}/{file}");
          fs_result = fs::read_to_string(&f);
          filename = Some(f);
        }
      }
      if let Err(e) = fs_result {
        println!("{}: could not open file for reading: {file}: {e}", binary_name());
        return ExitCode::from(4);
      }
      if filename.is_none() { filename = Some(state.string_copy(file)) }
      (fs_result.unwrap(), filename, false)
    };

    let mut parser = state.parser.take().unwrap();
    if let Some(s) = parser.source() { state.pool.add_string(s) }
    parser.reset(source, filename);
    state.parser = Some(parser);

    // Parse and eval loop
    loop {
      let w = state.parser_get_next();
      match w {
        Some(v) => {
          if let Some(f) = &mut logfile { v.fprint(f, "\n", false) }
          state = state.eval(v, None)
        },
        None => break,
      }
      if state.control.is_return() {
        if stdin { idx += 1 }
        break
      }
      if state.exited { break 'inputs }
    }
    if !stdin { idx += 1 }
  }

  let end = opts.end.unwrap_or_else(|| End::with(true));
  if !opts.quiet { print_end(&state, end); }
  if let Some(ref save_fn) = save_fn {
    return cogsave(&state, opts.save.as_ref().unwrap(), *save_fn)
  }

  // Ensure foreign libraries stay loaded when
  // dropping custom objects in cognition state
  let fllibs = state.fllibs.take();

  drop(state);

  ExitCode::SUCCESS
}

struct Config {
  help: bool,
  coglib: Option<String>,
  end: Option<End>,
  format: Option<String>,
  list_formats: bool,
  fllibs: Option<String>,
  suppress_fllibs: bool,
  logfile: Option<String>,
  load: Option<String>,
  quiet: bool,
  sources: i32,
  save: Option<String>,
  save_format: Option<String>,
  version: bool,
  usage: bool,
  fileidx: usize
}

impl Config {
  pub fn default() -> Self {
    Self {
      help: false,
      coglib: None,
      end: None,
      format: None,
      list_formats: false,
      fllibs: None,
      suppress_fllibs: false,
      logfile: None,
      load: None,
      quiet: false,
      sources: -1,
      save: None,
      save_format: None,
      version: false,
      usage: false,
      fileidx: 0
    }
  }
}

struct End {
  stack: bool,
  estack: bool,
  faliases: bool,
  parser: bool,
  cranks: bool,
  math: bool,
  fllibs: bool,
  pool: bool,
  customs: bool,
  exit: bool
}

impl End {
  pub fn with(b: bool) -> Self {
    Self {
      stack: b,
      estack: b,
      faliases: b,
      parser: b,
      cranks: b,
      math: b,
      fllibs: b,
      pool: b,
      customs: b,
      exit: b
    }
  }
}

fn parse_sources(args: &Vec<String>, argc: usize, i: usize, sources: i32) -> Result<(usize, i32), ExitCode> {
  if sources >= 0 { return Err(usage_help(1)) }
  else if i + 1 == argc { return Err(usage_help(3)) }
  match args[i + 1].parse::<i32>() {
    Ok(int) => if int < 0 {
      println!("{}: sources: index out of range", binary_name());
      Err(try_help(2))
    } else { Ok((i + 1, int)) },
    Err(_) => {
      println!("{}: sources: invalid argument", binary_name());
      Err(try_help(3))
    }
  }
}

fn parse_end(args: &Vec<String>, argc: usize, i: usize, end: bool) -> Result<(usize, Option<End>), ExitCode> {
  if end { return Err(usage_help(1)) }
  else if i + 1 == argc { return Err(usage_help(3)) }
  let mut new_end = End::with(false);
  for c in args[i + 1].chars() {
    match c {
      's' => new_end.stack = true,
      'e' => new_end.estack = true,
      'f' => new_end.faliases = true,
      'p' => new_end.parser = true,
      'c' => new_end.cranks = true,
      'm' => new_end.math = true,
      'F' => new_end.fllibs = true,
      'P' => new_end.pool = true,
      'C' => new_end.customs = true,
      'E' => new_end.exit = true,
      a => {
        println!("{}: end: invalid option -- '{}'", binary_name(), a);
        return Err(try_help(2))
      }
    }
  }
  Ok((i + 1, Some(new_end)))
}

macro_rules! define_config_parsers {
  ($set_bool:tt,$set_str:tt,$args:ident,$argc:ident) => {
    let $set_bool = |boolval| if boolval { Err(usage_help(1)) } else { Ok(true) };
    let $set_str  = |i: usize, strval: &Option<String>|
    if strval.as_ref().is_some() { Err(usage_help(1)) }
    else if i + 1 == $argc { Err(usage_help(3)) }
    else { Ok((i + 1, Some($args[i + 1].clone()))) };
  }
}

fn parse_short(short: &str, args: &Vec<String>, argc: usize, iptr: &mut usize, config: &mut Config) -> Result<bool, ExitCode> {
  define_config_parsers!{ set_bool, set_str, args, argc }
  let mut i = *iptr;
  let mut char_iter = short.chars().rev();
  let last = char_iter.next().unwrap();
  // check if short args are valid
  for c in short[..short.len()-1].chars() {
    if "hquv".chars().all(|ch| ch != c) {
      println!("{}: invalid option -- '{}'", binary_name(), c);
      return Err(try_help(2))
    }
  }
  // only last option can take arguments
  match last {
    'c' => (i, config.coglib)  = set_str(i, &config.coglib)?,
    'f' => (i, config.format)  = set_str(i, &config.format)?,
    'F' => (i, config.fllibs)  = set_str(i, &config.fllibs)?,
    'l' => (i, config.logfile) = set_str(i, &config.logfile)?,
    'L' => (i, config.load)    = set_str(i, &config.load)?,
    'S' => (i, config.save)    = set_str(i, &config.save)?,
    'e' => (i, config.end)     = parse_end(args, argc, i, config.end.is_some())?,
    's' => (i, config.sources) = parse_sources(args, argc, i, config.sources)?,
    _ => char_iter = short.chars().rev()
  }
  *iptr = i;

  for c in char_iter {
    match c {
      'h' => config.help    = set_bool(config.help)?,
      'q' => config.quiet   = set_bool(config.quiet)?,
      'u' => config.usage   = set_bool(config.usage)?,
      'v' => config.version = set_bool(config.version)?,
      _ => {
        println!("{}: invalid option -- '{}'", binary_name(), c);
        return Err(try_help(2))
      }
    }
  }
  Ok(false)
}

fn parse_other(args: &Vec<String>, argc: usize, iptr: &mut usize, config: &mut Config) -> Result<bool, ExitCode> {
  define_config_parsers!{ set_bool, set_str, args, argc }
  let slice = &args[*iptr];

  if let Some(c) = slice.bytes().next() {
    if c == b'-' && slice.len() > 1 { return parse_short(&slice[1..], args, argc, iptr, config) }
  }

  let s = if config.sources < 0 { 1 } else { config.sources as usize };
  config.fileidx = *iptr;
  if config.fileidx + s > argc {
    println!("{}: missing filename", binary_name());
    return Err(try_help(1))
  }
  Ok(true)
}

fn parse_configs(args: &Vec<String>, argc: usize) -> Result<Config, ExitCode> {
  if args.len() < 2 { return Err(usage_help(1)) }
  let mut config = Config::default();
  define_config_parsers!{ set_bool, set_str, args, argc }

  let mut i = 1;
  while i < argc {
    let slice = args[i].as_str();
    match slice {
      "--help"            => config.help             = set_bool(config.help)?,
      "--list-formats"    => config.list_formats     = set_bool(config.list_formats)?,
      "--suppress-fllibs" => config.suppress_fllibs  = set_bool(config.suppress_fllibs)?,
      "--quiet"           => config.quiet            = set_bool(config.quiet)?,
      "--usage"           => config.usage            = set_bool(config.usage)?,
      "--version"         => config.version          = set_bool(config.version)?,

      "--coglib-dir"      => (i, config.coglib)      = set_str(i, &config.coglib)?,
      "--format"          => (i, config.format)      = set_str(i, &config.format)?,
      "--fllibs"          => (i, config.fllibs)      = set_str(i, &config.fllibs)?,
      "--log-file"        => (i, config.logfile)     = set_str(i, &config.logfile)?,
      "--load"            => (i, config.load)        = set_str(i, &config.load)?,
      "--save"            => (i, config.save)        = set_str(i, &config.save)?,
      "--save-format"     => (i, config.save_format) = set_str(i, &config.save_format)?,
      "--end"             => (i, config.end)         = parse_end(args, argc, i, config.end.is_some())?,
      "--sources"         => (i, config.sources)     = parse_sources(args, argc, i, config.sources)?,

      _ => if parse_other(args, argc, &mut i, &mut config)? { break }
    }
    i += 1
  }

  if config.load.is_none() && (config.format.is_some() || config.fllibs.is_some() || config.suppress_fllibs) {
    return Err(usage_help(1))
  }
  if config.load.is_some() && config.list_formats { return Err(usage_help(1)) }
  if config.save.is_none() && config.save_format.is_some() { return Err(usage_help(1)) }
  if config.sources < 0 { config.sources = 1 }

  Ok(config)
}

fn binary_name() -> String { if let Some(n) = env::args().next() { n } else { "crank".to_string() } }

const OPTIONS: &str = "[-hquv] [-e [sefpcmFPCE]] [-l FILE] [-L FILE [-f FORMAT] [-F FILE] [--suppress-fllibs] -S FILE [--save-format FORMAT] | --list-formats]";

fn usage() -> ExitCode {
  println!("Usage: {} {OPTIONS} [-s N] [file...] [arg...]", binary_name());
  ExitCode::SUCCESS
}

fn usage_help(code: u8) -> ExitCode {
  usage();
  try_help(code)
}

fn try_help(code: u8) -> ExitCode {
  println!("Try '{} --help' for more information.", binary_name());
  ExitCode::from(code)
}

fn help() -> ExitCode {
  println!("Usage: {} [options] [file...] [arg...]", binary_name());
  println!("The [file] option can be either a filename or '-' (read from standard input)");
  println!("");
  println!("Options:");
  println!(" -h, --help                print this help message");
  println!(" -c, --coglib-dir DIR      use DIR as a secondary source directory");
  println!(" -e, --end [sefpcmFPCE]    select information to display at program end");
  println!(" -f, --format FORMAT       use FORMAT as the load format (see '--load')");
  println!("                             FORMAT can optionally be a comma-separated pair: '-f|--format LOAD_FORMAT,FLLIBS_FORMAT'");
  println!("                             to specify a different format for an optional fllib description file (see '--fllibs')");
  println!("     --list-formats        print a list of supported load formats");
  println!(" -F, --fllibs FILE         supplement cognition load file with an fllib description file");
  println!("                             (only used in combination with '--load')");
  println!("     --suppress-fllibs     suppress automatic interpretation of fllibs with '--load'");
  println!(" -l, --log-file FILE       enable token logging to FILE");
  println!(" -L, --load FILE           load cognition state from FILE, attempting to infer format from file extension");
  println!("                             for a list of supported formats and extensions, see '--list-formats'");
  println!(" -q, --quiet               don't show state information at program end");
  println!(" -s, --sources N           specify N source files to be composed (default is N=1)");
  println!("                             not compatible with '-'");
  println!(" -S, --save FILE           save cognition state to FILE on program exit");
  println!("     --save-format FORMAT  explicitly set save format");
  println!(" -u, --usage               print usage information");
  println!(" -v, --version             print version information");
  ExitCode::SUCCESS
}

fn version() -> ExitCode {
  println!("cognition {VERSION}, written by Matthew Hinton and Preston Pan, GNU GPLv3+ License 2024");
  ExitCode::SUCCESS
}

fn list_formats() -> ExitCode {
  println!("Currently supported data formats:");
  for (k, e, _, _, _, _, _, _, _) in DATA_FORMATS { println!("{k} \"{e}\""); }
  ExitCode::SUCCESS
}

fn log(logfile: Option<&String>) -> Result<Option<File>, ExitCode> {
  if let Some(ref s) = logfile {
    match File::options().write(true).truncate(true).create(true).open(s) {
      Ok(f) => Ok(Some(f)),
      Err(e) => {
        println!("{}: could not open logfile: {}: {e}", binary_name(), s);
        Err(ExitCode::from(4))
      }
    }
  } else { Ok(None) }
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

fn print_end(state: &CognitionState, e: End) {
  let cur = state.current_ref();

  if e.stack {
    println!("\nStack at end:");
    for v in cur.stack.iter() { v.print("\n"); }
  }
  if e.estack {
    println!("\nError stack:");
    if let Some(errors) = &cur.err_stack {
      for verror in errors.iter() { verror.print("\n"); }
    }
  }
  if e.faliases {
    print!("\nFaliases:");
    if cur.faliases.as_ref().map_or(false, |f| f.len() > 0) {
      for alias in cur.faliases.as_ref().unwrap().iter() {
        print!(" '");
        alias.print_pretty();
        print!("'");
      }
      println!("");
    } else {
      println!("(none)");
    }
  }
  if e.parser {
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
    if cur.sflag { println!("' (whitelist)"); }
    else         { println!("' (blacklist)"); }
  }

  if e.cranks {
    if let Some(cranks) = &cur.cranks {
      println!("");
      if cranks.iter().all(|cr| cr.base == 0) {
        println!("crank 0");
      } else {
        print!("cranks:");
        for (i, crank) in cranks.iter().enumerate() {
          if crank.base != 0 { print!(" {i}:({},{})", crank.modulo, crank.base); }
        }
        println!("");
      }
    } else {
      println!("\ncrank 0");
    }
  }

  if e.math {
    let math = cur.math.as_ref();
    println!("");
    println!("Math:");
    println!("base: {}", math.map_or(0, |m| m.base()));
    print!("digits: ");
    if math.map_or(0, |m| m.get_digits().len()) > 0 {
      for d in math.unwrap().get_digits() {
        print!("{d}");
      }
      println!("");
    } else {
      println!("(none)")
    }
    print!("negc: ");
    match math.map_or(None, |m| m.get_negc()) {
      Some(c) => println!("'{}\u{00A0}'", c),
      None => println!("(none)"),
    }
    print!("radix: ");
    match math.map_or(None, |m| m.get_radix()) {
      Some(c) => println!("'{c}'"),
      None => println!("(none)"),
    }
    print!("delim: ");
    match math.map_or(None, |m| m.get_delim()) {
      Some(c) => println!("'{c}'"),
      None => println!("(none)"),
    }
    print!("meta-radix: ");
    match math.map_or(None, |m| m.get_meta_radix()) {
      Some(c) => println!("'{c}'"),
      None => println!("(none)"),
    }
    print!("meta-delim: ");
    match math.map_or(None, |m| m.get_meta_delim()) {
      Some(c) => println!("'{c}'"),
      None => println!("(none)"),
    }
  }

  if e.fllibs {
    if let Some(ref fllibs) = state.fllibs {
      if fllibs.len() > 0 {
        println!("\nFLLibs:");
        for (name, lib) in fllibs.iter() {
          println!("'{}': {}", name, lib.library.lib_path);
        }
      }
    }
  }

  if e.pool {
    println!("\nPool:");
    state.pool.print();
  }

  if e.customs {
    if state.pool.has_customs() {
      println!("\nCustom Pools:");
      state.pool.print_custom_pools();
    }
  }

  if e.exit {
    println!("");
    if let Some(ref code) = state.exit_code {
      print!("Exit code: '");
      code.print_pretty();
      println!("'");
    } else {
      println!("Exit code: (none)");
    }
  }
}
