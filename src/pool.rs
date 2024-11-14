use crate::*;
use crate::tree::*;
use crate::math::*;

type ITree<T> = Tree<usize, T>;
type VTree = ITree<Value>;

struct Nodes {
  value: Vec<Box<Node<usize, Value>>>,
  verr_loc: Vec<Box<Node<usize, VErrorLoc>>>,
  stack: Vec<Box<Node<usize, Stack>>>,
  string: Vec<Box<Node<usize, String>>>,
  functions: Vec<Box<Node<usize, Functions>>>,
  cranks: Vec<Box<Node<usize, Cranks>>>,
  math: Vec<Box<Node<usize, Math>>>,
  digits: Vec<Box<Node<usize, Digits>>>,
  ints: Vec<Box<Node<usize, Vec<i32>>>>,
  faliases: Vec<Box<Node<usize, Faliases>>>,
  word_table: Vec<Box<Node<usize, WordTable>>>,
}

impl Nodes {
  fn new() -> Self {
    Self {
      value: Vec::new(),
      verr_loc: Vec::new(),
      stack: Vec::new(),
      string: Vec::new(),
      functions: Vec::new(),
      cranks: Vec::new(),
      math: Vec::new(),
      digits: Vec::new(),
      ints: Vec::new(),
      faliases: Vec::new(),
      word_table: Vec::new(),
    }
  }
}

pub enum CustomPool {
  Tree(ITree<Box<dyn Custom>>),
  Vec(Vec<Box<dyn Custom>>)
}

pub struct Pool {
  vwords: Option<VTree>,
  vstacks: Option<VTree>,
  vmacros: Option<VTree>,
  verrors: Option<VTree>,
  verror_locs: Option<ITree<VErrorLoc>>,
  vfllibs: Option<Stack>,
  stacks: Option<ITree<Stack>>,
  strings: Option<ITree<String>>,
  functionss: Option<ITree<Functions>>,
  crankss: Option<ITree<Cranks>>,
  maths: Option<ITree<Math>>,
  digitss: Option<ITree<Digits>>,
  intss: Option<ITree<Vec<i32>>>,
  faliasess: Option<ITree<Faliases>>,
  word_tables: Option<ITree<WordTable>>,
  word_defs: Option<Vec<WordDef>>,
  families: Option<Vec<Family>>,

  un_ops: Option<Vec<UnaryOp>>,
  bin_ops: Option<Vec<BinaryOp>>,
  str_ops: Option<Vec<StrOp>>,
  custom_ops: Option<Vec<CustomOp>>,

  nodes: Nodes,

  pub custom_pools: HashMap<String, CustomPool>,
}

trait DisregardPool {
  fn pnew(_: &mut Pool) -> Self;
  fn pdrop(_: &mut Pool, _v: Self);
}

impl<T> DisregardPool for Vec<T> {
  fn pnew(_: &mut Pool) -> Self {
    Self::with_capacity(DEFAULT_STACK_SIZE)
  }
  fn pdrop(_: &mut Pool, _v: Self) {}
}

macro_rules! pool_insert {
  ($v:expr,$capacity:expr,$self:ident,$tree:expr,$treetype:ty,$getnode:expr,$getstack:expr) => {
    let mut ptree = $tree.take();
    if ptree.is_none() { ptree = Some(ITree::<$treetype>::new()) }
    let tree = ptree.as_mut().unwrap();
    tree.insert($capacity, $v, $getnode, $getstack, $self);
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
    pool_insert!($v,$capacity,$self,$tree,Value,Self::get_value_node,Self::get_stack_for_pool);
  }
}
macro_rules! pool_push_val {
  ($v:expr,$self:ident,$stack:expr) => {
    pool_push!($v,$self,$stack,$self.get_stack_for_pool());
  }
}
macro_rules! pool_remove {
  ($self:ident,$tree:expr,$capacity:expr,$var:pat,$retval:tt,$drop_node:expr,$destruct:expr,$mod:block) => {
    if let Some(mut tree) = $tree.take() {
      if let Some($var) = tree.remove_at_least($capacity, $drop_node, $destruct, $self) {
        $mod;
        $tree = Some(tree);
        return $retval;
      }
      $tree = Some(tree);
    }
  };
}
macro_rules! pool_remove_val {
  ($self:ident,$tree:expr,$capacity:expr,$letpattern:pat,$retval:tt,$mod:block) => {
    if let Some(mut tree) = $tree.take() {
      if let Some(retval) = tree.remove_at_least($capacity, Self::add_value_node, Self::add_stack, $self) {
        let $letpattern = retval else { panic!("Bad value type in pool tree") };
        $mod;
        $tree = Some(tree);
        return $retval;
      }
      $tree = Some(tree);
    }
  }
}
macro_rules! pool_pop {
  ($stack:expr,$var:pat,$retval:tt,$mod:block) => {
    if let Some(stack) = &mut $stack {
      if let Some($var) = stack.pop() {
        $mod;
        return $retval;
      }
    }
  };
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
macro_rules! pool_pop_node {
  ($stack:expr,$letpattern:pat,$retval:tt,$key:expr,$data:expr,$mod:block) => {
    if let Some($letpattern) = $stack.pop() {
      $mod;
      $retval.key = $key;
      $retval.data = Some($data);
      $retval.height = 1;
      $retval.node_left = None;
      $retval.node_right = None;
      return $retval;
    }
  }
}
macro_rules! set_size {
  ($self:ident,$tree:expr,$capacity:expr,$default_key:expr,$create:expr,$node_new:expr,$new:expr,$gc:expr) => {
    let mut tree = if $tree.is_none() { Tree::new() } else { $tree.take().unwrap() };
    tree.set_size($capacity as usize, $default_key, $create, $node_new, $new, $gc, $self);
    $tree = Some(tree);
  };
  ($self:ident,$stack:expr,$capacity:expr,$new_item:expr,$getstack:expr) => {
    let mut stack = if $stack.is_none() { $getstack($self) } else { $stack.take().unwrap() };
    set_size!(stack, $capacity, $new_item);
    $stack = Some(stack);
  };
  ($stack:expr,$capacity:expr,$new_item:expr) => {
    let f = || $new_item();
    $stack.resize_with($capacity as usize, f);
  }
}

impl Pool {
  pub fn new() -> Pool {
    Pool{
      vwords: None,
      vstacks: None,
      vmacros: None,
      verrors: None,
      verror_locs: None,
      vfllibs: None,
      stacks: None,
      strings: None,
      functionss: None,
      crankss: None,
      maths: None,
      digitss: None,
      intss: None,
      faliasess: None,
      word_tables: None,
      word_defs: None,
      families: None,

      un_ops: None,
      bin_ops: None,
      str_ops: None,
      custom_ops: None,

      nodes: Nodes::new(),

      custom_pools: HashMap::new(),
    }
  }

  pub fn print(&self) {
    if let Some(ref vwords) = self.vwords {
      let size = vwords.size();
      if size > 0 {
        print!("{} vwords: ", size);
        vwords.print();
        println!("");
      }
    }
    if let Some(ref vstacks) = self.vstacks {
      let size = vstacks.size();
      if size > 0 {
        print!("{} vstacks: ", size);
        vstacks.print();
        println!("");
      }
    }
    if let Some(ref vmacros) = self.vmacros {
      let size = vmacros.size();
      if size > 0 {
        print!("{} vmacros: ", size);
        vmacros.print();
        println!("");
      }
    }
    if let Some(ref verrors) = self.verrors {
      let size = verrors.size();
      if size > 0 {
        print!("{} verrors: ", size);
        verrors.print();
        println!("");
      }
    }
    if let Some(ref verror_locs) = self.verror_locs {
      let size = verror_locs.size();
      if size > 0 {
        print!("{} verror_locs: ", size);
        verror_locs.print();
        println!("");
      }
    }
    if let Some(ref vfllibs) = self.vfllibs {
      if vfllibs.len() > 0 { println!("{} vfllibs", vfllibs.len()) }
    }
    if let Some(ref stacks) = self.stacks {
      let size = stacks.size();
      if size > 0 {
        print!("{} stacks: ", size);
        stacks.print();
        println!("");
      }
    }
    if let Some(ref strings) = self.strings {
      let size = strings.size();
      if size > 0 {
        print!("{} strings: ", size);
        strings.print();
        println!("");
      }
    }
    if let Some(ref functionss) = self.functionss {
      let size = functionss.size();
      if size > 0 {
        print!("{} functionss: ", size);
        functionss.print();
        println!("");
      }
    }
    if let Some(ref crankss) = self.crankss {
      let size = crankss.size();
      if size > 0 {
        print!("{} crankss: ", size);
        crankss.print();
        println!("");
      }
    }
    if let Some(ref maths) = self.maths {
      let size = maths.size();
      if size > 0 {
        print!("{} maths: ", size);
        maths.print();
        println!("");
      }
    }
    if let Some(ref digitss) = self.digitss {
      let size = digitss.size();
      if size > 0 {
        print!("{} digitss: ", size);
        digitss.print();
        println!("");
      }
    }
    if let Some(ref intss) = self.intss {
      let size = intss.size();
      if size > 0 {
        print!("{} intss: ", size);
        intss.print();
        println!("");
      }
    }
    if let Some(ref faliasess) = self.faliasess {
      let size = faliasess.size();
      if size > 0 {
        print!("{} faliasess: ", size);
        faliasess.print();
        println!("");
      }
    }
    if let Some(ref word_tables) = self.word_tables {
      let size = word_tables.size();
      if size > 0 {
        print!("{} word_tables: ", size);
        word_tables.print();
        println!("");
      }
    }
    if let Some(ref word_defs) = self.word_defs {
      if word_defs.len() > 0 { println!("{} word_defs", word_defs.len()) }
    }
    if let Some(ref families) = self.families {
      if families.len() > 0 { println!("{} families", families.len()) }
    }

    if let Some(ref un_ops) = self.un_ops {
      if un_ops.len() > 0 { println!("{} un_ops", un_ops.len()) }
    }
    if let Some(ref bin_ops) = self.bin_ops {
      if bin_ops.len() > 0 { println!("{} bin_ops", bin_ops.len()) }
    }
    if let Some(ref str_ops) = self.str_ops {
      if str_ops.len() > 0 { println!("{} str_ops", str_ops.len()) }
    }
    if let Some(ref custom_ops) = self.custom_ops {
      if custom_ops.len() > 0 { println!("{} custom_ops", custom_ops.len()) }
    }

    if self.nodes.value.len() > 0 {
      println!("{} value nodes", self.nodes.value.len());
    }
    if self.nodes.verr_loc.len() > 0 {
      println!("{} verr_loc nodes", self.nodes.verr_loc.len());
    }
    if self.nodes.stack.len() > 0 {
      println!("{} stack nodes", self.nodes.stack.len());
    }
    if self.nodes.string.len() > 0 {
      println!("{} string nodes", self.nodes.string.len());
    }
    if self.nodes.functions.len() > 0 {
      println!("{} functions nodes", self.nodes.functions.len());
    }
    if self.nodes.cranks.len() > 0 {
      println!("{} cranks nodes", self.nodes.cranks.len());
    }
    if self.nodes.math.len() > 0 {
      println!("{} math nodes", self.nodes.math.len());
    }
    if self.nodes.digits.len() > 0 {
      println!("{} digits nodes", self.nodes.digits.len());
    }
    if self.nodes.ints.len() > 0 {
      println!("{} ints nodes", self.nodes.ints.len());
    }
    if self.nodes.faliases.len() > 0 {
      println!("{} faliases nodes", self.nodes.faliases.len());
    }
    if self.nodes.word_table.len() > 0 {
      println!("{} word_table nodes", self.nodes.word_table.len());
    }

    if self.custom_pools.len() > 0 {
      print!("{} custom pools: ", self.custom_pools.len());
      let mut keys = self.custom_pools.keys();
      print!("{}", keys.next().unwrap());
      for key in keys {
        print!(", {}", key);
      }
      println!("");
    }
  }

  pub fn get_capacity(&self) -> [isize;32] {
    [
      self.vwords.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,
      self.vstacks.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,
      self.vmacros.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,
      self.verrors.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,

      self.verror_locs.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,
      self.vfllibs.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize)) as isize,
      self.stacks.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,
      self.strings.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,

      self.functionss.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,
      self.crankss.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,
      self.maths.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,
      self.digitss.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,

      self.intss.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,
      self.faliasess.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,
      self.word_tables.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize)) as isize,
      self.word_defs.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize)) as isize,

      self.families.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize)) as isize,
      self.un_ops.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize)) as isize,
      self.bin_ops.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize)) as isize,
      self.str_ops.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize)) as isize,

      self.custom_ops.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize)) as isize,
      self.nodes.value.len().min(isize::MAX as usize) as isize,
      self.nodes.verr_loc.len().min(isize::MAX as usize) as isize,
      self.nodes.stack.len().min(isize::MAX as usize) as isize,

      self.nodes.string.len().min(isize::MAX as usize) as isize,
      self.nodes.functions.len().min(isize::MAX as usize) as isize,
      self.nodes.cranks.len().min(isize::MAX as usize) as isize,
      self.nodes.math.len().min(isize::MAX as usize) as isize,

      self.nodes.digits.len().min(isize::MAX as usize) as isize,
      self.nodes.ints.len().min(isize::MAX as usize) as isize,
      self.nodes.faliases.len().min(isize::MAX as usize) as isize,
      self.nodes.word_table.len().min(isize::MAX as usize) as isize
    ]
  }

  pub fn set_capacity(&mut self, capacity: [isize;32]) {
    set_size!(self, self.vwords, capacity[0], DEFAULT_STRING_LENGTH, init_vword, Self::get_value_node, Self::get_stack_for_pool, Self::add_stack);
    set_size!(self, self.vstacks, capacity[1], DEFAULT_STACK_SIZE, init_vstack, Self::get_value_node, Self::get_stack_for_pool, Self::add_stack);
    set_size!(self, self.vmacros, capacity[2], DEFAULT_STACK_SIZE, init_vmacro, Self::get_value_node, Self::get_stack_for_pool, Self::add_stack);
    set_size!(self, self.verrors, capacity[3], DEFAULT_STRING_LENGTH, init_verror, Self::get_value_node, Self::get_stack_for_pool, Self::add_stack);

    set_size!(self, self.verror_locs, capacity[4], DEFAULT_STRING_LENGTH, init_verr_loc, Self::get_verr_loc_node, Vec::<VErrorLoc>::pnew, Vec::<VErrorLoc>::pdrop);
    set_size!(self, self.vfllibs, capacity[5], init_vfllib, Self::get_stack_for_pool);
    set_size!(self, self.stacks, capacity[6], DEFAULT_STACK_SIZE, Stack::with_capacity, Self::get_stack_node, Vec::<Stack>::pnew, Vec::<Stack>::pdrop);
    set_size!(self, self.strings, capacity[7], DEFAULT_STRING_LENGTH, String::with_capacity, Self::get_string_node, Strings::pnew, Strings::pdrop);

    set_size!(self, self.functionss, capacity[8], DEFAULT_STACK_SIZE, Functions::with_capacity, Self::get_functions_node, Vec::<Functions>::pnew, Vec::<Functions>::pdrop);
    set_size!(self, self.crankss, capacity[9], DEFAULT_STACK_SIZE, Cranks::with_capacity, Self::get_cranks_node, Vec::<Cranks>::pnew, Vec::<Cranks>::pdrop);
    set_size!(self, self.maths, capacity[10], DEFAULT_BASE, Math::with_capacity, Self::get_math_node, Vec::<Math>::pnew, Vec::<Math>::pdrop);
    set_size!(self, self.digitss, capacity[11], DEFAULT_STACK_SIZE, Digits::with_capacity, Self::get_digits_node, Vec::<Digits>::pnew, Vec::<Digits>::pdrop);

    set_size!(self, self.intss, capacity[12], DEFAULT_STACK_SIZE, Vec::<i32>::with_capacity, Self::get_ints_node, Vec::<Vec<i32>>::pnew, Vec::<Vec<i32>>::pdrop);
    set_size!(self, self.faliasess, capacity[13], DEFAULT_FALIASES_SIZE, Faliases::with_capacity, Self::get_faliases_node, Vec::<Faliases>::pnew, Vec::<Faliases>::pdrop);
    set_size!(self, self.word_tables, capacity[14], DEFAULT_WORD_TABLE_SIZE, WordTable::with_capacity, Self::get_word_table_node, Vec::<WordTable>::pnew, Vec::<WordTable>::pdrop);
    set_size!(self, self.word_defs, capacity[15], init_word_def, Vec::<WordDef>::pnew);

    set_size!(self, self.families, capacity[16], init_family, Vec::<Family>::pnew);
    set_size!(self, self.un_ops, capacity[17], UnaryOp::new, Vec::<UnaryOp>::pnew);
    set_size!(self, self.bin_ops, capacity[18], BinaryOp::new, Vec::<BinaryOp>::pnew);
    set_size!(self, self.str_ops, capacity[19], StrOp::new, Vec::<StrOp>::pnew);

    set_size!(self, self.custom_ops, capacity[20], CustomOp::new, Vec::<CustomOp>::pnew);
    set_size!(self.nodes.value, capacity[21], init_inode::<Value>);
    set_size!(self.nodes.verr_loc, capacity[22], init_inode::<VErrorLoc>);
    set_size!(self.nodes.stack, capacity[23], init_inode::<Stack>);

    set_size!(self.nodes.string, capacity[24], init_inode::<String>);
    set_size!(self.nodes.functions, capacity[25], init_inode::<Functions>);
    set_size!(self.nodes.cranks, capacity[26], init_inode::<Cranks>);
    set_size!(self.nodes.math, capacity[27], init_inode::<Math>);

    set_size!(self.nodes.digits, capacity[28], init_inode::<Digits>);
    set_size!(self.nodes.ints, capacity[29], init_inode::<Vec<i32>>);
    set_size!(self.nodes.faliases, capacity[30], init_inode::<Faliases>);
    set_size!(self.nodes.word_table, capacity[31], init_inode::<WordTable>);
  }

  pub fn add_val(&mut self, v: Value) {
    match v {
      Value::Word(vword)     => self.add_vword(vword),
      Value::Stack(vstack)   => self.add_vstack(vstack),
      Value::Macro(vmacro)   => self.add_vmacro(vmacro),
      Value::Error(verror)   => self.add_verror(verror),
      Value::FLLib(vfllib)   => self.add_vfllib(vfllib),
      _ => {}
    }
  }
  pub fn add_vword(&mut self, vword: Box<VWord>) {
    pool_insert_val!(Value::Word(vword), vword.str_word.capacity(), self, self.vwords);
  }
  pub fn add_vstack(&mut self, vstack: Box<VStack>) {
    pool_insert_val!(Value::Stack(vstack), vstack.container.stack.capacity(), self, self.vstacks);
  }
  pub fn add_vmacro(&mut self, vmacro: Box<VMacro>) {
    pool_insert_val!(Value::Macro(vmacro), vmacro.macro_stack.capacity(), self, self.vmacros);
  }
  pub fn add_verror(&mut self, verror: Box<VError>) {
    pool_insert_val!(Value::Error(verror), verror.error.capacity(), self, self.verrors);
  }
  pub fn add_verror_loc(&mut self, loc: VErrorLoc) {
    pool_insert!(loc, loc.filename.capacity(), self, self.verror_locs, VErrorLoc, Self::get_verr_loc_node, Vec::<VErrorLoc>::pnew);
  }
  pub fn add_vfllib(&mut self, vfllib: Box<VFLLib>) {
    pool_push_val!(Value::FLLib(vfllib), self, self.vfllibs);
  }

  pub fn add_stack(&mut self, s: Stack) {
    pool_insert!(s, s.capacity(), self, self.stacks, Stack, Self::get_stack_node, Vec::<Stack>::pnew);
  }
  pub fn add_string(&mut self, s: String) {
    pool_insert!(s, s.capacity(), self, self.strings, String, Self::get_string_node, Strings::pnew);
  }
  pub fn add_functions(&mut self, fs: Functions) {
    pool_insert!(fs, fs.capacity(), self, self.functionss, Functions, Self::get_functions_node, Vec::<Functions>::pnew);
  }
  pub fn add_cranks(&mut self, cs: Cranks) {
    pool_insert!(cs, cs.capacity(), self, self.crankss, Cranks, Self::get_cranks_node, Vec::<Cranks>::pnew);
  }
  pub fn add_math(&mut self, m: Math) {
    pool_insert!(m, usize::max(0, m.get_digits().capacity() * 2 - 1), self, self.maths, Math, Self::get_math_node, Vec::<Math>::pnew);
  }
  pub fn add_digits(&mut self, d: Digits) {
    pool_insert!(d, d.capacity(), self, self.digitss, Digits, Self::get_digits_node, Vec::<Digits>::pnew);
  }
  pub fn add_ints(&mut self, i: Vec<i32>) {
    pool_insert!(i, i.capacity(), self, self.intss, Vec<i32>, Self::get_ints_node, Vec::<Vec<i32>>::pnew);
  }
  pub fn add_faliases(&mut self, f: Faliases) {
    pool_insert!(f, f.capacity(), self, self.faliasess, Faliases, Self::get_faliases_node, Vec::<Faliases>::pnew);
  }
  pub fn add_word_table(&mut self, wt: WordTable) {
    pool_insert!(wt, wt.capacity(), self, self.word_tables, WordTable, Self::get_word_table_node, Vec::<WordTable>::pnew);
  }
  pub fn add_word_def(&mut self, wd: WordDef) {
    if Arc::<Value>::strong_count(&wd) != 1 { return }
    pool_push!(wd, self, self.word_defs, Vec::<WordDef>::with_capacity(DEFAULT_STACK_SIZE));
  }
  pub fn add_family(&mut self, f: Family) {
    pool_push!(f, self, self.families, Vec::<Family>::with_capacity(DEFAULT_STACK_SIZE));
  }
  pub fn add_parser(&mut self, mut p: Parser) {
    if let Some(s) = p.source() { self.add_string(s) }
    if let Some(f) = p.filename() { self.add_string(f) }
  }

  pub fn add_op(&mut self, o: Op) {
    match o {
      Op::Unary(u) => self.add_un_op(u),
      Op::Binary(b) => self.add_bin_op(b),
      Op::Str(s) => self.add_str_op(s),
      Op::Custom(c) => self.add_custom_op(c),
    }
  }
  pub fn add_un_op(&mut self, o: UnaryOp) {
    pool_push!(o, self, self.un_ops, Vec::<UnaryOp>::with_capacity(DEFAULT_STACK_SIZE));
  }
  pub fn add_bin_op(&mut self, o: BinaryOp) {
    pool_push!(o, self, self.bin_ops, Vec::<BinaryOp>::with_capacity(DEFAULT_STACK_SIZE));
  }
  pub fn add_str_op(&mut self, o: StrOp) {
    pool_push!(o, self, self.str_ops, Vec::<StrOp>::with_capacity(DEFAULT_STACK_SIZE));
  }
  pub fn add_custom_op(&mut self, o: CustomOp) {
    pool_push!(o, self, self.custom_ops, Vec::<CustomOp>::with_capacity(DEFAULT_STACK_SIZE));
  }

  pub fn add_value_node(&mut self, n: Box<Node<usize, Value>>) { self.nodes.value.push(n) }
  pub fn add_verr_loc_node(&mut self, n: Box<Node<usize, VErrorLoc>>) { self.nodes.verr_loc.push(n) }
  pub fn add_stack_node(&mut self, n: Box<Node<usize, Stack>>) { self.nodes.stack.push(n) }
  pub fn add_string_node(&mut self, n: Box<Node<usize, String>>) { self.nodes.string.push(n) }
  pub fn add_functions_node(&mut self, n: Box<Node<usize, Functions>>) { self.nodes.functions.push(n) }
  pub fn add_cranks_node(&mut self, n: Box<Node<usize, Cranks>>) { self.nodes.cranks.push(n) }
  pub fn add_math_node(&mut self, n: Box<Node<usize, Math>>) { self.nodes.math.push(n) }
  pub fn add_digits_node(&mut self, n: Box<Node<usize, Digits>>) { self.nodes.digits.push(n) }
  pub fn add_ints_node(&mut self, n: Box<Node<usize, Vec<i32>>>) { self.nodes.ints.push(n) }
  pub fn add_faliases_node(&mut self, n: Box<Node<usize, Faliases>>) { self.nodes.faliases.push(n) }
  pub fn add_word_table_node(&mut self, n: Box<Node<usize, WordTable>>) { self.nodes.word_table.push(n) }

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
      if container.math.is_some() {
        self.add_math(container.math.take().unwrap());
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
      if container.word_table.is_some() {
        self.add_word_table(container.word_table.take().unwrap());
      }
    });
    Box::new(VStack::with_container(Container::with_stack(self.get_stack(capacity))))
  }
  pub fn get_vmacro(&mut self, capacity: usize) -> Box<VMacro> {
    pool_remove_val!(self, self.vmacros, capacity, Value::Macro(mut vmacro), vmacro, {
      while let Some(v) = vmacro.macro_stack.pop() { self.add_val(v) }
    });
    Box::new(VMacro::with_capacity(capacity))
  }
  pub fn get_verror(&mut self, capacity: usize) -> Box<VError> {
    pool_remove_val!(self, self.verrors, capacity, Value::Error(mut verror), verror, {
      verror.error.clear();
      if let Some(word) = verror.str_word.take() {
        self.add_string(word);
      }
      if let Some(loc) = verror.loc.take() {
        self.add_verror_loc(loc);
      }
    });
    Box::new(VError::with_capacity(capacity))
  }
  pub fn get_verror_loc(&mut self, capacity: usize) -> VErrorLoc {
    pool_remove!(self, self.verror_locs, capacity, mut loc, loc, Self::add_verr_loc_node, Vec::<VErrorLoc>::pdrop, {
      loc.filename.clear();
      loc.line.clear();
      loc.column.clear();
    });
    VErrorLoc{ filename: self.get_string(capacity),
               line: self.get_string(4),
               column: self.get_string(4) }
  }
  pub fn get_vfllib(&mut self, f: CognitionFunction) -> Box<VFLLib> {
    pool_pop_val!(self.vfllibs, Value::FLLib(mut vfllib), vfllib, {
      if let Some(s) = vfllib.str_word.take() {
        self.add_string(s)
      }
      vfllib.fllib = f;
    });
    Box::new(VFLLib::with_fn(f))
  }

  pub fn get_stack(&mut self, capacity: usize) -> Stack {
    pool_remove!(self, self.stacks, capacity, mut stack, stack, Self::add_stack_node, Vec::<Stack>::pdrop, {
      while let Some(v) = stack.pop() { self.add_val(v) }
    });
    Stack::with_capacity(capacity)
  }
  pub fn get_stack_for_pool(&mut self) -> Stack {
    self.get_stack(DEFAULT_STACK_SIZE)
  }
  pub fn get_string(&mut self, capacity: usize) -> String {
    pool_remove!(self, self.strings, capacity, mut s, s, Self::add_string_node, Strings::pdrop, { s.clear() });
    String::with_capacity(capacity)
  }
  pub fn get_functions(&mut self, capacity: usize) -> Functions {
    pool_remove!(self, self.functionss, capacity, mut fs, fs, Self::add_functions_node, Vec::<Functions>::pdrop, { fs.clear() });
    Functions::with_capacity(capacity)
  }
  pub fn get_cranks(&mut self, capacity: usize) -> Cranks {
    pool_remove!(self, self.crankss, capacity, mut cs, cs, Self::add_cranks_node, Vec::<Cranks>::pdrop, { cs.clear() });
    Cranks::with_capacity(capacity)
  }
  pub fn get_math(&mut self, mut base: i32) -> Math {
    if base < 0 { base = 0 }
    pool_remove!(self, self.maths, base as usize, mut m, m, Self::add_math_node, Vec::<Math>::pdrop, { m.clean(self) });
    Math::new()
  }
  pub fn get_digits(&mut self, capacity: usize) -> Digits {
    pool_remove!(self, self.digitss, capacity, mut d, d, Self::add_digits_node, Vec::<Digits>::pdrop, { d.clear() });
    Digits::with_capacity(capacity)
  }
  pub fn get_ints(&mut self, capacity: usize) -> Vec<i32> {
    pool_remove!(self, self.intss, capacity, mut i, i, Self::add_ints_node, Vec::<Vec<i32>>::pdrop, { i.clear() });
    Vec::<i32>::with_capacity(capacity)
  }
  pub fn get_faliases(&mut self, capacity: usize) -> Faliases {
    pool_remove!(self, self.faliasess, capacity, mut faliases, faliases, Self::add_faliases_node, Vec::<Faliases>::pdrop, {
      for s in faliases.drain() { self.add_string(s); }
    });
    Faliases::with_capacity(capacity)
  }
  pub fn get_word_table(&mut self, capacity: usize) -> WordTable {
    pool_remove!(self, self.word_tables, capacity, mut wt, wt, Self::add_word_table_node, Vec::<WordTable>::pdrop, {
      for (key, word_def) in wt.drain() {
        self.add_string(key);
        self.add_word_def(word_def);
      }
    });
    WordTable::with_capacity(capacity)
  }
  pub fn get_word_def(&mut self, v: Value) -> WordDef {
    pool_pop!(self.word_defs, mut wd, wd, {
      *(Arc::<Value>::get_mut(&mut wd).expect("unauthorized references to word def in pool")) = v;
    });
    WordDef::from(v)
  }
  pub fn get_family(&mut self) -> Family {
    pool_pop!(self.families, mut family, family, { family.clear(); });
    Family::with_capacity(DEFAULT_STACK_SIZE)
  }

  pub fn get_un_op(&mut self) -> UnaryOp {
    pool_pop!(self.un_ops, mut op, op, { op.drain() });
    UnaryOp::new()
  }
  pub fn get_bin_op(&mut self) -> BinaryOp {
    pool_pop!(self.bin_ops, mut op, op, { op.drain() });
    BinaryOp::new()
  }
  pub fn get_str_op(&mut self) -> StrOp {
    pool_pop!(self.str_ops, mut op, op, { op.drain() });
    StrOp::new()
  }
  pub fn get_custom_op(&mut self) -> CustomOp {
    pool_pop!(self.custom_ops, mut op, op, { op.drain() });
    CustomOp::new()
  }

  pub fn get_value_node(&mut self, key: usize, data: Stack) -> Box<Node<usize, Value>> {
    pool_pop_node!(self.nodes.value, mut n, n, key, data, {
      if let Some(d) = n.data.take() {
        self.add_stack(d)
      }
    });
    Box::new(Node::<usize, Value>::new(key, data))
  }
  pub fn get_verr_loc_node(&mut self, key: usize, data: Vec<VErrorLoc>) -> Box<Node<usize, VErrorLoc>> {
    pool_pop_node!(self.nodes.verr_loc, mut n, n, key, data, {});
    Box::new(Node::<usize, VErrorLoc>::new(key, data))
  }
  pub fn get_stack_node(&mut self, key: usize, data: Vec<Stack>) -> Box<Node<usize, Stack>> {
    pool_pop_node!(self.nodes.stack, mut n, n, key, data, {});
    Box::new(Node::<usize, Stack>::new(key, data))
  }
  pub fn get_string_node(&mut self, key: usize, data: Strings) -> Box<Node<usize, String>> {
    pool_pop_node!(self.nodes.string, mut n, n, key, data, {});
    Box::new(Node::<usize, String>::new(key, data))
  }
  pub fn get_functions_node(&mut self, key: usize, data: Vec<Functions>) -> Box<Node<usize, Functions>> {
    pool_pop_node!(self.nodes.functions, mut n, n, key, data, {});
    Box::new(Node::<usize, Functions>::new(key, data))
  }
  pub fn get_cranks_node(&mut self, key: usize, data: Vec<Cranks>) -> Box<Node<usize, Cranks>> {
    pool_pop_node!(self.nodes.cranks, mut n, n, key, data, {});
    Box::new(Node::<usize, Cranks>::new(key, data))
  }
  pub fn get_math_node(&mut self, key: usize, data: Vec<Math>) -> Box<Node<usize, Math>> {
    pool_pop_node!(self.nodes.math, mut n, n, key, data, {});
    Box::new(Node::<usize, Math>::new(key, data))
  }
  pub fn get_digits_node(&mut self, key: usize, data: Vec<Digits>) -> Box<Node<usize, Digits>> {
    pool_pop_node!(self.nodes.digits, mut n, n, key, data, {});
    Box::new(Node::<usize, Digits>::new(key, data))
  }
  pub fn get_ints_node(&mut self, key: usize, data: Vec<Vec<i32>>) -> Box<Node<usize, Vec<i32>>> {
    pool_pop_node!(self.nodes.ints, mut n, n, key, data, {});
    Box::new(Node::<usize, Vec<i32>>::new(key, data))
  }
  pub fn get_faliases_node(&mut self, key: usize, data: Vec<Faliases>) -> Box<Node<usize, Faliases>> {
    pool_pop_node!(self.nodes.faliases, mut n, n, key, data, {});
    Box::new(Node::<usize, Faliases>::new(key, data))
  }
  pub fn get_word_table_node(&mut self, key: usize, data: Vec<WordTable>) -> Box<Node<usize, WordTable>> {
    pool_pop_node!(self.nodes.word_table, mut n, n, key, data, {});
    Box::new(Node::<usize, WordTable>::new(key, data))
  }
}

pub fn init_vword(capacity: usize) -> Value {
  Value::Word(Box::new(VWord::with_capacity(capacity)))
}
pub fn init_vstack(capacity: usize) -> Value {
  Value::Stack(Box::new(VStack::with_capacity(capacity)))
}
pub fn init_vmacro(capacity: usize) -> Value {
  Value::Macro(Box::new(VMacro::with_capacity(capacity)))
}
pub fn init_verror(capacity: usize) -> Value {
  Value::Error(Box::new(VError::with_capacity(capacity)))
}
pub fn init_verr_loc(capacity: usize) -> VErrorLoc {
  VErrorLoc{ filename: String::with_capacity(capacity),
             line: String::with_capacity(4),
             column: String::with_capacity(4) }
}
pub fn init_vfllib() -> Value {
  Value::FLLib(Box::new(VFLLib::with_nop()))
}
pub fn init_word_def() -> WordDef {
  WordDef::new(Value::Control(VControl::Ghost))
}
pub fn init_family() -> Family {
  Family::with_capacity(DEFAULT_STACK_SIZE)
}
pub fn init_inode<D>() -> Box<Node<usize, D>> {
  Box::new(Node{ key: 0, data: None, height: 1, node_left: None, node_right: None })
}
