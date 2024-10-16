use crate::*;
use crate::tree::*;

type ITree<T> = Tree<usize, T>;
type VTree = ITree<Value>;

pub struct Pool {
  vwords: Option<VTree>,
  vstacks: Option<VTree>,
  vmacros: Option<VTree>,
  verrors: Option<VTree>,
  vcustoms: Option<Stack>,
  vfllibs: Option<Stack>,

  stacks: Option<ITree<Stack>>,
  strings: Option<ITree<String>>,
  stringss: Option<ITree<Strings>>,
  crankss: Option<ITree<Cranks>>,
  word_tables: Option<Vec<WordTable>>,
  families: Option<Vec<Family>>,

  i: i32, // to keep rust-analyzer happy for the moment
}

trait DisregardPool {
  fn pnew(_p: &mut Pool) -> Self;
}

impl<T> DisregardPool for Vec<T> {
  fn pnew(_p: &mut Pool) -> Self {
    Self::with_capacity(DEFAULT_STACK_SIZE)
  }
}

macro_rules! pool_insert {
  ($v:ident,$capacity:expr,$self:ident,$tree:expr,$treetype:ty,$getstack:expr) => {
    let mut ptree = $tree.take();
    if ptree.is_none() { ptree = Some(ITree::<$treetype>::new()) }
    let tree = ptree.as_mut().unwrap();
    tree.insert($capacity, $v, $getstack, $self);
    $tree = ptree;
  }
}
macro_rules! pool_push {
  ($v:ident,$self:ident,$stack:expr,$getstack:expr) => {
    let mut pstack = $stack.take();
    if pstack.is_none() { pstack = Some($getstack) }
    let stack = pstack.as_mut().unwrap();
    stack.push($v);
    $stack = pstack;
  }
}
macro_rules! pool_insert_val {
  ($v:ident,$capacity:expr,$self:ident,$tree:expr) => {
    pool_insert!($v,$capacity,$self,$tree,Value,Self::get_stack_for_pool);
  }
}
macro_rules! pool_push_val {
  ($v:ident,$self:ident,$stack:expr) => {
    pool_push!($v,$self,$stack,$self.get_stack_for_pool());
  }
}

impl Pool {
  pub fn new() -> Pool {
    Pool{
      vwords: None,
      vstacks: None,
      vmacros: None,
      verrors: None,
      vcustoms: None,
      vfllibs: None,

      stacks: None,
      strings: None,
      stringss: None,
      crankss: None,
      word_tables: None,
      families: None,

      i: 0
    }
  }

  pub fn add_val(&mut self, mut v: Value) {
    match &mut v {
      Value::Word(vword)     => { pool_insert_val!(v, vword.str_word.capacity(), self, self.vwords); },
      Value::Stack(vstack)   => { pool_insert_val!(v, vstack.container.stack.capacity(), self, self.vstacks); },
      Value::Macro(vmacro)   => { pool_insert_val!(v, vmacro.macro_stack.capacity(), self, self.vstacks); },
      Value::Error(verror)   => { pool_insert_val!(v, verror.error.capacity(), self, self.verrors); },
      Value::Custom(vcustom) => { vcustom.custom = Box::new(Void{});
                                  pool_push_val!(v, self, self.vcustoms); },
      Value::FLLib(_)        => { pool_push_val!(v, self, self.vfllibs); },
    }
  }
  pub fn add_stack(&mut self, s: Stack) {
    pool_insert!(s, s.capacity(), self, self.stacks, Stack, Vec::<Stack>::pnew);
  }
  pub fn add_string(&mut self, s: String) {
    pool_insert!(s, s.capacity(), self, self.strings, String, Self::get_strings_for_pool);
  }
  pub fn add_strings(&mut self, ss: Strings) {
    pool_insert!(ss, ss.capacity(), self, self.stringss, Strings, Vec::<Strings>::pnew);
  }
  pub fn add_cranks(&mut self, cs: Cranks) {
    pool_insert!(cs, cs.capacity(), self, self.crankss, Cranks, Vec::<Cranks>::pnew);
  }
  pub fn add_word_table(&mut self, wt: WordTable) {
    pool_push!(wt, self, self.word_tables, Vec::<WordTable>::with_capacity(DEFAULT_STACK_SIZE));
  }
  pub fn add_family(&mut self, f: Family) {
    pool_push!(f, self, self.families, Vec::<Family>::with_capacity(DEFAULT_STACK_SIZE));
  }

  pub fn get_vword(&mut self, capacity: usize) -> Value {
    Value::Word(Box::new(VWord::with_capacity(capacity)))
  }
  pub fn get_vstack(&mut self, capacity: usize) -> Value {
    if let Some(mut tree) = self.vstacks.take() {
      if let Some(mut retval) = tree.remove_at_least(capacity, Self::add_stack, self) {
        let Value::Stack(vstack) = &mut retval else { panic!("Bad value type in pool tree") };
        vstack.container.state = RefState::Recycled;
        return retval;
      }
      self.vstacks = Some(tree);
    }
    Value::Stack(Box::new(VStack::with_container(Container::with_stack(self.get_stack(capacity)))))
  }
  pub fn get_vmacro(&mut self, capacity: usize) -> Value {
    Value::Macro(Box::new(VMacro::with_capacity(capacity)))
  }
  pub fn get_verror(&mut self, capacity: usize) -> Value {
    Value::Error(Box::new(VError::with_capacity(capacity)))
  }
  pub fn get_vcustom(&mut self, custom: Box<dyn Custom>) -> Value {
    Value::Custom(Box::new(VCustom { custom }))
  }
  pub fn get_vfllib(&mut self, f: CognitionFunction) -> Value {
    Value::FLLib(Box::new(VFLLib::with_fn(f)))
  }

  pub fn get_stack(&mut self, capacity: usize) -> Stack {
    Stack::with_capacity(capacity)
  }
  pub fn get_stack_for_pool(&mut self) -> Stack {
    self.get_stack(DEFAULT_STACK_SIZE)
  }
  pub fn get_string(&mut self, capacity: usize) -> String {
    String::with_capacity(capacity)
  }
  pub fn get_strings(&mut self, capacity: usize) -> Strings {
    Strings::with_capacity(capacity)
  }
  pub fn get_strings_for_pool(&mut self) -> Strings {
    self.get_strings(DEFAULT_STACK_SIZE)
  }
  pub fn get_cranks(&mut self, capacity: usize) -> Cranks {
    Cranks::with_capacity(capacity)
  }
  pub fn get_word_table(&mut self) -> WordTable {
    WordTable::new()
  }
  pub fn get_family(&mut self) -> Family {
    if let Some(stack) = &mut self.families {
      if let Some(mut family) = stack.pop() {
        family.clear();
        return family;
      }
    }
    Family::with_capacity(DEFAULT_STACK_SIZE)
  }
}
