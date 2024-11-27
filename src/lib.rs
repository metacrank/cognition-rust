#![allow(dead_code)]
#![allow(unused_macros)]

#[macro_use]
pub mod macros;
pub mod tree;
pub mod pool;
pub mod math;
pub mod builtins;
pub mod serde;

use crate::pool::Pool;
use crate::macros::*;
use crate::math::Math;

pub use cognition_macros::*;

use std::any::Any;
use std::collections::{HashSet, HashMap, BTreeMap};
use std::default::Default;
// use std::error::Error;
use std::fmt::Display;
use std::io::{Write, stdout};
use std::sync::Arc;

pub use ::serde::{Serialize, Deserialize};
pub use erased_serde;

pub type CognitionFunction = fn(CognitionState, Option<&Value>) -> CognitionState;
pub type AddWordsFn = unsafe extern fn(&mut CognitionState, &Library);

pub type DeserializeFn<T> = fn(&mut dyn erased_serde::Deserializer, &mut CognitionState) -> erased_serde::Result<Box<T>>;
pub type CognitionDeserializeResult = Result<CognitionState, (CognitionState, Box<dyn Display>)>;
pub type CogStateDeserializeFn = fn(&str, bool, CognitionState) -> CognitionDeserializeResult;
pub type CogLibsDeserializeFn = fn(&str, CognitionState) -> CognitionDeserializeResult;
pub type CogStateSerializeFn = fn(&CognitionState, &mut dyn Write) -> Result<(), Box<dyn Display>>;
pub type CogLibsSerializeFn = fn(&Option<ForeignLibraries>, &mut dyn Write) -> Result<(), Box<dyn Display>>;
pub type CogValueSerializeFn = fn(&Value, &mut dyn Write) -> Result<(), Box<dyn Display>>;
pub type CogValueDeserializeFn = fn(&str, &mut CognitionState) -> Result<Value, Box<dyn Display>>;

pub type Functions = Vec<CognitionFunction>;
pub type Stack = Vec<Value>;
pub type Cranks = Vec<Crank>;
pub type Strings = Vec<String>;
pub type Faliases = HashSet<String>;
pub type WordDef = Arc<Value>;
pub type Family = Vec<WordDef>;
pub type WordTable = HashMap<String, Arc<Value>>;
pub type ForeignLibraries = HashMap<String, ForeignLibrary>;

pub type Library = Arc<FLLibLibrary>;

pub struct FLLibLibrary {
  pub lib: libloading::Library,
  pub lib_name: String,
  pub lib_path: String,
}

pub struct ForeignLibrary {
  pub registry: BTreeMap<String, DeserializeFn<dyn Custom>>,
  pub functions: Vec<CognitionFunction>,
  pub library: Library,
}

pub struct MathBorrower {
  family: Family,
  cur: Option<Value>
}

impl MathBorrower {
  pub fn math(&self) -> &Math {
    if let Some(ref cur) = self.cur {
      return &cur.vstack_ref().container.math.as_ref().unwrap()
    }
    &self.family.last().unwrap().vstack_ref().container.math.as_ref().unwrap()
  }
  fn replenish(mut self, state: &mut CognitionState) {
    if let Some(cur) = self.cur {
      state.stack.push(cur)
    }
    while let Some(f) = self.family.pop() {
      state.family.push(f)
    }
    state.pool.add_family(self.family);
  }
}

pub struct Serde {
  serdes: Vec<SerdeDescriptor>,
  serializers: Vec<SerializerDescriptor>,
  deserializers: Vec<DeserializerDescriptor>
}

impl Serde {
  pub fn new() -> Self {
    Self {
      serdes: Vec::new(),
      serializers: Vec::new(),
      deserializers: Vec::new()
    }
  }
}

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

pub trait Custom: Any + erased_serde::Serialize {
  fn printfunc(&self, f: &mut dyn Write);
  // implemented as Box::new(self.clone()) in classes that implement Clone
  fn copyfunc(&self, state: &mut CognitionState) -> Box<dyn Custom>;

  // usually not implemented by users; use the cognition::custom proc macro
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
  fn custom_type_name(&self) -> &'static str;
}

pub trait CustomTypeData {
  fn custom_type_name() -> &'static str;
  fn deserialize_fn() -> DeserializeFn<dyn Custom>;
}

/// Useful Custom type
#[derive(Serialize, Deserialize)]
pub struct Void {}

#[custom]
impl Custom for Void {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(void)");
  }
  fn copyfunc(&self, _: &mut CognitionState) -> Box<dyn Custom> {
    Box::new(Void{})
  }
}

#[derive(Clone, Serialize, Deserialize)]
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
  pub fn isfalias(&self, v: &Value) -> bool {
    match &self.faliases {
      None => false,
      Some(f) => f.contains(&v.vword_ref().str_word),
    }
  }

  pub fn default_faliases() -> Option<Faliases> {
    let mut f = Faliases::with_capacity(DEFAULT_FALIASES_SIZE);
    f.insert(String::from("f"));
    f.insert(String::from("ing"));
    Some(f)
  }
}

#[derive(Serialize, Deserialize)]
pub struct VWord {
  pub str_word: String,
}
pub struct VStack {
  pub container: Container,
}
pub struct VMacro {
  pub macro_stack: Stack,
}
#[derive(Serialize, Deserialize)]
pub struct VErrorLoc {
  filename: String,
  line: String,
  column: String,
}
#[derive(Serialize, Deserialize)]
pub struct VError {
  error: String,
  str_word: Option<String>,
  loc: Option<VErrorLoc>
}
pub struct VFLLib {
  fllib: CognitionFunction,
  pub str_word: Option<String>,
  pub library: Option<Library>,
  pub key: u32,
}
#[derive(Serialize)]
pub struct VCustom {
  pub custom: Box<dyn Custom>,
}
#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum VControl {
  Eval,
  Return,
  Ghost,
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
    VError{ error, str_word: Some(str_word), loc: None }
  }
  pub fn with_error(error: String) -> VError {
    VError{ error, str_word: None, loc: None }
  }
  pub fn with_capacity(capacity: usize) -> VError {
    Self::with_error(String::with_capacity(capacity))
  }
}
impl VFLLib {
  pub fn with_fn(fllib: CognitionFunction) -> VFLLib {
    VFLLib{ fllib, str_word: None, library: None, key: 0}
  }
  pub fn with_nop() -> VFLLib {
    Self::with_fn(builtins::misc::cog_nop)
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
    let mut f = stdout();
    self.fprint(&mut f, end);
    if let Err(e) = f.flush() {
      let _ = std::io::stderr().write(format!("{e}").as_bytes()); }
  }
  pub fn fprint(&self, f: &mut dyn Write, end: &'static str) {
    match self {
      Self::Word(vword) => {
        fwrite_check!(f, b"'");
        vword.str_word.fprint_pretty(f);
        fwrite_check!(f, b"'");
      },
      Self::Stack(vstack) => {
        fwrite_check!(f, b"[ ");
        for v in vstack.container.stack.iter() { v.fprint(f, " "); }
        fwrite_check!(f, b"]");
      },
      Self::Macro(vmacro) => {
        fwrite_check!(f, b"( ");
        for v in vmacro.macro_stack.iter() { v.fprint(f, " "); }
        fwrite_check!(f, b")");
      },
      Self::Error(verror) => {
        match verror.loc {
          Some(ref loc) => {
            loc.filename.fprint_pretty(f);
            fwrite_check!(f, b":");
            loc.line.fprint_pretty(f);
            fwrite_check!(f, b":");
            loc.column.fprint_pretty(f);
            fwrite_check!(f, b":");
          },
          None => {
            fwrite_check!(f, b"(none):");
          },
        }
        match verror.str_word {
          Some(ref word) => {
            fwrite_check!(f, b"'");
            word.fprint_pretty(f);
            fwrite_check!(f, b"':");
          }
          None => {
            fwrite_check!(f, b"(none):");
          }
        }
        fwrite_check!(f, RED);
        verror.error.fprint_pretty(f);
        fwrite_check!(f, COLOR_RESET);
      },
      Self::FLLib(vfllib) => {
        match &vfllib.str_word {
          Some(s) => {
            s.fprint_pretty(f);
          },
          None => {
            fwrite_check!(f, HBLK);
            fwrite_check!(f, b"FLLIB");
            fwrite_check!(f, COLOR_RESET);
          },
        }
      },
      Self::Custom(vcustom) => {
        vcustom.custom.printfunc(f);
      },
      Self::Control(vcontrol) => {
        fwrite_check_pretty!(f, GRN);
        match vcontrol {
          VControl::Eval   => { fwrite_check!(f, b"eval"); },
          VControl::Return => { fwrite_check!(f, b"return"); },
          VControl::Ghost => { fwrite_check!(f, b"ghost"); },
        }
        fwrite_check!(f, COLOR_RESET);
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

pub struct ParserLoc {
  pub filename: Option<String>,
  pub pos: Option<(usize, usize)>
}

#[derive(Serialize, Deserialize)]
pub struct Parser {
  source: Option<String>,
  filename: Option<String>,
  i: usize,
  c: Option<char>,
  line: usize,
  column: usize,
}

impl Parser {
  pub fn new(source: Option<String>, filename: Option<String>) -> Parser {
    Self::with_loc(source, ParserLoc{ filename, pos: None })
  }
  pub fn with_loc(source: Option<String>, loc: ParserLoc) -> Parser {
    if source.is_none() { return Parser{ source: None, filename: None, i: 0, c: None, line: 1, column: 0 } }
    let c = match source.as_ref().unwrap().get(..) { Some(st) => st.chars().next(), None => None };
    let (line, column) = if let Some(pos) = loc.pos { (pos.0, pos.1) } else { (1, 0) };
    Parser{ source, filename: loc.filename, i: 0, c, line, column }
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

  pub fn reset(&mut self, source: String, filename: Option<String>) {
    self.column = 0;
    self.line = 1;
    self.c = match source.get(..) { Some(st) => st.chars().next(), None => None };
    self.i = 0;
    self.source = Some(source);
    self.filename = filename;
  }

  pub fn source(&mut self) -> Option<String> {
    self.source.take()
  }
  pub fn filename(&mut self) -> Option<String> {
    self.filename.take()
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
  pub args: Stack,
  pub fllibs: Option<ForeignLibraries>,
  pub builtins: Functions,
  pub serde: Serde,
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

macro_rules! evalstack_recurse_setup {
  ($self:ident,$is_macro:ident,$wd:ident,$wdn:ident,$local_family:ident,$legacy_family:ident,$family_stack_pushes:ident,$new_family_member:ident,$refstack:ident,$i:ident,$word_holder:ident) => {
    let mut local_family_iter = $local_family.into_iter();
    if $new_family_member { if local_family_iter.next().is_some() { $family_stack_pushes -= 1 } }
    if let Some(member) = local_family_iter.next() {
      if $legacy_family.is_none() {
        $legacy_family = Some($self.pool.get_family());
      }
      $legacy_family.as_mut().unwrap().push(member);
      $local_family = local_family_iter.collect();
      $legacy_family.as_mut().unwrap().append(&mut $local_family);
    } else { $local_family = local_family_iter.collect() }
    $self.pool.add_word_def($wd);
    $wd = $wdn;
    $is_macro = if $wd.is_stack() {
      let mut found = false;
      if let Some(lf) = &mut $legacy_family {
        if let Some(l) = lf.last() {
          if Arc::ptr_eq(l, &$wd) {
            $self.family.push(lf.pop().unwrap());
            found = true
          }
        }
      }
      if !found {
        if $self.family.iter().any(|x| Arc::ptr_eq(x, &$wd)) {
          found = true;
        }
      }
      if !found {
        $family_stack_pushes += 1;
        $new_family_member = true;
        $self.family.push($wd.clone());
      }
      false
    } else {
      $new_family_member = false;
      true
    };
    $refstack = $wd.value_stack_ref(); // frees previous refstack
    $i = usize::MAX;
    if let Some(wh) = $word_holder.take() {
      $self.pool.add_val(wh)
    }
  }
}
macro_rules! evalstack_recurse {
  ($self:ident,$is_macro:ident,$wd:ident,$wdn:ident,$local_family:ident,$legacy_family:ident,$family_stack_pushes:ident,$new_family_member:ident,$refstack:ident,$i:ident,$destructive:ident,$callword:ident,$crank_first:ident,DESTRUCTIVE,$stack:ident,$retstack:ident,$word_holder:ident,$crnkf:expr,$last_v:ident) => {
    while let Some(f) = $self.family.pop() { $local_family.push(f) }
    if $last_v {
      evalstack_recurse_setup!($self, $is_macro, $wd, $wdn, $local_family, $legacy_family, $family_stack_pushes, $new_family_member, $refstack, $i, $word_holder);
      $self.pool.add_stack($stack);
      $stack = $retstack;
      $destructive = true;
      $callword = None;
      $crank_first = $crnkf;
    } else {
      $self = $self.evalstack($wdn, Some($retstack), None, $crnkf);
      while let Some(f) = $local_family.pop() { $self.family.push(f) }
    }
  };
  ($self:ident,$is_macro:ident,$wd:ident,$wdn:ident,$local_family:ident,$legacy_family:ident,$family_stack_pushes:ident,$new_family_member:ident,$refstack:ident,$i:ident,$destructive:ident,$callword:ident,$crank_first:ident,$callw:expr,$word_holder:ident,$crnkf:expr,$last_v:ident) => {
    if $last_v {
      evalstack_recurse_setup!($self, $is_macro, $wd, $wdn, $local_family, $legacy_family, $family_stack_pushes, $new_family_member, $refstack, $i, $word_holder);
      $destructive = false;
      $callword = None;
      $word_holder = Some($callw);
      $crank_first = $crnkf;
    } else {
      $self = $self.evalstack($wdn, None, Some(&$callw), $crnkf);
      while let Some(f) = $local_family.pop() { $self.family.push(f) }
      $self.pool.add_val($callw);
    }
  }
}
macro_rules! callword {
  ($callword:ident,$word_holder:ident) => {
    if let Some(w) = $word_holder.as_ref() { Some(w) } else { $callword }
  }
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
    Self{
      chroots: Vec::<Stack>::with_capacity(DEFAULT_STACK_SIZE),
      stack,
      family: Family::with_capacity(DEFAULT_STACK_SIZE),
      parser: None,
      exited: false,
      exit_code: None,
      args: Stack::new(),
      fllibs: None,
      builtins: Vec::with_capacity(BUILTINS_SIZE),
      serde: Serde::new(),
      pool: Pool::new()
    }
  }

  pub fn verr_loc(&mut self) -> Option<VErrorLoc> {
    if let Some(ref parser) = self.parser {
      if let Some(ref filename) = parser.filename {
        let mut loc = self.pool.get_verror_loc(filename.len());
        loc.filename.push_str(filename);
        if let Some(math) = self.current().math.take() {
          if math.base() == 1 {
            let zero = math.get_digits().get(0).expect("Math missing digits");
            if self.parser.as_ref().unwrap().line == 0 { loc.line.push(zero.clone()) }
            if self.parser.as_ref().unwrap().column == 0 { loc.column.push(zero.clone()) }
          }
          if math.base() > 1 {
            if self.parser.as_ref().unwrap().line <= isize::MAX as usize {
              let line = self.parser.as_ref().unwrap().line as isize;
              if let Ok(line) = math.itos(line, self) {
                loc.line.push_str(&line);
                self.pool.add_string(line);
              }
            }
            if self.parser.as_ref().unwrap().column <= isize::MAX as usize {
              let column = self.parser.as_ref().unwrap().column as isize;
              if let Ok(column) = math.itos(column, self) {
                loc.column.push_str(&column);
                self.pool.add_string(column);
              }
            }
          }
          self.current().math = Some(math)
        }
        return Some(loc)
      }
    }
    None
  }

  pub fn eval_error_mut(&mut self, e: &'static str, w: Option<&Value>) {
    let mut verror = self.pool.get_verror(e.len());
    verror.error.push_str(e);
    verror.str_word = match w {
      None => None,
      Some(v) => Some(self.string_copy(&v.vword_ref().str_word)),
    };
    verror.loc = self.verr_loc();
    if let None = self.current_ref().err_stack {
      let temp = self.pool.get_stack(1);
      self.current().err_stack = Some(temp);
    }
    let estack: &mut Stack = &mut self.current().err_stack.as_mut().unwrap();
    estack.push(Value::Error(verror));
  }

  pub fn eval_error(mut self, e: &'static str, w: Option<&Value>) -> Self {
    self.eval_error_mut(e, w);
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

  pub fn parser_get_next(&mut self) -> Option<Value> {
    let Some(mut parser) = self.parser.take() else { return None };
    let retval = parser.get_next(self);
    self.parser = Some(parser);
    retval
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
    self.contain_copy_attributes(old, new);
  }

  pub fn contain_copy_attributes(&mut self, old: &Container, new: &mut Container) {
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
      new.faliases = Some(self.pool.get_faliases(faliases.capacity()));
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
      new.word_table = Some(self.pool.get_word_table(word_table.capacity()));
      for (key, word_def) in word_table.iter() {
        new.word_table.as_mut().unwrap().insert(self.string_copy(key), word_def.clone());
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
        if let Some(ref loc) = verror.loc {
          let mut new_loc = self.pool.get_verror_loc(loc.filename.len());
          new_loc.filename.push_str(&loc.filename);
          new_loc.line.push_str(&loc.line);
          new_loc.column.push_str(&loc.column);
          new_verror.loc = Some(new_loc);
        }
        Value::Error(new_verror)
      },
      Value::FLLib(vfllib) => {
        let mut new_vfllib = self.pool.get_vfllib(vfllib.fllib);
        if let Some(ref s) = vfllib.str_word {
          new_vfllib.str_word = Some(self.string_copy(s));
        }
        if let Some(ref library) = vfllib.library {
          new_vfllib.library = Some(library.clone());
        }
        new_vfllib.key = vfllib.key;
        Value::FLLib(new_vfllib)
      },
      Value::Custom(vcustom) => Value::Custom({
        VCustom::with_custom(vcustom.custom.copyfunc(self))
      }),
      Value::Control(vcontrol) => Value::Control(vcontrol.clone()),
    }
  }

  pub fn default_faliases(&mut self) -> Option<Faliases> {
    let mut f = self.pool.get_faliases(DEFAULT_FALIASES_SIZE);
    f.insert(String::from("f"));
    f.insert(String::from("ing"));
    Some(f)
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

  pub fn get_math(&mut self) -> Option<MathBorrower> {
    let mut borrower = MathBorrower {
      family: self.pool.get_family(),
      cur: None
    };
    while let Some(f) = self.family.pop() {
      borrower.family.push(f);
      if borrower.family.last().unwrap().vstack_ref().container.math.is_some() {
        return Some(borrower)
      }
    }
    if self.current_ref().math.is_some() {
      borrower.cur = Some(self.pop_cur());
      return Some(borrower)
    }
    while let Some(f) = borrower.family.pop() {
      self.family.push(f)
    }
    None
  }

  pub fn with_math(mut self, m: MathBorrower) -> Self {
    m.replenish(&mut self);
    self
  }

  pub fn set_math(&mut self, m: MathBorrower) { m.replenish(self) }

  pub fn def(&mut self, v: Value, name: String) {
    if self.current_ref().word_table.is_none() {
      self.current().word_table = Some(self.pool.get_word_table(DEFAULT_WORD_TABLE_SIZE));
    }
    let word_def = self.pool.get_word_def(v);
    if let Some(v) = self.current().word_table.as_mut().unwrap().insert(name, word_def) {
      self.pool.add_word_def(v);
    }
  }

  pub unsafe fn load_fllib(&mut self, lib_name: &String, filename: &String) -> Option<&'static str> {
    let Ok(lib) = libloading::Library::new(filename) else { return Some("INVALID FILENAME") };
    let fllib_add_words = match lib.get::<libloading::Symbol<AddWordsFn>>(b"add_words\0") {
      Ok(f) => f.into_raw(),
      Err(_) => return Some("INVALID FLLIB")
    };
    let lib_name = self.string_copy(lib_name);
    let lib_path = self.string_copy(filename);
    let fllib_library = FLLibLibrary{ lib, lib_name, lib_path };
    let library = Arc::new(fllib_library);
    fllib_add_words(self, &library);
    None
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
        if self.is_high_tide() || force_eval { return (self, RecurseControl::Return) }
        else {
          self.push_quoted(v);
          let (new_self, result) = self.get_crank_val();
          let Some((wd, defstack)) = result else { return (new_self, RecurseControl::None) };
          return (new_self, RecurseControl::Crank(wd, defstack))
        }
      },
      Value::Control(VControl::Ghost) => {}
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

  #[inline(always)]
  fn evalstack(mut self, mut wd: WordDef, defstack: Option<Stack>, mut callword: Option<&Value>, mut crank_first: bool) -> Self {
    let mut family_stack_pushes = 0;
    let mut new_family_member = false;
    let mut is_macro = if wd.is_stack() {
      family_stack_pushes += 1;
      new_family_member = true;
      self.family.push(wd.clone());
      false
    } else { true };
    let mut local_family = self.pool.get_family();
    let mut legacy_family: Option<Family> = None;
    let mut control: RecurseControl;

    let mut refstack = wd.value_stack_ref();
    let (mut stack, mut destructive) = if defstack.is_some() {
      (defstack.unwrap(), true)
    } else {
      (self.pool.get_stack(0), false)
    };
    let mut word_holder: Option<Value> = None;

    let mut i: usize = 0;
    loop {
      if if destructive { stack.len() == 0 } else { i >= refstack.len() } { break }
      let v = if destructive { stack.pop().unwrap() } else { self.value_copy(&refstack[i]) };
      let last_v = if destructive { stack.len() == 0 } else { i == refstack.len() - 1 };
      let cranking = if is_macro { crank_first && i == 0 } else { crank_first || i != 0 };
      (self, control) = self.eval_value(v, callword!(callword, word_holder), &mut local_family, is_macro || i == 0, cranking);

      match control {
        RecurseControl::Evalf(wdn, retstack) => {
          evalstack_recurse!(self, is_macro, wd, wdn, local_family, legacy_family, family_stack_pushes, new_family_member,
                             refstack, i, destructive, callword, crank_first, DESTRUCTIVE, stack, retstack, word_holder, false, last_v); },
        RecurseControl::Crank(wdn, retstack) => {
          evalstack_recurse!(self, is_macro, wd, wdn, local_family, legacy_family, family_stack_pushes, new_family_member,
                             refstack, i, destructive, callword, crank_first, DESTRUCTIVE, stack, retstack, word_holder, true, last_v); },
        RecurseControl::Def(wdn, v) => {
          evalstack_recurse!(self, is_macro, wd, wdn, local_family, legacy_family, family_stack_pushes, new_family_member,
                             refstack, i, destructive, callword, crank_first, v, word_holder, cranking, last_v); },
        RecurseControl::None => {},
        RecurseControl::Return => break,
      }
      if self.exited { break }
      i = i.wrapping_add(1);
    }
    if let Some(wh) = word_holder { self.pool.add_val(wh) }
    self.pool.add_stack(stack);
    while let Some(f) = local_family.pop() { self.family.push(f) }
    self.pool.add_family(local_family);
    if let Some(mut lf) = legacy_family {
      while let Some(f) = lf.pop() { self.family.push(f) }
      self.pool.add_family(lf)
    }
    for _ in 0..family_stack_pushes {
      if let Some(wd) = self.family.pop() {
        self.pool.add_word_def(wd) }}
    self
  }

  pub fn get_evalf_val(mut self, alias: Option<&Value>) -> (Self, Option<(WordDef, Stack)>) {
    let Some(mut needseval) = self.current().stack.pop() else { return (self.eval_error("EMPTY STACK", alias), None) };
    let mut defstack = self.pool.get_stack(needseval.value_stack_ref().len());
    while let Some(v) = needseval.value_stack().pop() { defstack.push(v) }
    let wd = self.pool.get_word_def(needseval);
    (self, Some((wd, defstack)))
  }

  fn get_crank_val(mut self) -> (Self, Option<(WordDef, Stack)>) {
    let cur = self.current();

    let cranks = match cur.cranks.as_mut() {
      None => { return (self, None) },
      Some(c) => c,
    };
    let high_tide = |c: &Crank| c.modulo == 0 && c.base != 0;
    let cindex: Option<usize> = cranks.iter().position(high_tide);
    let Some(cidx) = cindex else {
      cur.inc_crank();
      return (self, None)
    };
    let fixedindex: isize = cur.stack.len() as isize - 1 - cidx as isize;
    if fixedindex < 0 {
      cur.inc_crank();
      return (self.eval_error("CRANK TOO DEEP", None), None);
    }
    let mut needseval = cur.stack.remove(fixedindex as usize);
    let mut defstack = self.pool.get_stack(needseval.value_stack_ref().len());
    while let Some(v) = needseval.value_stack().pop() { defstack.push(v) }
    let wd = self.pool.get_word_def(needseval);
    (self, Some((wd, defstack)))
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
