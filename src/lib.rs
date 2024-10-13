#![allow(dead_code)]
#![allow(unused_macros)]

#[macro_use]
pub mod macros;
pub mod pool;
pub mod builtins;

use crate::pool::Pool;
use crate::macros::*;

use std::collections::HashMap;
use std::default::Default;
use std::io::Write;
use std::io::stdout;
use std::str::Chars;

pub type CognitionFunction = fn(CognitionState, &Value) -> CognitionState;
pub type Stack = Vec<Value>;
pub type Cranks = Vec<Crank>;
pub type Strings = Vec<String>;
pub type Faliases = Strings;
pub type WordTable = HashMap<String, Value>;


pub trait Pretty {
  fn print_pretty(&self);
  fn fprint_pretty(&self, f: &mut dyn Write);
}

impl Pretty for String {
  fn print_pretty(&self) {
    self.fprint_pretty(&mut stdout());
  }
  fn fprint_pretty(&self, f: &mut dyn Write) {
    fwrite_check_pretty!(f, self.as_bytes());
  }
}

impl Pretty for [u8] {
  fn print_pretty(&self) {
    self.fprint_pretty(&mut stdout());
  }
  fn fprint_pretty(&self, f: &mut dyn Write) {
    fwrite_check_pretty!(f, self);
  }
}

pub trait Custom {
  fn printfunc(&self, f: &mut dyn Write);
  // implemented as Box::new(self.clone()) in classes that implement Clone
  fn copyfunc(&self) -> Box<dyn Custom>;
}

/// Example of a rust library that would utilise Custom
/// If the type has to be unsafely allocated, or is a C
/// library, implement the Drop trait to free memory
struct ExampleCustom {
  i: String,
}

impl Clone for ExampleCustom {
  fn clone(&self) -> Self {
    ExampleCustom{ i: self.i.clone() }
  }
}

impl Custom for ExampleCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check_pretty!(f, self.i.as_bytes());
  }
  fn copyfunc(&self) -> Box<dyn Custom> {
    Box::new(self.clone())
  }
}
/// End of example
///
/// Real useful Custom type
pub struct Void {}
impl Custom for Void {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check_pretty!(f, b"(void)");
  }
  fn copyfunc(&self) -> Box<dyn Custom> {
    Box::new(Void{})
  }
}

pub struct Crank {
  pub modulo: i32,
  pub base: i32,
}

pub struct Container {
  pub stack: Stack,
  pub err_stack: Option<Stack>,
  pub cranks: Option<Cranks>,
  pub faliases: Option<Faliases>,
  pub delims: Option<String>,
  pub ignored: Option<String>,
  pub singlets: Option<String>,
  pub dflag: bool,
  pub iflag: bool,
  pub sflag: bool,

  word_table: Option<WordTable>,
}

impl Default for Container {
  fn default() -> Container {
    Container {
      stack: Stack::new(),
      err_stack: None,
      cranks: None,
      faliases: None,
      delims: None,
      ignored: None,
      singlets: None,
      dflag: false,
      iflag: true,
      sflag: true,

      word_table: None,
    }
  }
}

impl Container {
  pub fn with_stack(stack: Stack) -> Container {
    Container{ stack,..Default::default() }
  }
  pub fn with_capacity(capacity: usize) -> Container {
    let stack = Stack::with_capacity(capacity);
    Self::with_stack(stack)
  }

  fn inc_crank(&mut self) {
    let Some(cranks) = &mut self.cranks else { return };
    for crank in cranks {
      crank.modulo += 1;
      if crank.modulo >= crank.base {
        crank.modulo = 0;
      }
    }
  }
  fn dec_crank(&mut self) {
    let Some(cranks) = &mut self.cranks else { return };
    for crank in cranks {
      crank.modulo -= 1;
      if crank.modulo < 0 {
        crank.modulo = crank.base - 1;
      }
    }
  }
  pub fn default_faliases() -> Option<Faliases> {
    let mut f = Faliases::with_capacity(2);
    f.push(String::from("f"));
    f.push(String::from("ing"));
    Some(f)
  }
  //TODO: needs work
  pub fn add_word(&mut self, v: Value, name: &'static str) {
    match &mut self.word_table {
      Some(wt) => { wt.insert(String::from(name), v); },
      None => {
        let mut wt = WordTable::with_capacity(DEFAULT_WORD_TABLE_SIZE);
        wt.insert(String::from(name), v);
        self.word_table = Some(wt);
      },
    }
  }
}

pub struct VWord {
  str_word: String,
}
pub struct VStack {
  pub container: Container,
}
pub struct VMacro {
  macro_stack: Stack,
}
pub struct VError {
  error: String,
  str_word: Option<String>,
}
pub struct VCustom {
  custom: Box<dyn Custom>,
}
pub struct VFLLib {
  fllib: CognitionFunction,
  str_word: Option<String>,
}

impl VWord {
  pub fn with_string(str_word: String) -> VWord {
    VWord{ str_word }
  }
  pub fn with_capacity(capacity: usize) -> VWord {
    let str_word = String::with_capacity(capacity);
    VWord{ str_word }
  }
}
impl VStack {
  pub fn with_container(container: Container) -> VStack {
    VStack{ container }
  }
  pub fn with_capacity(capacity: usize) -> VStack {
    let stack = Stack::with_capacity(capacity);
    let container = Container::with_stack(stack);
    VStack{ container }
  }
}
impl VMacro {
  pub fn with_macro(macro_stack: Stack) -> VMacro {
    VMacro{ macro_stack }
  }
  pub fn with_capacity(capacity: usize) -> VMacro {
    let macro_stack = Stack::with_capacity(capacity);
    VMacro{ macro_stack }
  }
}
impl VError {
  pub fn with_strings(error: String, str_word: String) -> VError {
    VError{ error, str_word: Some(str_word) }
  }
  pub fn with_error(error: String) -> VError {
    VError{ error, str_word: None }
  }
  pub fn with_capacity(capacity: usize) -> VError {
    Self::with_error(String::with_capacity(capacity))
  }
}
impl VCustom {
  pub fn with_custom(custom: Box<dyn Custom>) -> VCustom {
    VCustom{ custom }
  }
  pub fn with_void() -> VCustom {
    Self::with_custom(Box::new(Void{}))
  }
}
impl VFLLib {
  pub fn with_fn(fllib: CognitionFunction) -> VFLLib {
    VFLLib{ fllib, str_word: None }
  }
  pub fn with_nop() -> VFLLib {
    Self::with_fn(builtins::cog_nop)
  }
}

pub enum Value {
  Word(Box<VWord>),
  Stack(Box<VStack>),
  Macro(Box<VMacro>),
  Error(Box<VError>),
  Custom(Box<VCustom>),
  FLLib(Box<VFLLib>),
}

impl Value {
  pub fn print(&self, end: &'static str) {
    self.fprint(&mut stdout(), end);
  }
  pub fn fprint(&self, f: &mut dyn Write, end: &'static str) {
    match self {
      Self::Word(vword) => {
        fwrite_check_pretty!(f, b"'");
        vword.str_word.fprint_pretty(f);
        fwrite_check_pretty!(f, b"'");
      }
      Self::Stack(vstack) => {
        fwrite_check_pretty!(f, b"[ ");
        for v in vstack.container.stack.iter() { v.fprint(f, " "); }
        fwrite_check_pretty!(f, b"]");
      }
      Self::Macro(vmacro) => {
        fwrite_check_pretty!(f, b"( ");
        for v in vmacro.macro_stack.iter() { v.fprint(f, " "); }
        fwrite_check_pretty!(f, b")");
      }
      Self::Error(verror) => {
        match verror.str_word {
          Some(ref word) => {
            fwrite_check_pretty!(f, b"'");
            word.fprint_pretty(f);
            fwrite_check_pretty!(f, b"'");
          }
          None => {
            fwrite_check_pretty!(f, b"(none)");
          }
        }
        fwrite_check_pretty!(f, RED);
        verror.error.fprint_pretty(f);
        fwrite_check_pretty!(f, COLOR_RESET);
      }
      Self::Custom(vcustom) => {
        vcustom.custom.printfunc(f);
      }
      Self::FLLib(vfllib) => {
        match &vfllib.str_word {
          Some(s) => {
            s.fprint_pretty(f);
          },
          None => {
            fwrite_check_pretty!(f, HBLK);
            fwrite_check_pretty!(f, b"FLLIB");
            fwrite_check_pretty!(f, COLOR_RESET);
          },
        }
      }
    }
    fwrite_check!(f, end.as_bytes());
  }
  pub fn metastack_container(&mut self) -> &mut Container {
    let Value::Stack(vstack) = self else { panic!("Bad value on metastack") };
    &mut vstack.container
  }
  pub fn metastack_container_ref(&self) -> &Container {
    let Value::Stack(vstack) = self else { panic!("Bad value on metastack") };
    &vstack.container
  }
  pub fn expect_word(&self, e: &'static str) -> &String {
    let Value::Word(vword) = self else { panic!("{e}") };
    &vword.str_word
  }
}

pub struct Parser<'a> {
  source: Chars<'a>,
  c: Option<char>,
}

impl Parser<'_> {
  pub fn new(source: &String) -> Parser {
    let mut source = source.chars();
    let c = source.next();
    Parser{ source, c }
  }
  pub fn next(&mut self) {
    self.c = self.source.next();
  }

  fn skip_ignored(&mut self, state: &CognitionState) -> bool {
    let mut skipped = false;
    while let Some(c) = self.c {
      if !state.isignore(c) { break };
      skipped = true;
      self.next();
    }
    skipped
  }

  fn parse_word(&mut self, skipped: bool, state: &mut CognitionState) -> Option<Value> {
    let Some(c) = self.c else { return None };
    let mut v = state.pool.get_vword(DEFAULT_STRING_LENGTH);
    let Value::Word(vword) = &mut v else { panic!("Pool::get_vword() failed") };
    if state.issinglet(c) {
      vword.str_word.push(c);
      self.next();
      return Some(v);
    }
    if !skipped {
      vword.str_word.push(c);
      self.next();
    }
    while let Some(c) = self.c {
      if state.isdelim(c) { break };
      vword.str_word.push(c);
      self.next();
      if state.issinglet(c) { return Some(v) };
    }
    Some(v)
  }

  // returns just the next character without delim/ignore/singlet behaviour
  #[allow(dead_code)]
  pub fn get_next_char(&mut self, state: &mut CognitionState) -> Option<Value> {
    let ch = self.c;
    self.next();
    match ch {
      None => None,
      Some(c) => {
        let mut v = state.pool.get_vword(c.len_utf8());
        let Value::Word(vword) = &mut v else { panic!("Pool::get_vword() failed") };
        vword.str_word.push(c);
        self.next();
        Some(v)
      },
    }
  }
  /// Parse next token and return it as a word value option
  pub fn get_next(&mut self, state: &mut CognitionState) -> Option<Value> {
    let skipped = self.skip_ignored(&state);
    self.parse_word(skipped, state)
  }
}

pub struct CognitionState {
  pub stack: Stack,
  //pub parser: Option<Parser>,
  pub exited: bool,
  pub exit_code: Option<String>,
  //root: &str,
  pub args: Strings,
  pub pool: Pool,
  pub i: i32, // to keep rust-analyser happy for the moment
}

impl CognitionState {
  pub fn new(stack: Stack) -> Self {
    Self{ stack,
          exited: false,
          exit_code: None,
          args: Strings::new(),
          pool: Pool::new(),
          i: 0 }
  }

  pub fn eval_error(&mut self, e: &'static str, w: Option<&Value>) {
    let mut verror = self.pool.get_verror(e.len());
    let Value::Error(error) = &mut verror else { panic!("Pool::get_verror() failed") };
    error.error.push_str(e);
    error.str_word = match w {
      None => None,
      Some(v) => {
        Some(v.expect_word("CognitionState::eval_error(): Bad argument type").clone())
      },
    };
    if let None = self.current_ref().err_stack {
      let temp = self.pool.get_stack(1);
      self.current().err_stack = Some(temp);
    }
    let estack: &mut Stack = &mut self.current().err_stack.as_mut().unwrap();
    estack.push(verror);
  }

  pub fn isdelim(&self, c: char) -> bool {
    let cur = self.current_ref();
    let found = match &cur.delims {
      None => false,
      Some(s) => s.chars().any(|x| x == c),
    };
    (found && cur.dflag) || (!found && !cur.dflag)
  }
  pub fn isignore(&self, c: char) -> bool {
    let cur = self.current_ref();
    let found = match &cur.ignored {
      None => false,
      Some(s) => s.chars().any(|x| x == c),
    };
    (found && cur.iflag) || (!found && !cur.iflag)
  }
  pub fn issinglet(&self, c: char) -> bool {
    let cur = self.current_ref();
    let found = match &cur.singlets {
      None => false,
      Some(s) => s.chars().any(|x| x == c),
    };
    (found && cur.sflag) || (!found && !cur.sflag)
  }

  pub fn isfalias(&self, v: &Value) -> bool {
    let Value::Word(vw) = v else { return false };
    match &self.current_ref().faliases {
      None => false,
      Some(f) => f.iter().any(|s| *s == vw.str_word ),
    }
  }

  pub fn current(&mut self) -> &mut Container {
    self.stack.last_mut().expect("Cognition metastack was empty").metastack_container()
  }
  pub fn current_ref(&self) -> &Container {
    self.stack.last().expect("Cognition metastack was empty").metastack_container_ref()
  }
  pub fn pop_cur(&mut self) -> Value {
    self.stack.pop().expect("Cognition metastack was empty")
  }
  pub fn push_cur(mut self, v: Value) -> Self {
    self.stack.push(v);
    self
  }

  pub fn push_quoted(&mut self, v: Value) {
    let mut wrapper: Value = self.pool.get_vstack(1);
    let Value::Stack(w) = &mut wrapper else { panic!("Pool::get_vstack() failed") };
    w.container.stack.push(v);
    self.current().stack.push(wrapper);
  }

  pub fn evalf(mut self, _v: &Value) -> Self {
    self.i = 0;
    self
  }

  fn evalstack(mut self, val: Value) -> Self {
    let Value::Stack(mut _vstack) = val else { panic!("CognitionState::evalstack(): Bad argument type") };
    //vstack.container.stack.push(Value::Stack(Box::new(VStack{ container: Container{..Default::default()} })));
    self.i = 0;
    self
  }
  fn evalmacro(mut self, _vmacro: Value) -> Self {
    self.i = 1;
    self
  }

  fn crank(mut self) -> Self {
    let mut cur_v = self.pop_cur();
    let cur = cur_v.metastack_container();

    let cranks = match cur.cranks.as_mut() {
      None => return self.push_cur(cur_v),
      Some(c) => c,
    };
    let high_tide = |c: &Crank| c.modulo == 0 && c.base != 0;
    let cindex: Option<usize> = cranks.iter().position(high_tide);
    let Some(cidx) = cindex else {
      cur.inc_crank();
      return self.push_cur(cur_v)
    };
    let fixedindex: isize = cur.stack.len() as isize - 1 - cidx as isize;
    if fixedindex < 0 {
      self.eval_error("CRANK TOO DEEP", None);
      cur.inc_crank();
      return self.push_cur(cur_v);
    }
    let needseval = cur.stack.remove(fixedindex as usize);
    match needseval {
      Value::Stack(_) =>
        self.push_cur(cur_v).evalstack(needseval),
      Value::Macro(_) =>
        self.push_cur(cur_v).evalmacro(needseval),
      _ =>
        panic!("BAD VALUE ON STACK"),
    }
  }

  fn evaluate(mut self, v: Value) -> Self {
    self = self.evalf(&v);
    self.pool.add_val(v);
    self
  }

  pub fn eval(mut self, v: Value) -> Self {
    let cur = self.current_ref();
    if self.isfalias(&v) {
      return match &cur.cranks {
        None => self.evaluate(v),
        Some(cranks) => {
          if cranks.len() == 0 {
            self.evaluate(v)
          } else if cranks[0].base != 1 && cranks[0].modulo != 1 { // if base==0, then modulo==0
            self.evaluate(v)
          } else {
            self.pool.add_val(v);
            self
          }
        },
      }
    }
    self.push_quoted(v);
    self.crank()
  }
}
