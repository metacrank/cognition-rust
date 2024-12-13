#![allow(dead_code)]
#![allow(unused_macros)]

pub const VERSION: &'static str = "0.3.10";

#[macro_use]
pub mod macros;
pub mod tree;
pub mod pool;
pub mod math;
pub mod builtins;
pub mod serde;

pub use crate::macros::*;
pub use crate::math::*;
pub use crate::pool::*;

pub use crate::serde::*;
pub use ::serde::{Serialize, Deserialize, Serializer, Deserializer};
pub use erased_serde;

pub use cognition_macros::*;

use std::any::Any;
use std::collections::{HashSet, HashMap, BTreeMap};
use std::default::Default;
// use std::error::Error;
use std::fmt::Display;
use std::io::{stdout, IsTerminal, Write};
use std::sync::Arc;

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

#[derive(Clone)]
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
    let mut out = stdout();
    if out.is_terminal() {
      fwrite_check_pretty!(out, self.as_bytes());
    } else {
      fwrite_check!(out, self.as_bytes());
    }
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

  // optional, only called when dropping a value, so it's useful for
  // defining specific pool-aware freeing instructions for a custom type
  fn custom_pool(&mut self, pool: &mut Pool) -> CustomPoolPackage;

  // usually not implemented by users; use the cognition::custom proc macro
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
  fn custom_type_name(&self) -> &'static str;
}

pub trait CustomTypeData {
  fn custom_type_name() -> &'static str;
  fn deserialize_fn() -> DeserializeFn<dyn Custom>;
}

pub trait CustomCast {
  unsafe fn as_custom<T: CustomTypeData>(self) -> Option<Box<T>>;
  unsafe fn as_custom_ref<T: CustomTypeData>(&self) -> Option<&T>;
  unsafe fn as_custom_mut<T: CustomTypeData>(&mut self) -> Option<&mut T>;
}

impl CustomCast for Box<dyn Custom> {
  unsafe fn as_custom<T>(self) -> Option<Box<T>>
  where T: CustomTypeData
  {
    if <T>::custom_type_name() != (*self).custom_type_name() { return None }
    let raw = Box::into_raw(self);
    Some(Box::from_raw(raw.cast::<T>()))
  }
  unsafe fn as_custom_ref<T>(&self) -> Option<&T>
  where T: CustomTypeData
  {
    if <T>::custom_type_name() != (*self).custom_type_name() { return None }
    Some(&*(&**self as *const dyn Custom).cast::<T>())
  }
  unsafe fn as_custom_mut<T>(&mut self) -> Option<&mut T>
  where T: CustomTypeData
  {
    if <T>::custom_type_name() != (*self).custom_type_name() { return None }
    Some(&mut *(&mut **self as *mut dyn Custom).cast::<T>())
  }
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

/// Unique crank system control value
#[derive(Serialize, Deserialize)]
pub struct Ghost {}

#[custom]
impl Custom for Ghost {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"ghost");
  }
  fn copyfunc(&self, _: &mut CognitionState) -> Box<dyn Custom> {
    Box::new(Ghost{})
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
  pub word_table: Option<WordTable>,
  pub faliases: Option<Faliases>,
  pub delims: Option<String>,
  pub ignored: Option<String>,
  pub singlets: Option<String>,
  pub dflag: bool,
  pub iflag: bool,
  pub sflag: bool,
}

impl Default for Container {
  fn default() -> Self {
    Container {
      stack: Stack::new(),
      err_stack: None,
      cranks: None,
      math: None,
      word_table: None,
      faliases: None,
      delims: None,
      ignored: None,
      singlets: None,
      dflag: false,
      iflag: true,
      sflag: false,
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
  pub fn inc_crank(&mut self) {
    let Some(cranks) = &mut self.cranks else { return };
    for crank in cranks {
      crank.modulo += 1;
      if crank.modulo >= crank.base {
        crank.modulo = 0;
      }
    }
  }
  pub fn dec_crank(&mut self) {
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
    f.insert(String::from(""));
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
  pub filename: String,
  pub line: String,
  pub column: String,
}
#[derive(Serialize, Deserialize)]
pub struct VError {
  pub error: String,
  pub str_word: Option<String>,
  pub loc: Option<VErrorLoc>
}
pub struct VFLLib {
  pub fllib: CognitionFunction,
  pub str_word: Option<String>,
  pub library: Option<Library>,
  pub key: u32,
}
#[derive(Serialize)]
pub struct VCustom {
  pub custom: Box<dyn Custom>,
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
    let is_terminal = f.is_terminal();
    self.fprint(&mut f, end, is_terminal);
    if let Err(e) = f.flush() {
      let _ = std::io::stderr().write(format!("{e}").as_bytes()); }
  }

  pub fn fprint(&self, f: &mut dyn Write, end: &'static str, term: bool) {
    match self {
      Self::Word(vword) => {
        fwrite_check!(f, b"'");
        if term { vword.str_word.fprint_pretty(f); }
        else { fwrite_check!(f, vword.str_word.as_bytes()); }
        fwrite_check!(f, b"'");
      },
      Self::Stack(vstack) => {
        fwrite_check!(f, b"[ ");
        for v in vstack.container.stack.iter() { v.fprint(f, " ", term); }
        fwrite_check!(f, b"]");
      },
      Self::Macro(vmacro) => {
        fwrite_check!(f, b"( ");
        for v in vmacro.macro_stack.iter() { v.fprint(f, " ", term); }
        fwrite_check!(f, b")");
      },
      Self::Error(verror) => {
        match verror.loc {
          Some(ref loc) => {
            if term { loc.filename.fprint_pretty(f); }
            else { fwrite_check!(f, loc.filename.as_bytes()); }
            fwrite_check!(f, b":");
            if term { loc.line.fprint_pretty(f); }
            else { fwrite_check!(f, loc.line.as_bytes()); }
            fwrite_check!(f, b":");
            if term { loc.column.fprint_pretty(f); }
            else { fwrite_check!(f, loc.column.as_bytes()); }
            fwrite_check!(f, b":");
          },
          None => {
            fwrite_check!(f, b"(none):");
          },
        }
        match verror.str_word {
          Some(ref word) => {
            fwrite_check!(f, b"'");
            if term { word.fprint_pretty(f); }
            else { fwrite_check!(f, word.as_bytes()); }
            fwrite_check!(f, b"':");
          }
          None => {
            fwrite_check!(f, b"(none):");
          }
        }
        if term {
          fwrite_check!(f, RED);
          verror.error.fprint_pretty(f);
          fwrite_check!(f, COLOR_RESET);
        } else {
          fwrite_check!(f, verror.error.as_bytes());
        }
      },
      Self::FLLib(vfllib) => {
        match &vfllib.str_word {
          Some(s) => {
            if term { s.fprint_pretty(f); }
            else { fwrite_check!(f, s.as_bytes()); }
          },
          None => {
            if term {
              fwrite_check!(f, HBLK);
              fwrite_check!(f, b"FLLIB");
              fwrite_check!(f, COLOR_RESET);
            } else {
              fwrite_check!(f, b"FLLIB");
            }
          },
        }
      },
      Self::Custom(vcustom) => {
        vcustom.custom.printfunc(f);
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
  pub fn vword_ref(&self) -> &Box<VWord> { return_value_type!(self, Value::Word(v), v) }
  pub fn vstack_ref(&self) -> &Box<VStack> { return_value_type!(self, Value::Stack(v), v) }
  pub fn vmacro_ref(&self) -> &Box<VMacro> { return_value_type!(self, Value::Macro(v), v) }
  pub fn verror_ref(&self) -> &Box<VError> { return_value_type!(self, Value::Error(v), v) }
  pub fn vfllib_ref(&self) -> &Box<VFLLib> { return_value_type!(self, Value::FLLib(v), v) }
  pub fn vcustom_ref(&self) -> &VCustom { return_value_type!(self, Value::Custom(v), v) }
  pub fn vword_mut(&mut self) -> &mut Box<VWord> { return_value_type!(self, Value::Word(v), v) }
  pub fn vstack_mut(&mut self) -> &mut Box<VStack> { return_value_type!(self, Value::Stack(v), v) }
  pub fn vmacro_mut(&mut self) -> &mut Box<VMacro> { return_value_type!(self, Value::Macro(v), v) }
  pub fn verror_mut(&mut self) -> &mut Box<VError> { return_value_type!(self, Value::Error(v), v) }
  pub fn vfllib_mut(&mut self) -> &mut Box<VFLLib> { return_value_type!(self, Value::FLLib(v), v) }
  pub fn vcustom_mut(&mut self) -> &mut VCustom { return_value_type!(self, Value::Custom(v), v) }

  pub fn is_word(&self) -> bool { is_value_type!(self, Value::Word(_)) }
  pub fn is_stack(&self) -> bool { is_value_type!(self, Value::Stack(_)) }
  pub fn is_macro(&self) -> bool { is_value_type!(self, Value::Macro(_)) }
  pub fn is_error(&self) -> bool { is_value_type!(self, Value::Error(_)) }
  pub fn is_fllib(&self) -> bool { is_value_type!(self, Value::FLLib(_)) }
  pub fn is_custom(&self) -> bool { is_value_type!(self, Value::Custom(_)) }
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
  parse_delim: bool
}

impl Parser {
  pub fn new(source: Option<String>, filename: Option<String>) -> Parser {
    Self::with_loc(source, ParserLoc{ filename, pos: None })
  }
  pub fn with_loc(source: Option<String>, loc: ParserLoc) -> Parser {
    if source.is_none() { return Parser{ source: None, filename: None, i: 0, c: None, line: 1, column: 0, parse_delim: false } }
    let c = match source.as_ref().unwrap().get(..) { Some(st) => st.chars().next(), None => None };
    let (line, column) = if let Some(pos) = loc.pos { (pos.0, pos.1) } else { (1, 0) };
    Parser{ source, filename: loc.filename, i: 0, c, line, column, parse_delim: false }
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
    self.parse_delim = false;
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
    if !skipped && !self.parse_delim {
      v.str_word.push(c);
      self.next();
      self.parse_delim = state.issinglet(c);
      if state.issinglet(c) { return Some(Value::Word(v)) }
    } else {
      self.parse_delim = false;
    }
    while let Some(c) = self.c {
      if state.isdelim(c) { break }
      v.str_word.push(c);
      self.next();
      if state.issinglet(c) {
        self.parse_delim = true;
        break
      }
    }
    Some(Value::Word(v))
  }

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

  pub fn get_next(&mut self, state: &mut CognitionState) -> Option<Value> {
    let skipped = self.skip_ignored(&state);
    self.parse_word(skipped, state)
  }
}

pub enum CognitionControl {
  Eval,
  Return,
  None
}

impl CognitionControl {
  pub fn eval(&mut self) { *self = Self::Eval }
  pub fn ret(&mut self) { *self = Self::Return  }
  pub fn clear(&mut self) { *self = Self::None }
  pub fn is_eval(&self) -> bool { if let Self::Eval = self { true } else { false } }
  pub fn is_return(&self) -> bool { if let Self::Return = self { true } else { false } }
  pub fn is_none(&self) -> bool { if let Self::None = self { true } else { false } }
}

pub struct CognitionState {
  pub chroots: Vec<Stack>, // meta metastack
  pub stack: Stack, // metastack
  pub family: Family, // for reuse
  pub parser: Option<Parser>,
  pub control: CognitionControl,
  pub exited: bool,
  pub exit_code: Option<String>,
  pub args: Stack, //
  pub fllibs: Option<ForeignLibraries>,
  pub builtins: Functions,
  pub serde: Serde,
  pub pool: Pool,
}

impl CognitionState {
  pub fn new(stack: Stack) -> Self {
    Self{
      chroots: Vec::<Stack>::with_capacity(DEFAULT_STACK_SIZE),
      stack,
      family: Family::with_capacity(DEFAULT_STACK_SIZE),
      parser: None,
      control: CognitionControl::None,
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

  pub fn string_copy(&mut self, s: &str) -> String {
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
    if let Some(ref delims) = old.singlets {
      new.singlets = Some(self.string_copy(singlets));
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
      })
    }
  }

  pub fn foreign_library_copy(&mut self, lib: &ForeignLibrary) -> ForeignLibrary {
    let mut registry = BTreeMap::new();
    for (k, v) in lib.registry.iter() {
      registry.insert(self.string_copy(k), v.clone());
    }
    let mut functions = Vec::with_capacity(lib.functions.len());
    for f in lib.functions.iter() { functions.push(f.clone()); }
    ForeignLibrary{ registry, functions, library: lib.library.clone() }
  }

  pub fn default_faliases(&mut self) -> Option<Faliases> {
    let mut f = self.pool.get_faliases(DEFAULT_FALIASES_SIZE);
    f.insert(String::from(""));
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

  pub fn add_constant(&mut self, name: &str, v: Value) {
    let mut vstack = self.pool.get_vstack(1);
    vstack.container.stack.push(v);
    let name = self.string_copy(name);
    self.def(Value::Stack(vstack), name)
  }

  pub fn add_const_value(&mut self, name: &str, v: Value) {
    let mut vstack = self.pool.get_vstack(1);
    vstack.container.stack.push(v);
    self.add_constant(name, Value::Stack(vstack))
  }

  pub fn add_const_custom(&mut self, name: &str, custom: Box<dyn Custom>) {
    self.add_const_value(name, Value::Custom(VCustom::with_custom(custom)))
  }

  pub fn add_const_word(&mut self, name: &str, s: &str) {
    let mut vword = self.pool.get_vword(s.len());
    vword.str_word.push_str(s);
    self.add_const_value(name, Value::Word(vword))
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

  pub fn ensure_quoted(&mut self, s: &mut Stack) {
    for val in s.iter_mut() {
      if !(val.is_stack() || val.is_macro()) {
        let new_val = self.pool.get_vstack(1);
        let old_val = std::mem::replace(val, Value::Stack(new_val));
        val.vstack_mut().container.stack.push(old_val);
      }
    }
  }

  pub fn eval_inside(mut self, mut v: Value, callword: Option<&Value>) -> Self {
    let mut vstack = self.pool.get_vstack(v.vstack_ref().container.stack.len());
    std::mem::swap(&mut vstack.container.stack, &mut v.vstack_mut().container.stack);
    self.stack.push(v);
    let wd = WordDef::new(Value::Stack(vstack));
    self.evalstack(wd, callword, true)
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

  pub fn evalstack(mut self, mut wd: WordDef, callword: Option<&Value>, crank_first: bool) -> Self {
    let mut eval = CognitionEval::setup(&mut self, &mut wd, crank_first);
    while !eval.is_empty() {
      let (state, recurse) = eval.eval_value(self, callword);
      (self, eval) = eval.eval_recurse(state, recurse, callword);
      if self.exited { break }
      if self.control.is_return() {
        if eval.kill_return() { self.control.clear(); }
        break
      }
    }
    eval.decommission(&mut self);
    self.pool.add_word_def(wd);
    self
  }

  pub fn get_evalf_val(&mut self, alias: Option<&Value>) -> Option<WordDef> {
    let Some(needseval) = self.current().stack.pop() else {
      self.eval_error_mut("EMPTY STACK", alias);
      return None
    };
    let wd = self.pool.get_word_def(needseval);
    Some(wd)
  }

  pub fn get_crank_val(&mut self, w: Option<&Value>) -> Option<WordDef> {
    let cur = self.current();

    let cranks = match cur.cranks.as_mut() {
      None => { return None },
      Some(c) => c,
    };
    let high_tide = |c: &Crank| c.modulo == 0 && c.base != 0;
    let cindex: Option<usize> = cranks.iter().position(high_tide);
    let Some(cidx) = cindex else {
      cur.inc_crank();
      return None
    };
    let fixedindex: isize = cur.stack.len() as isize - 1 - cidx as isize;
    if fixedindex < 0 {
      cur.inc_crank();
      self.eval_error_mut("CRANK TOO DEEP", w);
      return None;
    }
    let needseval = cur.stack.remove(fixedindex as usize);
    let wd = self.pool.get_word_def(needseval);
    Some(wd)
  }

  pub fn evalf(mut self, alias: Option<&Value>) -> Self {
    match self.get_evalf_val(alias) {
      Some(wd) => self.evalstack(wd, None, false),
      None => self
    }
  }
  pub fn crank(mut self, w: Option<&Value>) -> Self {
    match self.get_crank_val(w) {
      Some(wd) => self.evalstack(wd, None, true),
      None => self
    }
  }

  pub fn eval(mut self, v: Value, w: Option<&Value>) -> Self {
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
    self.crank(w)
  }
}

enum EvalStack {
  Refstack(WordDef, usize),
  Reverse(Stack)
}

impl EvalStack {
  pub fn from(wd: &mut WordDef, state: &mut CognitionState) -> Self {
    let Some(v_mut) = Arc::get_mut(wd) else {
      return Self::Refstack(wd.clone(), 0)
    };
    let mut stack = state.pool.get_stack(v_mut.value_stack_ref().len());
    while let Some(v) = v_mut.value_stack().pop() { stack.push(v) }
    Self::Reverse(stack)
  }
  pub fn advance(&mut self, state: &mut CognitionState) {
    match self {
      Self::Refstack(_, i) => *i += 1,
      Self::Reverse(stack) => { stack.pop().map(|v| state.pool.add_val(v)); },
    }
  }
  pub fn get_next(&mut self, state: &mut CognitionState) -> Option<Value> {
    match self {
      Self::Refstack(a, i) => {
        let v = a.value_stack_ref().get(*i);
        *i += 1;
        v.map(|v| state.value_copy(v))
      },
      Self::Reverse(stack) => stack.pop()
    }
  }
  pub fn peek(&self) -> Option<&Value> {
    match self {
      Self::Refstack(a, i) => a.value_stack_ref().get(*i),
      Self::Reverse(stack) => stack.last()
    }
  }
  pub fn last(&self) -> bool {
    match self {
      Self::Refstack(a, i) => i + 1 == a.value_stack_ref().len(),
      Self::Reverse(stack) => stack.len() <= 1
    }
  }
  pub fn decommission(self, state: &mut CognitionState) {
    match self {
      Self::Refstack(w, _) => state.pool.add_word_def(w),
      Self::Reverse(stack) => state.pool.add_stack(stack)
    }
  }
}

enum EvalRecurse {
  Def(WordDef),
  Evalf(WordDef),
  Crank(WordDef),
  None,
  Ghost
}

struct CognitionEval {
  stack: EvalStack,
  first_v: bool,
  local_family: Family,
  legacy_family: Option<Family>,
  is_macro: bool,
  callword_owned: Option<Value>,
  crank_first: bool,
  kill_return: bool,
  family_stack_pushes: usize,
}

impl CognitionEval {
  pub fn setup(state: &mut CognitionState, wd: &mut WordDef, crank_first: bool) -> CognitionEval {
    let stack = EvalStack::from(wd, state);
    let is_macro = wd.is_macro();
    if !is_macro { state.family.push(wd.clone()) }
    let family_stack_pushes = if is_macro { 0 } else { 1 };
    let local_family = state.pool.get_family();
    Self{ stack, first_v: true, local_family, legacy_family: None, is_macro,
          callword_owned: None, crank_first, kill_return: false, family_stack_pushes }
  }
  fn recurse(mut self, mut wd: WordDef, crank_first: bool, is_def: bool, state: &mut CognitionState) -> Self {
    let stack = EvalStack::from(&mut wd, state);
    let is_macro = wd.is_macro();
    let remove_len = self.local_family.len().min(self.family_stack_pushes);
    self.family_stack_pushes -= remove_len;
    for wdef in self.local_family.drain(..remove_len) { state.pool.add_word_def(wdef) }
    if self.local_family.len() > 0 {
      if self.legacy_family.is_none() {
        self.legacy_family = Some(state.pool.get_family())
      }
      self.legacy_family.as_mut().unwrap().append(&mut self.local_family);
    }
    if !is_macro && state.family.last().map_or(true, |w| !Arc::ptr_eq(&wd, w)) {
      state.family.push(wd.clone());
      self.family_stack_pushes += 1;
    }
    let callword_owned = if is_def { self.stack.get_next(state) } else { self.callword_owned };
    self.stack.decommission(state);
    state.pool.add_word_def(wd);
    let kill_return = is_def || self.kill_return;
    Self{ stack, first_v: true, local_family: self.local_family, legacy_family: self.legacy_family,
          is_macro, callword_owned, crank_first, kill_return, family_stack_pushes: self.family_stack_pushes }
  }
  pub fn decommission(mut self, state: &mut CognitionState) {
    while let Some(f) = self.local_family.pop() { state.family.push(f) }
    state.pool.add_family(self.local_family);
    for _ in 0..self.family_stack_pushes {
      if let Some(wdef) = state.family.pop() { state.pool.add_word_def(wdef) }
    }
    if let Some(mut lf) = self.legacy_family {
      while let Some(f) = lf.pop() { state.family.push(f) }
      state.pool.add_family(lf)
    }
    if let Some(callword) = self.callword_owned {
      state.pool.add_val(callword)
    }
    self.stack.decommission(state);
  }

  pub fn kill_return(&self) -> bool { self.kill_return }
  pub fn is_empty(&self) -> bool { self.stack.peek().is_none() }

  fn force_eval(&self) -> bool { self.is_macro || self.first_v }
  fn cranking(&self) -> bool {
    if self.is_macro { self.first_v && self.crank_first }
    else { self.crank_first || !self.first_v }
  }
  fn callword<'a>(&'a self, callword: Option<&'a Value>) -> Option<&Value> {
    if self.callword_owned.is_some() { self.callword_owned.as_ref() }
    else { callword }
  }

  fn eval_word_in_current(&mut self, mut state: CognitionState, callword: Option<&Value>) -> (CognitionState, EvalRecurse) {
    let v = self.stack.peek().unwrap();
    if state.is_high_tide() || self.force_eval() {
      if let Some(ref wt) = state.current().word_table {
        if let Some(wd) = wt.get(&v.vword_ref().str_word) {
          let new_word_def = wd.clone();
          return (state, EvalRecurse::Def(new_word_def))
        }
      }
    }
    if state.current().isfalias(v) {
      if !self.is_macro && (self.first_v || state.evalf_high_tide()) {
        if let Some(wd) = state.get_evalf_val(Some(v)) {
          self.stack.advance(&mut state);
          return (state, EvalRecurse::Evalf(wd))
        }
      }
      self.stack.advance(&mut state);
      return (state, EvalRecurse::None)
    }
    let v = self.stack.get_next(&mut state).unwrap();
    state.push_quoted(v);
    // attempt to crank
    if self.cranking() {
      if state.is_high_tide() || self.force_eval() { state.current().inc_crank() }
      else {
        // this value was not cranked; get another one
        if let Some(wd) = state.get_crank_val(self.callword(callword)) {
          return (state, EvalRecurse::Crank(wd))
        }
      }
    }
    (state, EvalRecurse::None)
  }

  fn eval_word(&mut self, mut state: CognitionState, callword: Option<&Value>) -> (CognitionState, EvalRecurse) {
    let v = self.stack.peek().unwrap();
    while let Some(family_stack) = state.family.pop() {
      let family_container = &family_stack.vstack_ref().container;
      if state.is_high_tide() || self.force_eval() {
        if let Some(ref wt) = family_container.word_table {
          if let Some(wd) = wt.get(&v.vword_ref().str_word) {
            let new_word_def = wd.clone();
            state.family.push(family_stack);
            return (state, EvalRecurse::Def(new_word_def))
          }
        }
      }
      if family_container.isfalias(v) {
        if !self.is_macro && (self.first_v || state.evalf_high_tide()) {
          if let Some(wd) = state.get_evalf_val(Some(v)) {
            self.stack.advance(&mut state);
            state.family.push(family_stack);
            return (state, EvalRecurse::Evalf(wd))
          }
        }
        self.stack.advance(&mut state);
        state.family.push(family_stack);
        return (state, EvalRecurse::None)
      }
      self.local_family.push(family_stack);
    }
    self.eval_word_in_current(state, callword)
  }

  fn eval_fllib(&mut self, mut state: CognitionState, callword: Option<&Value>) -> (CognitionState, EvalRecurse) {
    let v = self.stack.peek().unwrap();
    let fllib = v.vfllib_ref().fllib.clone();
    if state.is_high_tide() || self.force_eval() {
      if self.cranking() { state.current().inc_crank() }
      state = fllib(state, self.callword(callword));
      if state.control.is_eval() {
        state.control.clear();
        if let Some(wd) = state.get_evalf_val(self.callword(callword)) {
          self.stack.advance(&mut state);
          return (state, EvalRecurse::Evalf(wd))
        }
      }
      self.stack.advance(&mut state);
    } else {
      let v = self.stack.get_next(&mut state).unwrap();
      state.push_quoted(v);
      if self.cranking() {
        if let Some(wd) = state.get_crank_val(self.callword(callword)) {
          return (state, EvalRecurse::Crank(wd))
        }
      }
    }
    (state, EvalRecurse::None)
  }

  fn eval_custom(&mut self, mut state: CognitionState, callword: Option<&Value>) -> (CognitionState, EvalRecurse) {
    let v = self.stack.get_next(&mut state).unwrap();
    if v.vcustom_ref().custom.as_any().is::<Ghost>() {
      return (state, EvalRecurse::Ghost)
    }
    state.push_quoted(v);
    if self.cranking() {
      if state.is_high_tide() || self.force_eval() {
        state.current().inc_crank()
      } else if let Some(wd) = state.get_crank_val(self.callword(callword)) {
        return (state, EvalRecurse::Crank(wd))
      }
    }
    (state, EvalRecurse::None)
  }

  fn eval_push_to_stack(&mut self, mut state: CognitionState, callword: Option<&Value>) -> (CognitionState, EvalRecurse) {
    let v = self.stack.get_next(&mut state).unwrap();
    state.current().stack.push(v);
    if self.cranking() {
      if state.is_high_tide() || self.force_eval() {
        state.current().inc_crank()
      } else if let Some(wd) = state.get_crank_val(self.callword(callword)) {
        return (state, EvalRecurse::Crank(wd))
      }
    }
    (state, EvalRecurse::None)
  }

  pub fn eval_value(&mut self, state: CognitionState, callword: Option<&Value>) -> (CognitionState, EvalRecurse) {
    let Some(v) = self.stack.peek() else { return (state, EvalRecurse::None) };
    match v {
      Value::Word(_) => self.eval_word(state, callword),
      Value::Error(_) => panic!("VError on stack"),
      Value::FLLib(_) => self.eval_fllib(state, callword),
      Value::Custom(_) => self.eval_custom(state, callword),
      _ => self.eval_push_to_stack(state, callword)
    }
  }
  pub fn eval_recurse(mut self, mut state: CognitionState, recurse: EvalRecurse, callword: Option<&Value>) -> (CognitionState, Self) {
    (state, self) = match recurse {
      EvalRecurse::Evalf(wdn) => {
        self.first_v = false;
        if self.is_empty() {
          let new_eval = self.recurse(wdn, false, false, &mut state);
          (state, new_eval)
        } else {
          (state.evalstack(wdn, self.callword(callword), false), self)
        }
      },
      EvalRecurse::Crank(wdn) => {
        self.first_v = false;
        if self.is_empty() {
          let new_eval = self.recurse(wdn, true, false, &mut state);
          (state, new_eval)
        } else {
          (state.evalstack(wdn, self.callword(callword), true), self)
        }
      },
      EvalRecurse::Def(wdn) => {
        if self.stack.last() {
          let cranking = self.cranking();
          let new_eval = self.recurse(wdn, cranking, true, &mut state);
          (state, new_eval)
        } else {
          let w = self.stack.peek();
          state = state.evalstack(wdn, w, self.cranking());
          while let Some(f) = self.local_family.pop() { state.family.push(f) }
          self.stack.advance(&mut state);
          self.first_v = false;
          (state, self)
        }
      },
      EvalRecurse::None => {
        self.first_v = false;
        (state, self)
      },
      EvalRecurse::Ghost => (state, self)
    };
    while let Some(f) = self.local_family.pop() { state.family.push(f) }
    (state, self)
  }
}
