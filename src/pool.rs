use crate::*;

struct Tree<T> {
  tree: Vec<T>,
  left: Option<Box<Tree<T>>>,
  right: Option<Box<Tree<T>>>,
}

impl<T> Tree<T> {
  fn new() -> Self {
    Self{ tree: Vec::<T>::new(), left: None, right: None }
  }
}

type VTree = Tree<Value>;

pub struct Pool {
  vwords: Tree<Value>,
  vstacks: Tree<Value>,
  vmacros: Tree<Value>,
  verrors: Tree<Value>,
  vcustoms: Stack,
  vfllibs: Tree<Value>,

  stacks: Tree<Stack>,
  strings: Tree<String>,
  stringss: Tree<Strings>,
  cranks: Cranks,
  word_tables: Vec<WordTable>,

  i: i32, // to keep rust-analyzer happy for the moment
}

impl Pool {
  pub fn new() -> Pool {
    Pool{
      vwords: VTree::new(),
      vstacks: VTree::new(),
      vmacros: VTree::new(),
      verrors: VTree::new(),
      vcustoms: Stack::new(),
      vfllibs: VTree::new(),

      stacks: Tree::<Stack>::new(),
      strings: Tree::<String>::new(),
      stringss: Tree::<Strings>::new(),
      cranks: Cranks::new(),
      word_tables: Vec::<WordTable>::new(),

      i: 0
    }
  }

  pub fn add_val(&mut self, v: Value) {
    match v {
      Value::Word(_vword) => { self.i = 0; }
      Value::Stack(_vstack) => { self.i = 1; }
      Value::Macro(_vmacro) => { self.i = 2; }
      Value::Error(_verr) => { self.i = 3; }
      Value::Custom(_vcustom) => { self.i = 4; }
      Value::FLLib(_vclib) => { self.i = 5; }
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
    Value::Stack(Box::new(VStack::with_capacity(capacity)))
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
}
