#![allow(dead_code)]
#![allow(unused_macros)]

#[macro_use]
pub mod macros;
pub mod tree;
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
pub type WordTable = HashMap<String, WordDef>;
pub type Family = Vec<*const Value>;

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

pub enum RefState {
  Independent,
  Dependent,
  Recycled,
}

impl Clone for RefState {
  fn clone (&self) -> Self {
    match self {
      Self::Independent => Self::Independent,
      Self::Dependent   => Self::Dependent,
      Self::Recycled    => Self::Recycled,
    }
  }
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
  pub state: RefState,

  word_table: Option<WordTable>,
}

impl Default for Container {
  fn default() -> Self {
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
      state: RefState::Independent,

      word_table: None,
    }
  }
}

impl Container {
  pub fn with_stack(stack: Stack) -> Self {
    Container{ stack, ..Default::default() }
  }
  pub fn with_capacity(capacity: usize) -> Self {
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
  pub fn isfalias(&self, v: &Value) -> bool {
    let Value::Word(vw) = v else { return false };
    match &self.faliases {
      None => false,
      Some(f) => f.iter().any(|s| *s == vw.str_word ),
    }
  }
  pub fn add_word(&mut self, v: Value, name: String) {
    match &mut self.word_table {
      Some(wt) => {
        wt.insert(name, WordDef::Val(v));
      },
      None => {
        let mut wt = WordTable::with_capacity(DEFAULT_WORD_TABLE_SIZE);
        wt.insert(name, WordDef::Val(v));
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
  pub macro_stack: Stack,
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
  pub str_word: Option<String>,
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
  pub fn with_capacity(capacity: usize) -> Self {
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

pub enum WordDef {
  Val(Value),
  DRef(*const Value),
  MRef(*const Value),
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
  pub chroots: Vec<Stack>, // meta metastack
  pub stack: Stack, // metastack
  pub exited: bool,
  pub exit_code: Option<String>,
  pub args: Strings,
  pub pool: Pool,
  pub i: i32, // to keep rust-analyser happy for the moment
}

impl CognitionState {
  pub fn new(stack: Stack) -> Self {
    Self{ chroots: Vec::<Stack>::with_capacity(DEFAULT_STACK_SIZE),
          stack,
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

  fn eval_value(mut self, _val: &Value) -> Self {
    self.i = 0;
    self
  }

  fn evalstack_ref(mut self, _val: *const Value, callword: Option<&Value>, _family: &mut Family) -> Self {
    if let None = callword { return self }
    self.i = 0;
    self
  }
  fn evalmacro_ref(mut self, _val: *const Value, callword: Option<&Value>, _family: &mut Family) -> Self {
    if let None = callword { return self }
    self.i = 0;
    self
  }

  fn evalstack(mut self, mut val: Value, callword: Option<&Value>, family: &mut Family) -> Self {
    family.push(&val as *const Value);
    let Value::Stack(vstack) = &mut val else { panic!("CognitionState::evalstack(): Bad argument type") };
    let stack = &mut vstack.container.stack;

    let mut local_family = self.pool.get_family();

    stack.reverse();

    let Some(v) = stack.pop() else { return self };
    //self = self.eval_value(v);
    match &v {
      Value::Word(word) => {

        let mut family_container: &Container;
        self = loop {
          let Some(family_stack) = family.pop() else {
            self.push_quoted(v);
            break self;
          };
          // Dereference read-only family stack pointer
          // This points to a value which is currently
          // being held in an evalstack() instance, so
          // the memory is always available and static.
          unsafe {
            let Value::Stack(family_vstack) = &*family_stack else { panic!("Bad value in family stack") };
            let family_vstack: &VStack = family_vstack; // debugging
            family_container = &family_vstack.container;
          }
          if let Some(wt) = &family_container.word_table {
            if let Some(wdef) = wt.get(&word.str_word) {
              self.pool.add_val(v);
              match wdef {
                WordDef::Val(dval) => {
                  match dval {
                    Value::Stack(_) => break self.evalstack_ref(dval as *const Value, callword, family),
                    Value::Macro(_) => break self.evalmacro_ref(dval as *const Value, callword, family),
                    _ => panic!("Bad value type in word table"),
                  }
                },
                WordDef::DRef(dval_pointer) => break self.evalstack_ref(dval_pointer.clone(), callword, family),
                WordDef::MRef(mval_pointer) => break self.evalmacro_ref(mval_pointer.clone(), callword, family),
              }
            }
          }
          if family_container.isfalias(&v) {
            self = 'crank: {
              if let Some(cranks) = &self.current_ref().cranks {
                if let Some(crank) = cranks.first() {
                  if crank.modulo == 1 || crank.base == 1 {
                    break 'crank self;
                  }
                }
              }
              self.evalf(&v)
            };
            self.pool.add_val(v);
            break self;
          }

          local_family.push(family_stack);
        };
        while let Some(f) = local_family.pop() { family.push(f); }


      },



      _ => {
        self.push_quoted(v);
      }
    }


                // TODO: Replace with pool-aware hashtable when it comes available
                //       The new hashtable will also return the original key with
                //       remove() so that it can be used later in insertion
                // let Some(WordDef::Val(definition)) = wt.remove(&w.str_word) else { unreachable!() };
                // let mut tmpkey = self.pool.get_string(w.str_word.len());
                // tmpkey.push_str(&w.str_word);
                // match &definition {
                //   Value::Stack(_) => {
                //     wt.insert(tmpkey, WordDef::DRef(&definition as *const Value));
                //     self = self.evalstack_ref(&definition as *const Value, family);

                //   },
                //   Value::Macro(_) => {
                //     wt.insert(tmpkey, WordDef::MRef(&definition as *const Value));
                //     self = self.evalmacro_ref(&definition as *const Value, family);

                //   },
                //   _ => panic!("Bad value type in word table"),
                // };



    for v in stack[1..].iter() {
      print!("val: ");
      v.print("\n");
    }



    // if current container's word table had to be used:
    let cur = self.current();
    let original_cur_state = cur.state.clone();
    cur.state = RefState::Dependent;
    let original_cur_pointer = cur as *mut Container;


    // stuff

    unsafe {
      if let RefState::Dependent = (*original_cur_pointer).state {
        let Some(_wt) = &mut (*original_cur_pointer).word_table else { panic!("Current container: word table destroyed") };
        //wt.insert(tmpkey, WordDef::Val(definition));
        (*original_cur_pointer).inc_crank();
        (*original_cur_pointer).state = original_cur_state;
      }
    }
    // done

    self
  }

  fn evalmacro(mut self, _vmacro: Value, _callword: Option<Value>, _family: &mut Family) -> Self {
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
    let mut family = self.pool.get_family();
    match needseval {
      Value::Stack(_) => self.push_cur(cur_v).evalstack(needseval, None, &mut family),
      Value::Macro(_) => self.push_cur(cur_v).evalmacro(needseval, None, &mut family),
      _ => panic!("Bad value on stack"),
    }
  }

  pub fn eval(mut self, v: Value) -> Self {
    let cur = self.current_ref();
    if cur.isfalias(&v) {
      self = match &cur.cranks {
        None => self.evalf(&v),
        Some(cranks) => 'crank: {
          if let Some(crank) = cranks.first() {
            if crank.base == 1 || crank.modulo == 1 { break 'crank self; }
          }
          self.evalf(&v)
        },
      };
      self.pool.add_val(v);
      return self;
    }
    self.push_quoted(v);
    self.crank()
  }
}
