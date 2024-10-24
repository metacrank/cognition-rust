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
//use std::Arc::Arc;
use std::sync::Arc;
//use std::str::Chars;
use std::any::Any;

pub type CognitionFunction = fn(CognitionState, Option<&Value>) -> CognitionState;
pub type Stack = Vec<Value>;
pub type Cranks = Vec<Crank>;
pub type Strings = Vec<String>;
pub type Faliases = HashSet<String>;
pub type WordDef = Arc<Value>;
pub type Family = Vec<WordDef>;
pub type WordTable = HashMap<String, Arc<Value>>;

// pub struct Family { stack: Vec<*const Value> }

// impl Family {
//   fn new() -> Self { Self { stack: Vec::<*const Value>::new() } }
//   fn with_capacity(capacity: usize) -> Self {
//     Self { stack: Vec::<*const Value>::with_capacity(capacity) }
//   }
// }

// /// Only send families which are going to be
// /// completely reused: families in a pool
// unsafe impl Send for Family {}

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

pub trait Custom: Any {
  fn printfunc(&self, f: &mut dyn Write);
  // implemented as Box::new(self.clone()) in classes that implement Clone
  fn copyfunc(&self) -> Box<dyn Custom + Send + Sync>;
}

/// Useful Custom type
pub struct Void {}
impl Custom for Void {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check_pretty!(f, b"(void)");
  }
  fn copyfunc(&self) -> Box<dyn Custom + Send + Sync> {
    Box::new(Void{})
  }
}

#[derive(Clone)]
pub struct Crank {
  pub modulo: i32,
  pub base: i32,
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
pub struct VFLLib {
  fllib: CognitionFunction,
  pub str_word: Option<String>,
}
pub struct VCustom {
  custom: Option<Box<dyn Custom + Send + Sync>>,
}
#[derive(PartialEq, Eq, Clone)]
pub enum VControl {
  Eval,
  Return,
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
impl VFLLib {
  pub fn with_fn(fllib: CognitionFunction) -> VFLLib {
    VFLLib{ fllib, str_word: None }
  }
  pub fn with_nop() -> VFLLib {
    Self::with_fn(builtins::cog_nop)
  }
}
impl VCustom {
  pub fn with_custom(custom: Box<dyn Custom + Send + Sync>) -> VCustom {
    VCustom{ custom: Some(custom) }
  }
  pub fn with_void() -> VCustom {
    Self::with_custom(Box::new(Void{}))
  }
}

pub enum Value {
  Word(Box<VWord>),
  Stack(Box<VStack>),
  Macro(Box<VMacro>),
  Error(Box<VError>),
  FLLib(Box<VFLLib>),
  Custom(VCustom),
  Control(VControl)
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
      },
      Self::Stack(vstack) => {
        fwrite_check_pretty!(f, b"[ ");
        for v in vstack.container.stack.iter() { v.fprint(f, " "); }
        fwrite_check_pretty!(f, b"]");
      },
      Self::Macro(vmacro) => {
        fwrite_check_pretty!(f, b"( ");
        for v in vmacro.macro_stack.iter() { v.fprint(f, " "); }
        fwrite_check_pretty!(f, b")");
      },
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
      },
      Self::FLLib(vfllib) => {
        match &vfllib.str_word {
          Some(s) => {
            fwrite_check_pretty!(f, HBLK);
            s.fprint_pretty(f);
            fwrite_check_pretty!(f, COLOR_RESET);
          },
          None => {
            fwrite_check_pretty!(f, HBLK);
            fwrite_check_pretty!(f, b"FLLIB");
            fwrite_check_pretty!(f, COLOR_RESET);
          },
        }
      },
      Self::Custom(vcustom) => {
        if let Some(ref c) = vcustom.custom { c.printfunc(f) }
        else { fwrite_check_pretty!(f, b"(void)") }
      },
      Self::Control(vcontrol) => {
        fwrite_check_pretty!(f, GRN);
        match vcontrol {
          VControl::Eval   => fwrite_check_pretty!(f, b"eval"),
          VControl::Return => fwrite_check_pretty!(f, b"return"),
        }
        fwrite_check_pretty!(f, COLOR_RESET);
      },
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
  pub fn vfllib(self) -> Box<VFLLib> { return_value_type!(self, Value::FLLib(v), v) }
  pub fn vcustom(self) -> VCustom { return_value_type!(self, Value::Custom(v), v) }
  pub fn vcontrol(self) -> VControl { return_value_type!(self, Value::Control(v), v) }
  pub fn vword_ref(&self) -> &Box<VWord> { return_value_type!(self, Value::Word(v), v) }
  pub fn vstack_ref(&self) -> &Box<VStack> { return_value_type!(self, Value::Stack(v), v) }
  pub fn vmacro_ref(&self) -> &Box<VMacro> { return_value_type!(self, Value::Macro(v), v) }
  pub fn verror_ref(&self) -> &Box<VError> { return_value_type!(self, Value::Error(v), v) }
  pub fn vfllib_ref(&self) -> &Box<VFLLib> { return_value_type!(self, Value::FLLib(v), v) }
  pub fn vcustom_ref(&self) -> &VCustom { return_value_type!(self, Value::Custom(v), v) }
  pub fn vcontrol_ref(&self) -> &VControl { return_value_type!(self, Value::Control(v), v) }
  pub fn vword_mut(&mut self) -> &mut Box<VWord> { return_value_type!(self, Value::Word(v), v) }
  pub fn vstack_mut(&mut self) -> &mut Box<VStack> { return_value_type!(self, Value::Stack(v), v) }
  pub fn vmacro_mut(&mut self) -> &mut Box<VMacro> { return_value_type!(self, Value::Macro(v), v) }
  pub fn verror_mut(&mut self) -> &mut Box<VError> { return_value_type!(self, Value::Error(v), v) }
  pub fn vfllib_mut(&mut self) -> &mut Box<VFLLib> { return_value_type!(self, Value::FLLib(v), v) }
  pub fn vcustom_mut(&mut self) -> &mut VCustom { return_value_type!(self, Value::Custom(v), v) }
  pub fn vcontrol_mut(&mut self) -> &mut VControl { return_value_type!(self, Value::Control(v), v) }

  pub fn is_word(&self) -> bool { is_value_type!(self, Value::Word(_)) }
  pub fn is_stack(&self) -> bool { is_value_type!(self, Value::Stack(_)) }
  pub fn is_macro(&self) -> bool { is_value_type!(self, Value::Macro(_)) }
  pub fn is_error(&self) -> bool { is_value_type!(self, Value::Error(_)) }
  pub fn is_fllib(&self) -> bool { is_value_type!(self, Value::FLLib(_)) }
  pub fn is_custom(&self) -> bool { is_value_type!(self, Value::Custom(_)) }
  pub fn is_control(&self) -> bool { is_value_type!(self, Value::Control(_)) }
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

enum RecurseControl {
  Evalf(WordDef, Stack),
  Crank(WordDef, Stack),
  Def(WordDef, Value),
  None,
  Return,
}

impl CognitionState {
  pub fn new(stack: Stack) -> Self {
    Self{ chroots: Vec::<Stack>::with_capacity(DEFAULT_STACK_SIZE), stack,
          family: Family::with_capacity(DEFAULT_STACK_SIZE), parser: None,
          exited: false, exit_code: None, args: Strings::new(), pool: Pool::new(), }
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

  // pub fn word_def_copy(&mut self, word_def: &WordDef) -> WordDef {
  //   match word_def {
  //     WordDef::Stack(s) => {
  //       let mut new_vstack = self.pool.get_vstack(s.container.stack.len());
  //       self.contain_copy(&s.container, &mut new_vstack.container);
  //       WordDef::Stack(new_vstack.into())
  //     },
  //     WordDef::Macro(m) => {
  //       let mut new_vmacro = self.pool.get_vmacro(m.macro_stack.len());
  //       for v in m.macro_stack.iter() {
  //         new_vmacro.macro_stack.push(self.value_copy(v));
  //       }
  //       WordDef::Macro(new_vmacro.into())
  //     },
  //   }
  // }

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

    if let Some(ref word_table) = old.word_table {
      new.word_table = Some(self.pool.get_word_table());
      for (key, word_def) in word_table.iter() {
        let v = self.value_copy(&word_def);
        new.word_table.as_mut().unwrap().insert(self.string_copy(key), self.pool.get_word_def(v));
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
      Value::FLLib(vfllib) => Value::FLLib(self.pool.get_vfllib(vfllib.fllib)),
      Value::Custom(vcustom) => Value::Custom({
        match vcustom.custom {
          Some(ref custom) => self.pool.get_vcustom(custom.copyfunc()),
          None             => VCustom{ custom: None },
        }
      }),
      Value::Control(vcontrol) => Value::Control(vcontrol.clone()),
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

  pub fn def(&mut self, v: Value, name: String) {
    if self.current_ref().word_table.is_none() {
      self.current().word_table = Some(self.pool.get_word_table());
    }
    let word_def = self.pool.get_word_def(v);
    self.current().word_table.as_mut().unwrap().insert(name, word_def);
  }
  
  pub fn push_quoted(&mut self, v: Value) {
    let mut wrapper = self.pool.get_vstack(1);
    wrapper.container.stack.push(v);
    self.current().stack.push(Value::Stack(wrapper));
  }

  pub fn is_high_tide(&self) -> bool {
    if let Some(ref cranks) = self.current_ref().cranks {
      if let Some(crank) = cranks.first() {
        if crank.modulo == 0 && crank.base != 0 {
          return true }}}
    false
  }
  pub fn evalf_high_tide(&self) -> bool {
    if let Some(cranks) = &self.current_ref().cranks {
      if let Some(crank) = cranks.first() {
        if crank.modulo == 1 || crank.base == 1 {
          return false }}}
    true
  }

  fn evalword(mut self, v: Value, local_family: &mut Family, always_evalf: bool, try_eval: bool, cranking: bool) -> (Self, RecurseControl) {
    loop {
      let Some(family_stack) = self.family.pop() else {
        // Assuming family stack has failed
        if try_eval {
          if let Some(ref wt) = self.current_ref().word_table {
            if let Some(wd) = wt.get(&v.vword_ref().str_word) {
              //self = self.evalstack_new(new_word_def, None, Some(&v), false);
              //self.pool.add_val(v);
              let new_word_def = wd.clone();
              break (self, RecurseControl::Def(new_word_def, v))
            }
          }
        }
        if self.current().isfalias(&v) {
          if always_evalf || self.evalf_high_tide() {
            let (mut new_self, result) = self.get_evalf_val(Some(&v));
            new_self.pool.add_val(v);
            let Some((wd, defstack)) = result else { break (new_self, RecurseControl::None) };
            break (new_self, RecurseControl::Evalf(wd, defstack))
          }
          self.pool.add_val(v);
          break (self, RecurseControl::None)
        }
        self.push_quoted(v);
        if cranking {
          if try_eval { self.current().inc_crank() }
          else {
            let (new_self, result) = self.get_crank_val();
            let Some((wd, defstack)) = result else { break (new_self, RecurseControl::None) };
            break (new_self, RecurseControl::Crank(wd, defstack))
          }
        }

        break (self, RecurseControl::None)
      };

      let family_container = &family_stack.vstack_ref().container;
      if try_eval {
        if let Some(ref wt) = family_container.word_table {
          if let Some(wd) = wt.get(&v.vword_ref().str_word) {
            // self = self.evalstack_new(wd.clone(), None, Some(&v), false);
            // self.pool.add_val(v);
            let new_word_def = wd.clone();
            self.family.push(family_stack);
            break (self, RecurseControl::Def(new_word_def, v))
          }
        }
      }
      if family_container.isfalias(&v) {
        if always_evalf || self.evalf_high_tide() {
          let (mut new_self, result) = self.get_evalf_val(Some(&v));
          new_self.pool.add_val(v);
          let Some((wd, defstack)) = result else { break (new_self, RecurseControl::None) };
          break (new_self, RecurseControl::Evalf(wd, defstack))
        }
        self.pool.add_val(v);
        break (self, RecurseControl::None)
      }
      local_family.push(family_stack);
    }
  }

  fn eval_value(mut self, v: Value, callword: Option<&Value>, local_family: &mut Family, force_eval: bool, cranking: bool) -> (Self, RecurseControl) {
    match &v {
      Value::Word(_) => {
        let high_tide = self.is_high_tide();
        return self.evalword(v, local_family, force_eval, high_tide || force_eval, cranking);
      },
      Value::Error(_) => panic!("VError on stack"),
      Value::FLLib(vfllib) => {
        let fllib = vfllib.fllib.clone();
        if self.is_high_tide() || force_eval {
          self.pool.add_val(v);
          if cranking { self.current().inc_crank() }
          self = fllib(self, callword.clone())
        } else {
          self.push_quoted(v);
          if cranking {
            let (new_self, result) = self.get_crank_val();
            let Some((wd, defstack)) = result else { return (new_self, RecurseControl::None) };
            return (new_self, RecurseControl::Crank(wd, defstack))
          }
        }
      },
      Value::Custom(_) => {
        self.push_quoted(v);
        if cranking {
          if self.is_high_tide() || force_eval { self.current().inc_crank() }
          else {
            let (new_self, result) = self.get_crank_val();
            let Some((wd, defstack)) = result else { return (new_self, RecurseControl::None) };
            return (new_self, RecurseControl::Crank(wd, defstack))
          }
        }
      },
      Value::Control(VControl::Eval) => {
        if self.is_high_tide() || force_eval {
          self.pool.add_val(v);
          if cranking { self.current().inc_crank() }
          let (new_self, result) = self.get_evalf_val(callword);
          let Some((wd, defstack)) = result else { return (new_self, RecurseControl::None) };
          return (new_self, RecurseControl::Evalf(wd, defstack))
        } else {
          self.push_quoted(v);
          let (new_self, result) = self.get_crank_val();
          let Some((wd, defstack)) = result else { return (new_self, RecurseControl::None) };
          return (new_self, RecurseControl::Crank(wd, defstack))
        }
      },
      Value::Control(VControl::Return) => {
        if force_eval { return (self, RecurseControl::Return) }
      },
      _ => {
        self.current().stack.push(v);
        if cranking {
          if self.is_high_tide() || force_eval { self.current().inc_crank() }
          else {
            let (new_self, result) = self.get_crank_val();
            let Some((wd, defstack)) = result else { return (new_self, RecurseControl::None) };
            return (new_self, RecurseControl::Crank(wd, defstack))
          }
        }
      },
    }
    (self, RecurseControl::None)
  }

  #[allow(unused_mut)]
  fn evalstack(mut self, wd: WordDef, defstack: Option<Stack>, callword: Option<&Value>, crank_first: bool) -> Self {
    let mut family_stack_pushes = 0;
    let mut is_macro = if wd.is_stack() {
      family_stack_pushes += 1;
      self.family.push(wd.clone()); false
    } else { true };
    let mut local_family = self.pool.get_family();
    let mut control: RecurseControl;

    let mut refstack = wd.value_stack_ref();
    let (mut stack, mut destructive) = if defstack.is_some() {
      (defstack.unwrap(), true)
    } else {
      (self.pool.get_stack(0), false)
    };

    let mut i = 0;
    loop {
      if if destructive { stack.len() == 0 } else { i >= refstack.len() } { break }
      let v = if destructive { stack.pop().unwrap() } else { self.value_copy(&refstack[i]) };
      let last_v = if destructive { stack.len() == 0 } else { i == refstack.len() - 1 };
      let cranking = if is_macro { crank_first && i == 0 } else { crank_first || i != 0 };
      (self, control) = self.eval_value(v, callword, &mut local_family, is_macro || i == 0, cranking);
      match control {
        RecurseControl::Evalf(wd, retstack) => {
          if last_v && false {}
          else { self = self.evalstack(wd, Some(retstack), None, false) }
          while let Some(f) = local_family.pop() { self.family.push(f) }
        },
        RecurseControl::Crank(wd, retstack) => {
          if last_v && false {}
          else { self = self.evalstack(wd, Some(retstack), None, true) }
          while let Some(f) = local_family.pop() { self.family.push(f) }
        },
        RecurseControl::Def(wd, v) => {
          if last_v && false {}
          else {
            self = self.evalstack(wd, None, Some(&v), cranking);
            self.pool.add_val(v);
          }
          while let Some(f) = local_family.pop() { self.family.push(f) }
        },
        RecurseControl::None => {},
        RecurseControl::Return => break,
      }
      i += 1;
    }
    self.pool.add_stack(stack);

    self.pool.add_family(local_family);
    for _ in 0..family_stack_pushes {
      if let Some(wd) = self.family.pop() {
        self.pool.add_word_def(wd) }}
    self
  }

  // don't increment crank
  pub fn get_evalf_val(mut self, alias: Option<&Value>) -> (Self, Option<(WordDef, Stack)>) {
    let Some(mut needseval) = self.current().stack.pop() else { return (self.eval_error("EMPTY STACK", alias), None) };
    let mut defstack = self.pool.get_stack(needseval.value_stack_ref().len());
    while let Some(v) = needseval.value_stack().pop() { defstack.push(v) }
    let wd = self.pool.get_word_def(needseval);
    (self, Some((wd, defstack)))
  }

  fn get_crank_val(mut self) -> (Self, Option<(WordDef, Stack)>) {
    let mut cur_v = self.pop_cur();
    let cur = cur_v.metastack_container();

    let cranks = match cur.cranks.as_mut() {
      None => { return (self.push_cur(cur_v), None) },
      Some(c) => c,
    };
    let high_tide = |c: &Crank| c.modulo == 0 && c.base != 0;
    let cindex: Option<usize> = cranks.iter().position(high_tide);
    let Some(cidx) = cindex else {
      cur.inc_crank();
      return (self.push_cur(cur_v), None)
    };
    let fixedindex: isize = cur.stack.len() as isize - 1 - cidx as isize;
    if fixedindex < 0 {
      cur.inc_crank();
      return (self.push_cur(cur_v).eval_error("CRANK TOO DEEP", None), None);
    }
    let mut needseval = cur.stack.remove(fixedindex as usize);
    let mut defstack = self.pool.get_stack(needseval.value_stack_ref().len());
    while let Some(v) = needseval.value_stack().pop() { defstack.push(v) }
    let wd = self.pool.get_word_def(needseval);
    (self.push_cur(cur_v), Some((wd, defstack)))
  }

  pub fn evalf(self, alias: Option<&Value>) -> Self {
    let (new_self, result) = self.get_evalf_val(alias);
    let Some((wd, defstack)) = result else { return new_self };
    new_self.evalstack(wd, Some(defstack), None, false)
  }
  pub fn crank(self) -> Self {
    let (new_self, result) = self.get_crank_val();
    let Some((wd, defstack)) = result else { return new_self };
    new_self.evalstack(wd, Some(defstack), None, true)
  }

  pub fn eval(mut self, v: Value) -> Self {
    let cur = self.current_ref();
    if cur.isfalias(&v) {
      self = match &cur.cranks {
        None => self.evalf(Some(&v)),
        Some(cranks) => 'crank: {
          if let Some(crank) = cranks.first() {
            if crank.base == 1 || crank.modulo == 1 { break 'crank self }
          }
          self.evalf(Some(&v))
        },
      };
      self.pool.add_val(v);
      return self;
    }
    self.push_quoted(v);
    self.crank()
  }
}
