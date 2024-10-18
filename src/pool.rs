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
  faliasess: Option<Vec<Faliases>>
}

trait DisregardPool {
  fn pnew(_p: &mut Pool) -> Self;
  fn pdrop(_p: &mut Pool, _v: Self);
}

impl<T> DisregardPool for Vec<T> {
  fn pnew(_p: &mut Pool) -> Self {
    Self::with_capacity(DEFAULT_STACK_SIZE)
  }
  fn pdrop(_p: &mut Pool, _v: Self) {}
}

macro_rules! pool_insert {
  ($v:expr,$capacity:expr,$self:ident,$tree:expr,$treetype:ty,$getstack:expr) => {
    let mut ptree = $tree.take();
    if ptree.is_none() { ptree = Some(ITree::<$treetype>::new()) }
    let tree = ptree.as_mut().unwrap();
    tree.insert($capacity, $v, $getstack, $self);
    $tree = ptree;
  }
}
macro_rules! pool_push {
  ($v:expr,$self:ident,$stack:expr,$getstack:expr) => {
    let mut pstack = $stack.take();
    if pstack.is_none() { pstack = Some($getstack) }
    let stack = pstack.as_mut().unwrap();
    stack.push($v);
    $stack = pstack;
  }
}
macro_rules! pool_insert_val {
  ($v:expr,$capacity:expr,$self:ident,$tree:expr) => {
    pool_insert!($v,$capacity,$self,$tree,Value,Self::get_stack_for_pool);
  }
}
macro_rules! pool_push_val {
  ($v:expr,$self:ident,$stack:expr) => {
    pool_push!($v,$self,$stack,$self.get_stack_for_pool());
  }
}
macro_rules! pool_remove_val {
  ($self:ident,$tree:expr,$capacity:expr,$letpattern:pat,$retval:tt,$mod:block) => {
    if let Some(mut tree) = $tree.take() {
      if let Some(retval) = tree.remove_at_least($capacity, Self::add_stack, $self) {
        let $letpattern = retval else { panic!("Bad value type in pool tree") };
        $mod;
        $tree = Some(tree);
        return $retval;
      }
      $tree = Some(tree);
    }
  }
}

macro_rules! pool_pop_val {
  ($stack:expr,$letpattern:pat,$retval:tt,$mod:block) => {
    if let Some(stack) = &mut $stack {
      if let Some(v) = stack.pop() {
        let $letpattern = v else { panic!("Bad value type in pool vec") };
        $mod;
        return $retval;
      }
    }
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
      faliasess: None,
    }
  }

  pub fn print(&self) {
    if let Some(ref vwords) = self.vwords {
      println!("vwords:");
      vwords.print();
    }
    if let Some(ref vstacks) = self.vstacks {
      println!("vstacks:");
      vstacks.print();
    }
    if let Some(ref vmacros) = self.vmacros {
      println!("vmacros:");
      vmacros.print();
    }
    if let Some(ref verrors) = self.verrors {
      println!("verrors:");
      verrors.print();
    }
    if let Some(ref vcustoms) = self.vcustoms {
      println!("vcustoms: {}", vcustoms.len());
    }
    if let Some(ref vfllibs) = self.vfllibs {
      println!("vfllibs: {}", vfllibs.len());
    }

    if let Some(ref stacks) = self.stacks {
      println!("stacks:");
      stacks.print();
    }
    if let Some(ref strings) = self.strings {
      println!("strings:");
      strings.print();
    }
    if let Some(ref stringss) = self.stringss {
      println!("stringss:");
      stringss.print();
    }
    if let Some(ref crankss) = self.crankss {
      println!("crankss:");
      crankss.print();
    }
    if let Some(ref word_tables) = self.word_tables {
      println!("word_tables: {}", word_tables.len());
    }
    if let Some(ref families) = self.families {
      println!("families: {}", families.len());
    }
  }

  pub fn add_val(&mut self, v: Value) {
    match v {
      Value::Word(vword)     => self.add_vword(vword),
      Value::Stack(vstack)   => self.add_vstack(vstack),
      Value::Macro(vmacro)   => self.add_vmacro(vmacro),
      Value::Error(verror)   => self.add_verror(verror),
      Value::Custom(vcustom) => self.add_vcustom(vcustom),
      Value::FLLib(vfllib)   => self.add_vfllib(vfllib),
    }
  }
  pub fn add_vword(&mut self, vword: Box<VWord>) {
    pool_insert_val!(Value::Word(vword), vword.str_word.capacity(), self, self.vwords);
  }
  pub fn add_vstack(&mut self, vstack: Box<VStack>) {
    pool_insert_val!(Value::Stack(vstack), vstack.container.stack.capacity(), self, self.vstacks);
  }
  pub fn add_vmacro(&mut self, vmacro: Box<VMacro>) {
    pool_insert_val!(Value::Macro(vmacro), vmacro.macro_stack.capacity(), self, self.vstacks);
  }
  pub fn add_verror(&mut self, verror: Box<VError>) {
    pool_insert_val!(Value::Error(verror), verror.error.capacity(), self, self.verrors);
  }
  pub fn add_vcustom(&mut self, mut vcustom: Box<VCustom>) {
    vcustom.custom = Box::new(Void{});
    pool_push_val!(Value::Custom(vcustom), self, self.vcustoms);
  }
  pub fn add_vfllib(&mut self, vfllib: Box<VFLLib>) {
    pool_push_val!(Value::FLLib(vfllib), self, self.vfllibs);
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
  pub fn add_faliases(&mut self, f: Faliases) {
    pool_push!(f, self, self.faliasess, Vec::<Faliases>::with_capacity(DEFAULT_STACK_SIZE));
  }

  pub fn add_def(&mut self, definition: (Option<String>, Option<WordDef>)) {
    self.add_string(definition.0.unwrap());
    let Some(WordDef::Val(val)) = definition.1 else { panic!("Pool::add_def(): definition lost") };
    self.add_val(val);
  }

  pub fn get_vword(&mut self, capacity: usize) -> Box<VWord> {
    pool_remove_val!(self, self.vwords, capacity, Value::Word(mut vword), vword, {
      vword.str_word.clear();
    });
    Box::new(VWord::with_capacity(capacity))
  }
  pub fn get_vstack(&mut self, capacity: usize) -> Box<VStack> {
    pool_remove_val!(self, self.vstacks, capacity, Value::Stack(mut vstack), vstack, {
      let container = &mut vstack.container;
      while let Some(v) = container.stack.pop() {
        self.add_val(v);
      }
      if container.err_stack.is_some() {
        self.add_stack(container.err_stack.take().unwrap());
      }
      if container.cranks.is_some() {
        self.add_cranks(container.cranks.take().unwrap());
      }
      if container.faliases.is_some() {
        self.add_faliases(container.faliases.take().unwrap());
      }
      if container.delims.is_some() {
        self.add_string(container.delims.take().unwrap());
      }
      if container.ignored.is_some() {
        self.add_string(container.ignored.take().unwrap());
      }
      if container.singlets.is_some() {
        self.add_string(container.singlets.take().unwrap());
      }
      container.dflag = false;
      container.iflag = true;
      container.sflag = true;
      container.dependent = false;
      if container.word_table.is_some() {
        self.add_word_table(container.word_table.take().unwrap());
      }
    });
    Box::new(VStack::with_container(Container::with_stack(self.get_stack(capacity))))
  }
  pub fn get_vmacro(&mut self, capacity: usize) -> Box<VMacro> {
    pool_remove_val!(self, self.vmacros, capacity, Value::Macro(mut vmacro), vmacro, {
      while let Some(v) = vmacro.macro_stack.pop() {
        self.add_val(v);
      }
    });
    Box::new(VMacro::with_capacity(capacity))
  }
  pub fn get_verror(&mut self, capacity: usize) -> Box<VError> {
    pool_remove_val!(self, self.verrors, capacity, Value::Error(mut verror), verror, {
      verror.error.clear();
      if verror.str_word.is_some() {
        self.add_string(verror.str_word.take().unwrap());
      }
    });
    Box::new(VError::with_capacity(capacity))
  }
  pub fn get_vcustom(&mut self, custom: Box<dyn Custom + Send>) -> Box<VCustom> {
    pool_pop_val!(self.vcustoms, Value::Custom(mut vcustom), vcustom, {
      vcustom.custom = custom;
    });
    Box::new(VCustom { custom })
  }
  pub fn get_vfllib(&mut self, f: CognitionFunction) -> Box<VFLLib> {
    pool_pop_val!(self.vfllibs, Value::FLLib(mut vfllib), vfllib, {
      vfllib.fllib = f;
    });
    Box::new(VFLLib::with_fn(f))
  }

  pub fn get_stack(&mut self, capacity: usize) -> Stack {
    if let Some(mut tree) = self.stacks.take() {
      if let Some(mut retval) = tree.remove_at_least(capacity, Vec::<Stack>::pdrop, self) {
        while let Some(v) = retval.pop() {
          self.add_val(v);
        }
        self.stacks = Some(tree);
        return retval;
      }
      self.stacks = Some(tree);
    }
    Stack::with_capacity(capacity)
  }
  pub fn get_stack_for_pool(&mut self) -> Stack {
    self.get_stack(DEFAULT_STACK_SIZE)
  }
  pub fn get_string(&mut self, capacity: usize) -> String {
    if let Some(mut tree) = self.strings.take() {
      if let Some(mut retval) = tree.remove_at_least(capacity, Self::add_strings, self) {
        retval.clear();
        self.strings = Some(tree);
        return retval;
      }
      self.strings = Some(tree);
    }
    String::with_capacity(capacity)
  }
  pub fn get_strings(&mut self, capacity: usize) -> Strings {
    if let Some(mut tree) = self.stringss.take() {
      if let Some(mut retval) = tree.remove_at_least(capacity, Vec::<Strings>::pdrop, self) {
        while let Some(s) = retval.pop() {
          self.add_string(s);
        }
        self.stringss = Some(tree);
        return retval;
      }
      self.stringss = Some(tree);
    }
    Strings::with_capacity(capacity)
  }
  pub fn get_strings_for_pool(&mut self) -> Strings {
    self.get_strings(DEFAULT_STACK_SIZE)
  }
  pub fn get_cranks(&mut self, capacity: usize) -> Cranks {
    if let Some(mut tree) = self.crankss.take() {
      if let Some(mut retval) = tree.remove_at_least(capacity, Vec::<Cranks>::pdrop, self) {
        retval.clear();
        self.crankss= Some(tree);
        return retval;
      }
      self.crankss = Some(tree);
    }
    Cranks::with_capacity(capacity)
  }
  pub fn get_word_table(&mut self) -> WordTable {
    if let Some(word_tables) = &mut self.word_tables {
      if let Some(mut wt) = word_tables.pop() {
        for (key, word_def) in wt.drain() {
          self.add_string(key);
          if let Some(WordDef::Val(v)) = word_def { self.add_val(v); }
        }
        return wt;
      }
    }
    WordTable::new()
  }
  pub fn get_family(&mut self) -> Family {
    if let Some(stack) = &mut self.families {
      if let Some(mut family) = stack.pop() {
        // Families should be pushed to the pool already empty, but just in case
        family.stack.clear();
        return family;
      }
    }
    Family::with_capacity(DEFAULT_STACK_SIZE)
  }
  pub fn get_faliases(&mut self) -> Faliases {
    if let Some(stack) = &mut self.faliasess {
      if let Some(mut faliases) = stack.pop() {
        for s in faliases.drain() {
          self.add_string(s);
        }
        return faliases;
      }
    }
    Faliases::with_capacity(DEFAULT_FALIASES_SIZE)
  }
}
