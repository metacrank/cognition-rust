use crate::*;
use crate::tree::*;
use crate::math::*;

type ITree<T> = Tree<usize, T>;
type VTree = ITree<Value>;

pub struct Pool {
  vwords: Option<VTree>,
  vstacks: Option<VTree>,
  vmacros: Option<VTree>,
  verrors: Option<VTree>,
  vcustoms: Option<Stack>,
  vfllibs: Option<Stack>,

  verror_locs: Option<ITree<VErrorLoc>>,
  stacks: Option<ITree<Stack>>,
  strings: Option<ITree<String>>,
  stringss: Option<ITree<Strings>>,
  crankss: Option<ITree<Cranks>>,
  maths: Option<ITree<Math>>,
  digitss: Option<ITree<Digits>>,
  intss: Option<ITree<Vec<i32>>>,

  word_tables: Option<Vec<WordTable>>,
  word_defs: Option<Vec<WordDef>>,
  families: Option<Vec<Family>>,
  faliasess: Option<Vec<Faliases>>,
  parsers: Option<Vec<Parser>>,

  un_ops: Option<Vec<UnaryOp>>,
  bin_ops: Option<Vec<BinaryOp>>,
  str_ops: Option<Vec<StrOp>>,
  custom_ops: Option<Vec<CustomOp>>,
  ops_tables: Option<Vec<OpsTable>>,

  reg: [isize; 8],
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
macro_rules! pool_remove {
  ($self:ident,$tree:expr,$capacity:expr,$var:pat,$retval:tt,$destruct:expr,$mod:block) => {
    if let Some(mut tree) = $tree.take() {
      if let Some($var) = tree.remove_at_least($capacity, $destruct, $self) {
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

impl Pool {
  pub fn new() -> Pool {
    Pool{
      vwords: None,
      vstacks: None,
      vmacros: None,
      verrors: None,
      vcustoms: None,
      vfllibs: None,

      verror_locs: None,
      stacks: None,
      strings: None,
      stringss: None,
      crankss: None,
      maths: None,
      digitss: None,
      intss: None,

      word_tables: None,
      word_defs: None,
      families: None,
      faliasess: None,
      parsers: None,

      un_ops: None,
      bin_ops: None,
      str_ops: None,
      custom_ops: None,
      ops_tables: None,

      reg: [0; 8],
    }
  }

  pub fn print(&self) {
    if let Some(ref vwords) = self.vwords {
      print!("vwords: ");
      vwords.print();
      println!("");
    }
    if let Some(ref vstacks) = self.vstacks {
      print!("vstacks: ");
      vstacks.print();
      println!("");
    }
    if let Some(ref vmacros) = self.vmacros {
      print!("vmacros: ");
      vmacros.print();
      println!("");
    }
    if let Some(ref verrors) = self.verrors {
      print!("verrors: ");
      verrors.print();
      println!("");
    }
    if let Some(ref verror_locs) = self.verror_locs {
      print!("verror_locs: ");
      verror_locs.print();
      println!("");
    }
    if let Some(ref vcustoms) = self.vcustoms {
      println!("vcustoms: {}", vcustoms.len());
    }
    if let Some(ref vfllibs) = self.vfllibs {
      println!("vfllibs: {}", vfllibs.len());
    }

    if let Some(ref stacks) = self.stacks {
      print!("stacks: ");
      stacks.print();
      println!("");
    }
    if let Some(ref strings) = self.strings {
      print!("strings: ");
      strings.print();
      println!("");
    }
    if let Some(ref stringss) = self.stringss {
      print!("stringss: ");
      stringss.print();
      println!("");
    }
    if let Some(ref crankss) = self.crankss {
      print!("crankss: ");
      crankss.print();
      println!("");
    }
    if let Some(ref maths) = self.maths {
      print!("maths: ");
      maths.print();
      println!("");
    }
    if let Some(ref digitss) = self.digitss {
      print!("digitss: ");
      digitss.print();
      println!("");
    }
    if let Some(ref intss) = self.intss {
      print!("intss: ");
      intss.print();
      println!("");
    }

    if let Some(ref word_tables) = self.word_tables {
      println!("word_tables: {}", word_tables.len());
    }
    if let Some(ref word_defs) = self.word_defs {
      println!("word_defs: {}", word_defs.len());
    }
    if let Some(ref families) = self.families {
      println!("families: {}", families.len());
    }
    if let Some(ref faliasess) = self.faliasess {
      println!("faliasess: {}", faliasess.len());
    }
    if let Some(ref parsers) = self.parsers {
      println!("parsers: {}", parsers.len());
    }

    if let Some(ref un_ops) = self.un_ops {
      println!("un_ops: {}", un_ops.len());
    }
    if let Some(ref bin_ops) = self.bin_ops {
      println!("bin_ops: {}", bin_ops.len());
    }
    if let Some(ref str_ops) = self.str_ops {
      println!("str_ops: {}", str_ops.len());
    }
    if let Some(ref custom_ops) = self.custom_ops {
      println!("custom_ops: {}", custom_ops.len());
    }
    if let Some(ref ops_tables) = self.ops_tables {
      println!("ops_tables: {}", ops_tables.len());
    }

    print!("reg: [{}", self.reg[0]);
    for i in &self.reg[1..] {
      print!(", {}", i);
    }
    println!("]");
  }

  pub fn get_capacity(&self) -> [isize;32] {
    [
      self.vwords.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize) as isize),
      self.vstacks.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize) as isize),
      self.vmacros.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize) as isize),
      self.verrors.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize) as isize),
      self.vcustoms.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize) as isize),
      self.vfllibs.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize) as isize),

      self.verror_locs.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize) as isize),
      self.stacks.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize) as isize),
      self.strings.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize) as isize),
      self.stringss.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize) as isize),
      self.crankss.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize) as isize),
      self.maths.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize) as isize),
      self.digitss.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize) as isize),
      self.intss.as_ref().map_or(0, |t| t.size().min(isize::MAX as usize) as isize),

      self.word_tables.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize) as isize),
      self.word_defs.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize) as isize),
      self.families.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize) as isize),
      self.faliasess.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize) as isize),
      self.parsers.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize) as isize),

      self.un_ops.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize) as isize),
      self.bin_ops.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize) as isize),
      self.str_ops.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize) as isize),
      self.custom_ops.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize) as isize),
      self.ops_tables.as_ref().map_or(0, |v| v.len().min(isize::MAX as usize) as isize),

      self.reg[0],
      self.reg[1],
      self.reg[2],
      self.reg[3],
      self.reg[4],
      self.reg[5],
      self.reg[6],
      self.reg[7]
    ]
  }

  pub fn set_capacity(&mut self, _capacity: [isize;32]) {







  }

  pub fn add_val(&mut self, v: Value) {
    match v {
      Value::Word(vword)     => self.add_vword(vword),
      Value::Stack(vstack)   => self.add_vstack(vstack),
      Value::Macro(vmacro)   => self.add_vmacro(vmacro),
      Value::Error(verror)   => self.add_verror(verror),
      Value::FLLib(vfllib)   => self.add_vfllib(vfllib),
      Value::Custom(vcustom) => self.add_vcustom(vcustom),
      Value::Control(_) => {},
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
    pool_insert!(loc, loc.filename.capacity(), self, self.verror_locs, VErrorLoc, Vec::<VErrorLoc>::pnew);
  }
  pub fn add_vfllib(&mut self, vfllib: Box<VFLLib>) {
    pool_push_val!(Value::FLLib(vfllib), self, self.vfllibs);
  }
  pub fn add_vcustom(&mut self, mut vcustom: VCustom) {
    vcustom.custom.take();
    pool_push_val!(Value::Custom(vcustom), self, self.vcustoms);
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
  pub fn add_math(&mut self, m: Math) {
    pool_insert!(m, m.base() as usize, self, self.maths, Math, Vec::<Math>::pnew);
  }
  pub fn add_digits(&mut self, d: Digits) {
    pool_insert!(d, d.capacity(), self, self.digitss, Digits, Vec::<Digits>::pnew);
  }
  pub fn add_ints(&mut self, i: Vec<i32>) {
    pool_insert!(i, i.capacity(), self, self.intss, Vec<i32>, Vec::<Vec<i32>>::pnew);
  }

  pub fn add_word_table(&mut self, wt: WordTable) {
    pool_push!(wt, self, self.word_tables, Vec::<WordTable>::with_capacity(DEFAULT_STACK_SIZE));
  }
  pub fn add_word_def(&mut self, wd: WordDef) {
    if Arc::<Value>::strong_count(&wd) != 1 { return }
    pool_push!(wd, self, self.word_defs, Vec::<WordDef>::with_capacity(DEFAULT_STACK_SIZE));
  }
  pub fn add_family(&mut self, f: Family) {
    pool_push!(f, self, self.families, Vec::<Family>::with_capacity(DEFAULT_STACK_SIZE));
  }
  pub fn add_faliases(&mut self, f: Faliases) {
    pool_push!(f, self, self.faliasess, Vec::<Faliases>::with_capacity(DEFAULT_STACK_SIZE));
  }
  pub fn add_parser(&mut self, p: Parser) {
    pool_push!(p, self, self.parsers, Vec::<Parser>::with_capacity(DEFAULT_STACK_SIZE));
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
  pub fn add_ops_table(&mut self, ot: OpsTable) {
    pool_push!(ot, self ,self.ops_tables, Vec::<OpsTable>::with_capacity(DEFAULT_STACK_SIZE));
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
    pool_remove!(self, self.verror_locs, capacity, mut loc, loc, Vec::<VErrorLoc>::pdrop, {
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
      vfllib.fllib = f;
      if let Some(s) = vfllib.str_word.take() {
        self.add_string(s)
      }
    });
    Box::new(VFLLib::with_fn(f))
  }
  pub fn get_vcustom(&mut self, custom: Box<dyn CustomAny>) -> VCustom {
    pool_pop_val!(self.vcustoms, Value::Custom(mut vcustom), vcustom, { vcustom.custom = Some(custom); });
    VCustom{ custom: Some(custom) }
  }

  pub fn get_stack(&mut self, capacity: usize) -> Stack {
    pool_remove!(self, self.stacks, capacity, mut stack, stack, Vec::<Stack>::pdrop, {
      while let Some(v) = stack.pop() { self.add_val(v) }
    });
    Stack::with_capacity(capacity)
  }
  pub fn get_stack_for_pool(&mut self) -> Stack {
    self.get_stack(DEFAULT_STACK_SIZE)
  }
  pub fn get_string(&mut self, capacity: usize) -> String {
    pool_remove!(self, self.strings, capacity, mut s, s, Self::add_strings, { s.clear() });
    String::with_capacity(capacity)
  }
  pub fn get_strings(&mut self, capacity: usize) -> Strings {
    pool_remove!(self, self.stringss, capacity, mut ss, ss, Vec::<Strings>::pdrop, {
      while let Some(s) = ss.pop() { self.add_string(s) }
    });
    Strings::with_capacity(capacity)
  }
  pub fn get_strings_for_pool(&mut self) -> Strings {
    self.get_strings(DEFAULT_STACK_SIZE)
  }
  pub fn get_cranks(&mut self, capacity: usize) -> Cranks {
    pool_remove!(self, self.crankss, capacity, mut cs, cs, Vec::<Cranks>::pdrop, { cs.clear() });
    Cranks::with_capacity(capacity)
  }
  pub fn get_math(&mut self, mut base: i32) -> Math {
    if base < 0 { base = 0 }
    pool_remove!(self, self.maths, base as usize, mut m, m, Vec::<Math>::pdrop, { m.clean() });
    Math::new()
  }
  pub fn get_digits(&mut self, capacity: usize) -> Digits {
    pool_remove!(self, self.digitss, capacity, mut d, d, Vec::<Digits>::pdrop, { d.clear() });
    Digits::with_capacity(capacity)
  }
  pub fn get_ints(&mut self, capacity: usize) -> Vec<i32> {
    pool_remove!(self, self.intss, capacity, mut i, i, Vec::<Vec<i32>>::pdrop, { i.clear() });
    Vec::<i32>::with_capacity(capacity)
  }

  pub fn get_word_table(&mut self) -> WordTable {
    pool_pop!(self.word_tables, mut wt, wt, {
      for (key, word_def) in wt.drain() {
        self.add_string(key);
        self.add_word_def(word_def);
      }
    });
    WordTable::new()
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
  pub fn get_faliases(&mut self) -> Faliases {
    pool_pop!(self.faliasess, mut faliases, faliases, {
      for s in faliases.drain() { self.add_string(s); }
    });
    Faliases::with_capacity(DEFAULT_FALIASES_SIZE)
  }
  pub fn get_parser(&mut self) -> Parser {
    pool_pop!(self.parsers, mut parser, parser, {
      if let Some(s) = parser.source() { self.add_string(s) }
    });
    Parser::new(None, None)
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
  pub fn get_ops_table(&mut self) -> OpsTable {
    pool_pop!(self.ops_tables, mut op, op, {
      for (key, op) in op.drain() {
        self.add_string(key);
        self.add_op(op);
      }
    });
    OpsTable::new()
  }
}
