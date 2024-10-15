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

  stacks: Option<Tree<usize, Stack>>,
  strings: Option<Tree<usize, String>>,
  stringss: Option<Tree<usize, Strings>>,
  crankss: Option<Cranks>,
  word_tables: Option<Vec<WordTable>>,
  families: Option<Vec<Family>>,

  i: i32, // to keep rust-analyzer happy for the moment
}

macro_rules! pool_insert_val {
  ($v:ident,$capacity:expr,$self:ident,$tree:expr) => {
    let mut ptree = $tree.take();
    if ptree.is_none() { ptree = Some(VTree::new()) }
    let tree = ptree.as_mut().unwrap();
    tree.insert($capacity, $v, Self::get_stack_for_pool, $self);
    $tree = ptree;
  };
}
macro_rules! pool_push_val {
  ($v:ident,$self:ident,$stack:expr) => {
    let mut pstack = $stack.take();
    if pstack.is_none() { pstack = Some($self.get_stack_for_pool()) }
    let stack = pstack.as_mut().unwrap();
    stack.push($v);
    $stack = pstack;
  };
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
      Value::Word(vword)   => { pool_insert_val!(v, vword.str_word.capacity(), self, self.vwords); },
      Value::Stack(vstack) => { pool_insert_val!(v, vstack.container.stack.capacity(), self, self.vstacks); },
      Value::Macro(vmacro) => { pool_insert_val!(v, vmacro.macro_stack.capacity(), self, self.vstacks); },
      Value::Error(verror) => { pool_insert_val!(v, verror.error.capacity(), self, self.verrors); },
      Value::Custom(_)     => { pool_push_val!(v, self, self.vcustoms); },
      Value::FLLib(_)      => { pool_push_val!(v, self, self.vfllibs); },
    }
  }
  pub fn add_stack(&mut self, _s: Stack) {}
  pub fn add_string(&mut self, _s: String) {}
  pub fn add_strings(&mut self, _ss: Strings) {}
  pub fn add_cranks(&mut self, _cs: Cranks) {}
  pub fn add_word_table(&mut self, _wt: WordTable) {}

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
  pub fn get_vcustom(&mut self) -> Value {
    Value::Custom(Box::new(VCustom::with_void()))
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
  pub fn get_cranks(&mut self, capacity: usize) -> Cranks {
    Cranks::with_capacity(capacity)
  }
  pub fn get_word_table(&mut self) -> WordTable {
    WordTable::new()
  }
  pub fn get_family(&mut self) -> Family {
    if let Some(stack) = &mut self.families {
      if let Some(family) = stack.pop() {
        return family;
      }
    }
    Family::with_capacity(DEFAULT_STACK_SIZE)
  }
}
