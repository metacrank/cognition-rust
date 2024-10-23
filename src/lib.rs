#![allow(dead_code)]
#![allow(unused_macros)]

#[macro_use]
pub mod macros;
pub mod tree;
pub mod pool;
pub mod math;
pub mod builtins;

use crate::pool::Pool;
use crate::macros::*;
use crate::math::Math;

use std::collections::HashSet;
use std::collections::HashMap;
use std::default::Default;
use std::io::Write;
use std::io::stdout;
//use std::str::Chars;
//use std::any::Any;

pub type CognitionFunction = fn(CognitionState, Option<&Value>) -> CognitionState;
pub type Stack = Vec<Value>;
pub type Cranks = Vec<Crank>;
pub type Strings = Vec<String>;
pub type Faliases = HashSet<String>;
pub type WordTable = HashMap<String, Option<WordDef>>;

pub struct Family { stack: Vec<*const Value> }

impl Family {
  fn new() -> Self { Self { stack: Vec::<*const Value>::new() } }
  fn with_capacity(capacity: usize) -> Self {
    Self { stack: Vec::<*const Value>::with_capacity(capacity) }
  }
}

/// Only send families which are going to be
/// completely reused: families in a pool
unsafe impl Send for Family {}

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
  fn copyfunc(&self) -> Box<dyn Custom + Send>;
}

/// Example of a rust library that would utilise Custom
/// If the type has to be unsafely allocated, or is a C
/// library, implement the Drop trait to free memory
#[derive(Clone)]
struct ExampleCustom {
  i: String,
}

impl Custom for ExampleCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check_pretty!(f, self.i.as_bytes());
  }
  fn copyfunc(&self) -> Box<dyn Custom + Send> {
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
  fn copyfunc(&self) -> Box<dyn Custom + Send> {
    Box::new(Void{})
  }
}

#[derive(Clone)]
pub struct Crank {
  pub modulo: i32,
  pub base: i32,
}

trait WTCrankFuncs {
  fn insert_stolen_value(&mut self, key: &String, word_def:  Option<WordDef>);
}

impl WTCrankFuncs for WordTable {
  fn insert_stolen_value(&mut self, key: &String, word_def: Option<WordDef>) {
    let Some(current_def) = self.get_mut(key) else { return };
    match current_def.as_ref() {
      None                  => { return },                  // removed from all existence
      Some(WordDef::Val(_)) => { return },                  // redefined by some other function
      Some(WordDef::Ref(_)) => { *current_def = word_def }, // replace reference with stolen value
    }
  }
}

pub struct Container {
  pub stack: Stack,
  pub err_stack: Option<Stack>,
  pub cranks: Option<Cranks>,
  pub math: Option<Math>,
  pub faliases: Option<Faliases>,
  pub delims: Option<String>,
  pub ignored: Option<String>,
  pub singlets: Option<String>,
  pub dflag: bool,
  pub iflag: bool,
  pub sflag: bool,
  pub dependent: bool,

  word_table: Option<WordTable>,
}

impl Default for Container {
  fn default() -> Self {
    Container {
      stack: Stack::new(),
      err_stack: None,
      cranks: None,
      math: None,
      faliases: None,
      delims: None,
      ignored: None,
      singlets: None,
      dflag: false,
      iflag: true,
      sflag: true,
      dependent: false,

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
    let mut f = Faliases::with_capacity(DEFAULT_FALIASES_SIZE);
    f.insert(String::from("f"));
    f.insert(String::from("ing"));
    Some(f)
  }
  pub fn isfalias(&self, v: &Value) -> bool {
    match &self.faliases {
      None => false,
      Some(f) => f.contains(&v.vword_ref().str_word),
    }
  }
  pub fn add_word(&mut self, v: Value, name: String) {
    match &mut self.word_table {
      Some(wt) => {
        wt.insert(name, Some(WordDef::Val(v)));
      },
      None => {
        let mut wt = WordTable::with_capacity(DEFAULT_WORD_TABLE_SIZE);
        wt.insert(name, Some(WordDef::Val(v)));
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
  custom: Box<dyn Custom + Send>,
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
  pub fn with_custom(custom: Box<dyn Custom + Send>) -> VCustom {
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

macro_rules! return_value_type {
  ($self:ident,$letpat:pat,$v:tt) => {{ let $letpat = $self else { panic!("Value type assumed") }; $v }};
}
macro_rules! is_value_type {
  ($self:ident,$letpat:pat) => { if let $letpat = $self { true } else { false } }
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
            fwrite_check_pretty!(f, b"':");
          }
          None => {
            fwrite_check_pretty!(f, b"(none):");
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
  pub fn value_stack(&mut self) -> &mut Stack {
    match self {
      Value::Stack(vstack) => &mut vstack.container.stack,
      Value::Macro(vmacro) => &mut vmacro.macro_stack,
      _ => bad_value_err!(),
    }
  }
  pub fn value_stack_ref(&self) -> &Stack {
    match self {
      Value::Stack(vstack) => &vstack.container.stack,
      Value::Macro(vmacro) => &vmacro.macro_stack,
      _ => bad_value_err!(),
    }
  }

  pub fn vword(self) -> Box<VWord> { return_value_type!(self, Value::Word(v), v) }
  pub fn vstack(self) -> Box<VStack> { return_value_type!(self, Value::Stack(v), v) }
  pub fn vmacro(self) -> Box<VMacro> { return_value_type!(self, Value::Macro(v), v) }
  pub fn verror(self) -> Box<VError> { return_value_type!(self, Value::Error(v), v) }
  pub fn vcustom(self) -> Box<VCustom> { return_value_type!(self, Value::Custom(v), v) }
  pub fn vfllib(self) -> Box<VFLLib> { return_value_type!(self, Value::FLLib(v), v) }
  pub fn vword_ref(&self) -> &Box<VWord> { return_value_type!(self, Value::Word(v), v) }
  pub fn vstack_ref(&self) -> &Box<VStack> { return_value_type!(self, Value::Stack(v), v) }
  pub fn vmacro_ref(&self) -> &Box<VMacro> { return_value_type!(self, Value::Macro(v), v) }
  pub fn verror_ref(&self) -> &Box<VError> { return_value_type!(self, Value::Error(v), v) }
  pub fn vcustom_ref(&self) -> &Box<VCustom> { return_value_type!(self, Value::Custom(v), v) }
  pub fn vfllib_ref(&self) -> &Box<VFLLib> { return_value_type!(self, Value::FLLib(v), v) }
  pub fn vword_mut(&mut self) -> &mut Box<VWord> { return_value_type!(self, Value::Word(v), v) }
  pub fn vstack_mut(&mut self) -> &mut Box<VStack> { return_value_type!(self, Value::Stack(v), v) }
  pub fn vmacro_mut(&mut self) -> &mut Box<VMacro> { return_value_type!(self, Value::Macro(v), v) }
  pub fn verror_mut(&mut self) -> &mut Box<VError> { return_value_type!(self, Value::Error(v), v) }
  pub fn vcustom_mut(&mut self) -> &mut Box<VCustom> { return_value_type!(self, Value::Custom(v), v) }
  pub fn vfllib_mut(&mut self) -> &mut Box<VFLLib> { return_value_type!(self, Value::FLLib(v), v) }

  pub fn is_word(&self) -> bool { is_value_type!(self, Value::Word(_)) }
  pub fn is_stack(&self) -> bool { is_value_type!(self, Value::Stack(_)) }
  pub fn is_macro(&self) -> bool { is_value_type!(self, Value::Macro(_)) }
  pub fn is_error(&self) -> bool { is_value_type!(self, Value::Error(_)) }
  pub fn is_custom(&self) -> bool { is_value_type!(self, Value::Custom(_)) }
  pub fn is_fllib(&self) -> bool { is_value_type!(self, Value::FLLib(_)) }
}

#[derive(Clone)]
pub enum DefRef {
  D(*const Value),
  M(*const Value),
}

impl DefRef {
  fn as_ptr(&self) -> *const Value {
    match self {
      Self::D(r) => r.clone(),
      Self::M(r) => r.clone(),
    }
  }
}

// Do not ever use this trait. Ensure that all WordDefs being shared
// between threads are recursively cloned so that no references are sent
unsafe impl Send for DefRef {}

pub enum WordDef {
  Val(Value),
  Ref(DefRef),
}

impl WordDef {
  /// No ref will ever be pushed to a word table without
  /// an eval_value instance holding the Value
  unsafe fn copy_value(&self, state: &mut CognitionState) -> Value {
    match self {
      Self::Val(v) => state.value_copy(v),
      Self::Ref(r) => state.value_copy(&*r.as_ptr()),
    }
  }
}

pub struct Parser {
  source: Option<String>,
  i: usize,
  c: Option<char>,
  line: usize,
  column: usize,
}

impl Parser {
  pub fn new(source: Option<String>) -> Parser {
    if source.is_none() { return Parser{ source: None, i: 0, c: None, line: 1, column: 0 } }
    let c = match source.as_ref().unwrap().get(..) { Some(st) => st.chars().next(), None => None };
    Parser{ source, i: 0, c, line: 1, column: 0 }
  }
  pub fn next(&mut self) {
    if self.c.is_none() || self.source.is_none() { return }
    self.i += self.c.unwrap().len_utf8();
    self.c = match self.source.as_ref().unwrap().get(self.i..) {
      Some(st) => st.chars().next(),
      None => None
    };
    self.column += 1;
    if self.c == Some('\n') {
      self.line += 1;
      self.column = 0;
    }
  }

  pub fn reset(&mut self, source: String) {
    self.column = 0;
    self.line = 1;
    self.c = match source.get(..) { Some(st) => st.chars().next(), None => None };
    self.i = 0;
    self.source = Some(source);
  }

  pub fn source(&mut self) -> Option<String> {
    self.source.take()
  }
  pub fn line(&self) -> usize { self.line }
  pub fn column(&self) -> usize { self.column }

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
    if state.issinglet(c) {
      v.str_word.push(c);
      self.next();
      return Some(Value::Word(v));
    }
    if !skipped {
      v.str_word.push(c);
      self.next();
    }
    while let Some(c) = self.c {
      if state.isdelim(c) { break };
      v.str_word.push(c);
      self.next();
      if state.issinglet(c) { return Some(Value::Word(v)) };
    }
    Some(Value::Word(v))
  }

  // returns just the next character without delim/ignore/singlet behaviour
  #[allow(dead_code)]
  pub fn get_next_char(&mut self, state: &mut CognitionState) -> Option<Box<VWord>> {
    let ch = self.c;
    self.next();
    match ch {
      None => None,
      Some(c) => {
        let mut v = state.pool.get_vword(c.len_utf8());
        v.str_word.push(c);
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
  pub family: Family, // for reuse
  pub parser: Option<Parser>,
  pub returned: bool,
  pub exited: bool,
  pub exit_code: Option<String>,
  pub args: Strings,
  pub pool: Pool,
}

macro_rules! eval_value_if_cranked {
  ($self:ident,$v:ident,$callword:ident,$local_family:expr,$do_crank:expr) => {
    if let Some(cranks) = &$self.current_ref().cranks {
      if let Some(crank) = cranks.first() {
        if crank.modulo == 0 && crank.base != 0 {
          $self = $self.eval_value($v, $callword, $local_family, false, false, $do_crank, false);
          continue;
        }
      }
    }
  };
}

impl CognitionState {
  pub fn new(stack: Stack) -> Self {
    Self{ chroots: Vec::<Stack>::with_capacity(DEFAULT_STACK_SIZE), stack,
          family: Family::with_capacity(DEFAULT_STACK_SIZE), parser: None,
          returned: false, exited: false, exit_code: None, args: Strings::new(),
          pool: Pool::new(), }
  }

  pub fn eval_error(mut self, e: &'static str, w: Option<&Value>) -> Self {
    let mut verror = self.pool.get_verror(e.len());
    verror.error.push_str(e);
    verror.str_word = match w {
      None => None,
      Some(v) => Some(self.string_copy(&v.vword_ref().str_word)),
    };
    if let None = self.current_ref().err_stack {
      let temp = self.pool.get_stack(1);
      self.current().err_stack = Some(temp);
    }
    let estack: &mut Stack = &mut self.current().err_stack.as_mut().unwrap();
    estack.push(Value::Error(verror));
    self
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

  pub fn string_copy(&mut self, s: &String) -> String {
    let mut newstr = self.pool.get_string(s.len());
    newstr.push_str(s);
    newstr
  }

  pub fn contain_copy(&mut self, old: &Container, new: &mut Container) {
    for v in old.stack.iter() {
      new.stack.push(self.value_copy(v));
    }
    if let Some(ref err_stack) = old.err_stack {
      new.err_stack = Some(self.pool.get_stack(err_stack.len()));
      let new_err_stack = new.err_stack.as_mut().unwrap();
      for v in err_stack.iter() {
        new_err_stack.push(self.value_copy(v));
      }
    }
    if let Some(ref cranks) = old.cranks {
      new.cranks = Some(self.pool.get_cranks(cranks.len()));
      let new_cranks = new.cranks.as_mut().unwrap();
      for crank in cranks.iter() {
        new_cranks.push(crank.clone());
      }
    }
    if let Some(ref math) = old.math {
      new.math = Some(self.pool.get_math(math.base()));
      let new_math = new.math.as_mut().unwrap();
      math.copy_into(new_math, self);
    }
    if let Some(ref faliases) = old.faliases {
      new.faliases = Some(self.pool.get_faliases());
      let new_faliases = new.faliases.as_mut().unwrap();
      for alias in faliases.iter() {
        new_faliases.insert(self.string_copy(alias));
      }
    }
    if let Some(ref delims) = old.delims {
      new.delims = Some(self.string_copy(delims));
    }
    if let Some(ref ignored) = old.ignored {
      new.ignored = Some(self.string_copy(ignored));
    }
    if let Some(ref singlets) = old.singlets {
      new.singlets = Some(self.string_copy(singlets))
    }
    new.dflag = old.dflag;
    new.iflag = old.iflag;
    new.sflag = old.sflag;
    new.dependent = false;

    if let Some(ref word_table) = old.word_table {
      new.word_table = Some(self.pool.get_word_table());
      for (key, word_def) in word_table.iter() {
        let new_word_def = match word_def {
          Some(wd) => Some(WordDef::Val(unsafe{ wd.copy_value(self) })), // see documentation
          None     => None,
        };
        new.word_table.as_mut().unwrap().insert(self.string_copy(key), new_word_def);
      }
    }
  }

  pub fn value_copy(&mut self, v: &Value) -> Value {
    match v {
      Value::Word(vword) => {
        let mut new_vword = self.pool.get_vword(vword.str_word.len());
        new_vword.str_word.push_str(&vword.str_word);
        Value::Word(new_vword)
      },
      Value::Stack(vstack) => {
        let mut new_vstack = self.pool.get_vstack(vstack.container.stack.len());
        self.contain_copy(&vstack.container, &mut new_vstack.container);
        Value::Stack(new_vstack)
      },
      Value::Macro(vmacro) => {
        let mut new_vmacro = self.pool.get_vmacro(vmacro.macro_stack.len());
        for v in vmacro.macro_stack.iter() {
          new_vmacro.macro_stack.push(self.value_copy(v));
        }
        Value::Macro(new_vmacro)
      },
      Value::Error(verror) => {
        let mut new_verror = self.pool.get_verror(verror.error.len());
        new_verror.error.push_str(&verror.error);
        if let Some(ref word) = verror.str_word {
          new_verror.str_word = Some(self.string_copy(word));
        }
        Value::Error(new_verror)
      },
      Value::Custom(vcustom) => Value::Custom(self.pool.get_vcustom(vcustom.custom.copyfunc())),
      Value::FLLib(vfllib) => Value::FLLib(self.pool.get_vfllib(vfllib.fllib)),
    }
  }

  pub fn current(&mut self) -> &mut Container {
    &mut self.stack.last_mut().expect("Cognition metastack was empty").vstack_mut().container
  }
  pub fn current_ref(&self) -> &Container {
    &self.stack.last().expect("Cognition metastack was empty").vstack_ref().container
  }
  pub fn pop_cur(&mut self) -> Value {
    self.stack.pop().expect("Cognition metastack was empty")
  }
  pub fn push_cur(mut self, v: Value) -> Self {
    self.stack.push(v); self
  }

  pub fn push_quoted(&mut self, v: Value) {
    let mut wrapper = self.pool.get_vstack(1);
    wrapper.container.stack.push(v);
    self.current().stack.push(Value::Stack(wrapper));
  }

  // don't increment crank
  pub fn evalf(mut self, alias: &Value) -> Self {
    let Some(v) = self.current().stack.pop() else { return self.eval_error("EMPTY STACK", Some(alias)) };
    self = match &v {
      Value::Stack(_) => self.evalstack(v, None, false),
      Value::Macro(_) => self.evalmacro(v, None, false),
      _ => bad_value_err!(),
    };
    self
  }
  fn try_evalf(mut self, v: Value, always_evalf: bool) -> Self {
    self = 'crank: {
      if !always_evalf {
        if let Some(cranks) = &self.current_ref().cranks {
          if let Some(crank) = cranks.first() {
            if crank.modulo == 1 || crank.base == 1 {
              break 'crank self;
            }
          }
        }
      }
      self.evalf(&v)
    };
    self.pool.add_val(v);
    self
  }

  fn evalword_in_cur(mut self, v: Value, always_evalf: bool, only_evalf: bool,
                     definition: &mut (Option<String>, Option<WordDef>)) -> Self {

    if !only_evalf {
      if let Some(wt) = &mut self.current().word_table {
        if let Some(wdef) = wt.get_mut(&v.vword_ref().str_word) {
          definition.1 = wdef.take();
          if let Some(WordDef::Val(ref dval)) = definition.1 {
            definition.0 = Some(v.vword_ref().str_word.clone());
            self = match dval {
              Value::Stack(_) => {
                *wdef = Some(WordDef::Ref(DefRef::D(dval as *const Value)));
                self.evalstack_ref(dval as *const Value, Some(&v))
              },
              Value::Macro(_) => {
                *wdef = Some(WordDef::Ref(DefRef::M(dval as *const Value)));
                self.evalmacro_ref(dval as *const Value, Some(&v))
              },
              _ => panic!("Bad value type in word table")
            };
            self.pool.add_val(v);
            return self;
          }
          else if let Some(WordDef::Ref(ref defref)) = definition.1 {
            self = match defref {
              DefRef::D(dval_pointer) => {
                let defptr = dval_pointer.clone();
                *wdef = definition.1.take();
                self.evalstack_ref(defptr, Some(&v))
              },
              DefRef::M(mval_pointer) => {
                let defptr = mval_pointer.clone();
                *wdef = definition.1.take();
                self.evalmacro_ref(defptr, Some(&v))
              },
            };
            self.pool.add_val(v);
            return self;
          }
        }
      }
    }
    if self.current().isfalias(&v) { return self.try_evalf(v, always_evalf); }
    self.push_quoted(v);
    self
  }

  fn evalword(mut self, v: Value, local_family: &mut Family, always_evalf: bool,
              only_evalf: bool, definition: &mut (Option<String>, Option<WordDef>)) -> Self {

    loop {
      let Some(family_stack) = self.family.stack.pop() else {
        // Assuming family stack has failed
        break self.evalword_in_cur(v, always_evalf, only_evalf, definition);
      };

      // Dereference read-only family stack pointer
      // This points to a value which is currently
      // being held in an evalstack() instance, so
      // the memory is always available and static.
      let family_container = unsafe {
        let family_vstack: &VStack = &(*family_stack).vstack_ref();
        &family_vstack.container
      };
      if !only_evalf {
        if let Some(wt) = &family_container.word_table {
          if let Some(Some(wdef)) = wt.get(&v.vword_ref().str_word) {
            self = match wdef {
              WordDef::Val(dval) => {
                match dval {
                  Value::Stack(_) => self.evalstack_ref(dval as *const Value, Some(&v)),
                  Value::Macro(_) => self.evalmacro_ref(dval as *const Value, Some(&v)),
                  _ => panic!("Bad value type in word table"),
                }
              },
              WordDef::Ref(DefRef::D(dval_pointer)) => self.evalstack_ref(dval_pointer.clone(), Some(&v)),
              WordDef::Ref(DefRef::M(mval_pointer)) => self.evalmacro_ref(mval_pointer.clone(), Some(&v)),
            };
            self.pool.add_val(v);
            break self;
          }
        }
      }
      if family_container.isfalias(&v) {
        break self.try_evalf(v, always_evalf);
      }
      local_family.stack.push(family_stack);
    }
  }

  fn eval_value(mut self, v: Value, callword: Option<&Value>, local_family: &mut Family,
                always_evalf: bool, only_evalf: bool, do_crank: bool, crank_on_return: bool) -> Self {

    // Ensure the current container doesn't die so we can increment the crank later
    let cur = self.current();
    let original_cur_state = cur.dependent;
    cur.dependent = true; // container will live
    let original_cur_pointer = cur as *mut Container;

    let mut not_cranked: bool = true;
    let mut definition: (Option<String>, Option<WordDef>) = (None, None);
    self = match &v {
      Value::Word(_) => self.evalword(v, local_family, always_evalf, only_evalf, &mut definition),
      Value::Error(_) => panic!("VError on stack"),
      Value::Custom(_) => {
        self.push_quoted(v);
        if do_crank { not_cranked = false; self.crank() }
        else { self }
      },
      Value::FLLib(vfllib) => {
        let fllib = vfllib.fllib.clone();
        self.pool.add_val(v);
        fllib(self, callword.clone())
      },
      _ => {
        self.current().stack.push(v);
        if do_crank { not_cranked = false; self.crank() }
        else { self }
      },
    };

    while let Some(f) = local_family.stack.pop() { self.family.stack.push(f); }

    // Access the original current container
    // Refactor this code into more safe functions
    unsafe {
      if (*original_cur_pointer).dependent {
        if let Some(ref key) = definition.0 {
          if let Some(wt) = (*original_cur_pointer).word_table.as_mut() {  // take *mut WordTable pointer?
            wt.insert_stolen_value(key, definition.1);
            self.pool.add_string(definition.0.unwrap());
          } else { self.pool.add_def(definition); }
        }
        if do_crank && not_cranked || crank_on_return && self.returned { (*original_cur_pointer).inc_crank(); }
        (*original_cur_pointer).dependent = original_cur_state;
      }
      else if definition.0.is_some() { self.pool.add_def(definition); }
    }
    self
  }

  // don't crank last value
  fn evalstack_ref(mut self, val: *const Value, callword: Option<&Value>) -> Self {
    self.family.stack.push(val);
    // Evalword promises this pointer is a valid and unique reference
    let vstack = { unsafe{&*val} }.vstack_ref();
    let stack = &vstack.container.stack;
    let mut local_family = self.pool.get_family();

    if let Some(v) = stack.first() {
      // First value always evaluated
      let new_v = self.value_copy(v);
      self = self.eval_value(new_v, callword, &mut local_family, true, false, stack.len() > 1, false);
      // Loop over stack, not cranking last value
      let n = stack.len();
      for i in 1..n {
        if self.returned { break }
        let new_v = self.value_copy(&stack[i]);
        eval_value_if_cranked!(self, new_v, callword, &mut local_family, i != n - 1);
        self = self.eval_value(new_v, callword, &mut local_family, false, true, i != n - 1, false);
      }
      if self.returned { self.returned = false }
    }

    self.pool.add_family(local_family);
    self.family.stack.pop();
    self
  }
  // don't crank
  fn evalmacro_ref(mut self, val: *const Value, callword: Option<&Value>) -> Self {
    // Evalword promises this pointer is a valid and unique reference
    let vmacro = { unsafe{&*val} }.vmacro_ref();
    let macro_stack = &vmacro.macro_stack;
    let mut local_family = self.pool.get_family();

    for v in macro_stack.iter() {
      let new_v = self.value_copy(v);
      self = self.eval_value(new_v, callword, &mut local_family, true, false, false, false);
      if self.returned { self.returned = false; break }
    }

    self.pool.add_family(local_family);
    self
  }

  fn evalstack(mut self, mut val: Value, callword: Option<&Value>, crank_last: bool) -> Self {
    self.family.stack.push(&val as *const Value);
    let vstack = val.vstack_mut();
    let stack = &mut vstack.container.stack;
    let mut local_family = self.pool.get_family();

    stack.reverse();

    // First value is always evaluated
    if let Some(v) = stack.pop() {
      self = self.eval_value(v, callword, &mut local_family, true, false, crank_last || stack.len() != 0, false);
    }
    // Loop over stack
    while let Some(v) = stack.pop() {
      if self.returned { break }
      eval_value_if_cranked!(self, v, callword, &mut local_family, crank_last || stack.len() != 0);
      self = self.eval_value(v, callword, &mut local_family, false, true, crank_last || stack.len() != 0, false);
    }
    if self.returned { self.returned = false }

    self.pool.add_family(local_family);
    self.pool.add_val(val);
    self.family.stack.pop();
    self
  }

  // crank once
  fn evalmacro(mut self, mut val: Value, callword: Option<&Value>, crank_last: bool) -> Self {
    let vmacro = val.vmacro_mut();
    let macro_stack = &mut vmacro.macro_stack;
    let mut local_family = self.pool.get_family();

    macro_stack.reverse();

    while let Some(v) = macro_stack.pop() {
      self = self.eval_value(v, callword, &mut local_family, true, false, macro_stack.len() == 0 && crank_last, crank_last);
      if self.returned { self.returned = false; break }
    }

    self.pool.add_family(local_family);
    self.pool.add_val(val);
    self
  }

  // fn evalstack(mut self, mut val: Value, callword: Option<&Value>, crank_last: bool) -> Self {
  //   let (stack, is_macro) = if val.is_stack() {
  //     self.family.stack.push(&val as )
  //     (&mut val.vstack_mut().container.stack, false)
  //   } else {
  //     (&mut val.vmacro_mut().macro_stack, true)
  //   };
  //   let mut local_family = self.pool.get_family();

  //   let mut i = 0;
  //   while i < stack.len() {
  //     let v = stack[i];
  //     if i != stack.len() - 1 {
  //       self = self.eval_value(v, callword, &mut local_family, true, false, false, crank_last);
  //       if self.returned { self.returned = false; break }
  //       i += 1; continue
  //     }


  //   }

  //   self.pool.add_family(local_family);

  //   self
  // }

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
      self = self.eval_error("CRANK TOO DEEP", None);
      cur.inc_crank();
      return self.push_cur(cur_v);
    }
    let needseval = cur.stack.remove(fixedindex as usize);
    self = match needseval {
      Value::Stack(_) => self.push_cur(cur_v).evalstack(needseval, None, true),
      Value::Macro(_) => self.push_cur(cur_v).evalmacro(needseval, None, true),
      _ => bad_value_err!(),
    };
    self
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
