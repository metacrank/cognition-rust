use crate::*;

use std::str::Chars;
//use std::iter::Rev;
//use std::slice::IterMut;

const N_O_D: usize = 12;
const BASE: usize = 10;
//const RADIUS: usize = BASE / 2 + 1;
const B_ODD: bool = BASE % 2 == 1;

/// The standard in this module is to use codepoint U0305 (Combining Overline)
/// to denote negative digits. Having some restriction on the constant digit
/// patterns is necessary to avoid ambiguous number strings, and checking for
/// a single char makes way more sense than matching substrings.
const DIGITS: [char; N_O_D] = [ '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '↊', '↋' ];
const NEG_C: char = '\u{0305}'; // Combining Overline

// const fn digits(n: [&'static str; N_O_D], p: [&'static str; N_O_D]) -> [&'static str; BASE] {
//   let mut result = [""; BASE];
//   let mut i = 0;
//   while i < N_O_D {
//     result[i] = a[i];
//     i += 1;
//   }
//   while i < A + B {
//     result[i] = b[i - A];
//     i += 1;
//   }
//   result
// }

//const DIGGITS: &'static [&str] = &POS_DIGITS[];

//const DIGITS: [&str; BASE] = concat(POS_DIGITS[..BASE/2], NEG_DIGITS[..BASE/2]);

const fn addition_table(_d: [char; N_O_D]) -> [[char; BASE]; BASE] {
  let mut a = 0;
  while a < BASE {
    // donothing
    a += 1;
    a -= 1;
    if a == 4 {
      return [['0'; BASE]; BASE];
    }
  }
  [['1'; BASE]; BASE]
}

const fn multiplication_table(_d: [char; N_O_D]) -> [[char; BASE]; BASE] {
  [['0'; BASE]; BASE]
}

//const ADDITION_TABLE: [[char; BASE]; BASE] = addition_table(DIGITS);

pub type Digits = Vec<Digit>;
pub type UnaryOp = HashMap<Digit, Digit>;
pub type BinaryOp = HashMap<(Digit, Digit), (Digit, Digit)>;
pub type StrOp = HashMap<String, String>;
pub type CustomOp = HashMap<Operand, Operand>;

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct Digit {
  digit: char,
  neg: bool,
}

// #[derive(Eq, Hash, PartialEq, Clone)]
// enum Carry {
//   PosOne,
//   Zero,
//   NegOne,
// }

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum Operand {
  Unary(Digit),
  Binary(Digit, Digit),
  Ternary(Digit, Digit, Digit),
  Quaternary(Digit, Digit, Digit, Digit),
  Arbitrary(Vec<Digit>),
}

impl Operand {
  pub fn is_unary(&self) -> bool { if let Self::Unary(_) = self { true } else { false } }
  pub fn is_binary(&self) -> bool { if let Self::Binary(_, _) = self { true } else { false } }
  pub fn is_ternary(&self) -> bool { if let Self::Ternary(_, _, _) = self { true } else { false } }
  pub fn is_quaternary(&self) -> bool { if let Self::Quaternary(_, _, _, _) = self { true } else { false } }
  pub fn is_arbitrary(&self) -> bool { if let Self::Arbitrary(_) = self { true } else { false } }

  // pub fn first(&self) -> char {
  //   match self {
  //     Self::Unary(u) => u.clone(),
  //     Self::Binary(b, _) => b.clone(),
  //     Self::Ternary(t, _, _) => t.clone(),
  //     Self::Quaternary(q, _, _, _) => q.clone(),
  //     Self::Arbitrary(v) => v.first().expect("TableObject::Arbitrary does not have first value").clone(),
  //   }
  // }
  // pub fn second(&self) -> char {
  //   match self {
  //     Self::Unary(_) => panic!("TableObject::Unary does not have second value"),
  //     Self::Binary(_, b) => b.clone(),
  //     Self::Ternary(_, t, _) => t.clone(),
  //     Self::Quaternary(_, q, _, _) => q.clone(),
  //     Self::Arbitrary(v) => v.get(1).expect("TableObject::Arbitrary does not have second value").clone(),
  //   }
  // }
  // pub fn third(&self) -> char {
  //   match self {
  //     Self::Unary(_) => panic!("TableObject::Unary does not have third value"),
  //     Self::Binary(_, _) => panic!("TableObject::Binary does not have third value"),
  //     Self::Ternary(_, _, t) => t.clone(),
  //     Self::Quaternary(_, _, t, _) => q.clone(),
  //     Self::Arbitrary(v) => v.get(2).expect("TableObject::Arbitrary does not have second value").clone(),
  //   }
  // }
  // pub fn fourth(&self) -> char {
  //   match self {
  //     Self::Unary(_) => panic!("TableObject::Unary does not have fourth value"),
  //     Self::Binary(_, _) => panic!("TableObject::Unary does not have fourth value"),
  //     Self::Ternary(_, _, _) => panic!("TableObject::Ternary does not have fourth value"),
  //     Self::Quaternary(_, _, _, q) => q.clone(),
  //     Self::Arbitrary(v) => v.get(3).expect("TableObject::Arbitrary does not have second value").clone(),
  //   }
  // }

  // pub fn first_ref(&self) -> &char {
  //   match self {
  //     Self::Unary(u) => u,
  //     Self::Binary(b, _) => b,
  //     Self::Ternary(t, _, _) => t,
  //     Self::Quaternary(q, _, _, _) => q,
  //     Self::Arbitrary(v) => v.first().expect("TableObject::Arbitrary does not have first value"),
  //   }
  // }
  // pub fn second_ref(&self) -> &char {
  //   match self {
  //     Self::Unary(_) => panic!("TableObject::Unary does not have second value"),
  //     Self::Binary(_, b) => b,
  //     Self::Ternary(_, t, _) => t,
  //     Self::Quaternary(_, q, _, _) => q,
  //     Self::Arbitrary(v) => v.get(1).expect("TableObject::Arbitrary does not have second value"),
  //   }
  // }
  // pub fn third_ref(&self) -> &char {
  //   match self {
  //     Self::Unary(_) => panic!("TableObject::Unary does not have third value"),
  //     Self::Binary(_, _) => panic!("TableObject::Binary does not have third value"),
  //     Self::Ternary(_, _, t) => t,
  //     Self::Quaternary(_, _, t, _) => q,
  //     Self::Arbitrary(v) => v.get(2).expect("TableObject::Arbitrary does not have second value"),
  //   }
  // }
  // pub fn fourth_ref(&self) -> &char {
  //   match self {
  //     Self::Unary(_) => panic!("TableObject::Unary does not have fourth value"),
  //     Self::Binary(_, _) => panic!("TableObject::Unary does not have fourth value"),
  //     Self::Ternary(_, _, _) => panic!("TableObject::Ternary does not have fourth value"),
  //     Self::Quaternary(_, _, _, q) => q,
  //     Self::Arbitrary(v) => v.get(3).expect("TableObject::Arbitrary does not have second value"),
  //   }
  // }
}

macro_rules! push_next {
  ($c:ident,$iter:ident,$tmp:ident,$negc:ident,$radix:ident,$delim:ident) => {
    if $c == $negc {
      let Some(c) = $iter.next() else { return Some("INVALID NUMBER STRING") };
      $tmp.push(Digit{ digit: c, neg: true }); (true, false)
    }
    else if $c == $radix { (false, true) }
    else if $c == $delim { (false, false) }
    else { $tmp.push(Digit{ digit: $c, neg: false }); (true, false) }
  };
}
macro_rules! push_next_int {
  ($self:ident,$c:ident,$iter:ident,$tmp:ident,$negc:ident,$radix:ident,$delim:ident) => {
    if $c == $negc {
      let Some(c) = $iter.next() else { return Err("INVALID NUMBER STRING") };
      let Some(i) = $self.d_idx.get(&c) else { return Err("INVALID NUMBER STRING") };
      $tmp.push(-(*i)); (true, false)
    }
    else if $c == $radix { (false, true) }
    else if $c == $delim { (false, false) }
    else {
      let Some(i) = $self.d_idx.get(&$c) else { return Err("INVALID NUMBER STRING") };
      $tmp.push(*i); (true, false)
    }
  };
}

pub struct Math {
  base: i32,
  digits: Vec<char>,
  d_idx: HashMap<char, i32>,
  // add: HashMap<(char, char, Carry), (Digit, Carry)>,
  mul: HashMap<(char, char), (i32, i32)>,

  pub un_ops: Vec<UnaryOp>,
  pub bin_ops: Vec<BinaryOp>,
  pub str_ops: Vec<StrOp>,
  pub custom_ops: Vec<CustomOp>,

  negc: Option<char>,
  radix: Option<char>,
  delim: Option<char>,
}

pub enum BaseResult {
  Ok,
  Digits,
  Negc,
  Radix,
  Delim,
}

impl Math {
  pub fn new() -> Self {
    Self { base: 0, digits: Vec::<char>::new(),
           d_idx: HashMap::new(), mul: HashMap::new(),
           un_ops: Vec::new(), bin_ops: Vec::new(),
           str_ops: Vec::new(), custom_ops: Vec::new(),
           negc: None, radix: None, delim: None }
  }

  pub fn clean(&mut self) {
    self.base = 0;
    self.digits.clear();
    self.d_idx.drain();
    self.mul.drain();

    for ht in self.un_ops.iter_mut()     { ht.drain(); }
    for ht in self.bin_ops.iter_mut()    { ht.drain(); }
    for ht in self.str_ops.iter_mut()    { ht.drain(); }
    for ht in self.custom_ops.iter_mut() { ht.drain(); }

    self.negc = None;
    self.radix = None;
    self.delim = None;
  }

  pub fn copy_into(&self, math: &mut Math, state: &mut CognitionState) {
    math.base = self.base;
    for d in self.digits.iter() {
      math.digits.push(d.clone());
    }
    for (c, i) in self.d_idx.iter() {
      math.d_idx.insert(c.clone(), i.clone());
    }
    // for ((d11, d12, c1), (d21, c2)) in self.add.iter() {
    //   math.add.insert((d11.clone(), d12.clone(), c1.clone()), (d21.clone(), c2.clone()));
    // }
    for ((d11, d12), (d21, d22)) in self.mul.iter() {
      math.mul.insert((d11.clone(), d12.clone()), (d21.clone(), d22.clone()));
    }
    for op in self.un_ops.iter() {
      let mut new_op = state.pool.get_un_op();
      for (d1, d2) in op.iter() {
        new_op.insert(d1.clone(), d2.clone());
      }
      math.un_ops.push(new_op);
    }
    for op in self.bin_ops.iter() {
      let mut new_op = state.pool.get_bin_op();
      for ((d11, d12), (d21, d22)) in op.iter() {
        new_op.insert((d11.clone(), d12.clone()), (d21.clone(), d22.clone()));
      }
      math.bin_ops.push(new_op);
    }
    for op in self.str_ops.iter() {
      let mut new_op = state.pool.get_str_op();
      for (s1, s2) in op.iter() {
        new_op.insert(s1.clone(), s2.clone());
      }
      math.str_ops.push(new_op);
    }
    for op in self.custom_ops.iter() {
      let mut new_op = state.pool.get_custom_op();
      for (c1, c2) in op.iter() {
        new_op.insert(c1.clone(), c2.clone());
      }
      math.custom_ops.push(new_op);
    }
    math.negc = self.negc;
    math.radix = self.radix;
    math.delim = self.delim;
  }

  pub fn set_digits(&mut self, s: &String) {
    self.digits.clear();
    for c in s.chars() {
      self.digits.push(c);
    }
    self.d_idx.drain();
    for i in 0..self.digits.len() {
      self.d_idx.insert(self.digits[i], i as i32);
    }
  }

  pub fn set_negc (&mut self, c: char) { self.negc = Some(c) }
  pub fn set_radix(&mut self, c: char) { self.radix = Some(c) }
  pub fn set_delim(&mut self, c: char) { self.delim = Some(c) }

  pub fn set_base(&mut self, base: i32) -> BaseResult {
    if self.digits.len() < (base / 2 + 1) as usize { return BaseResult::Digits }
    if self.negc.is_none() { return BaseResult::Negc }
    if self.radix.is_none() { return BaseResult::Radix }
    if self.delim.is_none() { return BaseResult::Delim }
    self.base = base;
    self.init_mul();
    BaseResult::Ok
  }

  pub fn base(&self) -> i32 { self.base }

  // fn init_add(&mut self) {
  //   let radius = self.base / 2 + 1;
  //   let b_odd = self.base % 2;
  //   for i in 0..radius {
  //     for j in 0..radius {
  //       let di = Digit { digit: self.digits[i], sign: true };
  //       let dj = Digit { digit: self.digits[j], sign: true };

  //       let sum = if {
  //         i + j <= radius + (b_odd - 1) {
  //           Digit { self.digits[i + j], sign: true };
  //         } else {

  //         }
  //       }

  //     }
  //   }
  // }
  fn init_mul(&mut self) {
    let radius = self.base / 2 + 1;
    for i in 0..radius {
      for j in 0..radius {
        let product = i * j;
        let mut carry = product / self.base;
        let mut product = product % self.base;
        if product >= radius {
          product -= self.base;
          carry += 1;
        }
        self.mul.insert((self.digits[i as usize], self.digits[j as usize]), (product, carry));
      }
    }

    println!("Multiplication table:");
    for i in 0..radius {
      for j in 0..radius {
        let Some((product, carry)) = self.mul.get(&(self.digits[i as usize], self.digits[j as usize])) else { unreachable!() };
        print!("{}", self.digits[*carry as usize]);
        if *product < 0 { print!("{}{} ", self.negc.unwrap(), self.digits[-(*product) as usize]) }
        else { print!("{} ", self.digits[*product as usize]); }
      }
      println!("");
    }
  }

  pub fn len(&self, s: &str) -> Result<usize, &'static str> {
    let Some(negc) = self.negc else { return Err("NEGC UNINITIALIZED") };
    let Some(radix) = self.radix else { return Err("RADIX UNINITIALIZED") };
    let mut l: usize = 0;
    let mut s_iter = s.chars();
    loop {
      let Some(c) = s_iter.next() else { break Ok(l) };
      if c == negc {
        if s_iter.next() == self.radix { break Ok(l + 1) }
      } else if c == radix { break Ok(l) }
      l += 1
    }
  }

  fn into_digits(&self, s: &str, v: &mut Vec<Digit>) {
    let negc = self.negc.expect("negc uninitialized");
    let mut iter = s.chars();
    while let Some(c) = iter.next() {
      if c == negc {
        let Some(d) = iter.next() else { break };
        v.push(Digit{ digit: d, neg: true });
        continue
      }
      v.push(Digit{ digit: c, neg: false });
    }
  }

  // fn get_next_digit(&self, c1: &mut char, iter: &mut Rev<Chars>, negc: char) -> Result<(i32, bool), &'static str> {
  //   let c1_old = *c1;
  //   let Some(i) = self.d_idx.get(&c1_old) else { return Err("INVALID NUMBER STRING") };
  //   let (c_val, alive) = if let Some(c) = iter.next() {
  //     if c == negc {
  //       if let Some(c) = iter.next() {
  //         *c1 = c;
  //         (-(*i), true)
  //       } else {
  //         (-(*i), false)
  //       }
  //     } else {
  //       *c1 = c;
  //       (*i, true)
  //     }
  //   } else {
  //     (*i, false)
  //   };
  //   Ok((c_val, alive))
  // }

  // fn add_digits(&self, c1_val: i32, c2_val: i32, mut carry: i32, mut neg: bool, r_iter: &mut Rev<IterMut<Digit>>) -> (i32, bool) {
  //   let radius = &self.base / 2 + 1;
  //   let mut sum = c1_val + c2_val + carry;
  //   if sum < -radius {
  //     neg = false;
  //     carry = -1;
  //     sum += self.base;
  //   } else if sum == -radius {
  //     sum = radius;
  //     if neg { neg = false; carry = -1 }
  //     else   { neg = true;  carry =  0 }
  //   } else if sum < 0 {
  //     sum = -sum;
  //     neg = true;
  //   } else if sum > 0 && sum < radius {
  //     neg = false;
  //   } else if sum == radius {
  //     if neg { neg = false; carry = 0 }
  //     else   { neg = true;  carry = 1 }
  //   } else { // sum > radius
  //     neg = true;
  //     carry = 1;
  //     sum = self.base - sum;
  //   }
  //   *r_iter.next().unwrap() = Digit{ digit: self.digits[sum as usize], neg };
  //   (carry, neg)
  // }

  // /// f2 is always longer than f1 and includes a radix point
  // /// i2 if always longer than i1
  // fn add(&self, mut neg: bool,
  //        i1: &str, i2: &str,
  //        f1: &str, f2: &str,
  //        f_len_diff: usize,
  //        float: bool, mut r: Digits
  // ) -> Result<Digits, &'static str> {

  //   let Some(negc) = self.negc else { return Err("NEGC UNINITIALIZED") };
  //   let Some(radix) = self.radix else { return Err("RADIX UNINITIALIZED") };

  //   let mut r_iter = r.iter_mut().rev();
  //   let mut carry: i32 = 0;
  //   if float {
  //     let mut f1_iter = f1.chars().rev();
  //     let mut f2_iter = f2.chars().rev();
  //     if let Some(mut c2) = f2_iter.next() {
  //       // tail end of f2
  //       for _ in 0..f_len_diff {
  //         let (c2_val, _) = match self.get_next_digit(&mut c2, &mut f2_iter, negc) {
  //           Ok((a, b)) => (a, b),
  //           Err(e) => return Err(e),
  //         };
  //         (carry, neg) = self.add_digits(0, c2_val, carry, neg, &mut r_iter);
  //       }
  //       // add f1 and f2
  //       if let Some(mut c1) = f1_iter.next() {
  //         loop {
  //           let (c1_val, alive) = match self.get_next_digit(&mut c1, &mut f1_iter, negc) {
  //             Ok((a, b)) => (a, b),
  //             Err(e) => return Err(e),
  //           };
  //           let (c2_val, _) = match self.get_next_digit(&mut c2, &mut f2_iter, negc) {
  //             Ok((a, b)) => (a, b),
  //             Err(e) => return Err(e),
  //           };
  //           (carry, neg) = self.add_digits(c1_val, c2_val, carry, neg, &mut r_iter);
  //           if !alive { break }
  //         }
  //       }
  //     }
  //     *r_iter.next().unwrap() = Digit{ digit: radix, neg: false };
  //   }

  //   let mut i1_iter = i1.chars().rev();
  //   let mut i2_iter = i2.chars().rev();

  //   if let Some(mut c2) = i2_iter.next() {
  //     if let Some(mut c1) = i1_iter.next() {
  //       loop {
  //         let (c1_val, alive) = match self.get_next_digit(&mut c1, &mut i1_iter, negc) {
  //           Ok((a, b)) => (a, b),
  //           Err(e) => return Err(e),
  //         };
  //         let (c2_val, _) = match self.get_next_digit(&mut c2, &mut i2_iter, negc) {
  //           Ok((a, b)) => (a, b),
  //           Err(e) => return Err(e),
  //         };
  //         (carry, neg) = self.add_digits(c1_val, c2_val, carry, neg, &mut r_iter);
  //         if !alive { break }
  //       }
  //     }
  //     loop {
  //       let (c2_val, alive) = match self.get_next_digit(&mut c2, &mut i2_iter, negc) {
  //         Ok((a, b)) => (a, b),
  //         Err(e) => return Err(e),
  //       };
  //       (carry, neg) = self.add_digits(0, c2_val, carry, neg, &mut r_iter);
  //       if !alive { break }
  //     }
  //   }

  //   Ok(r)
  // }

  // pub fn sum(&self, s1: &String, s2: &String, pool: &mut Pool) -> Result<String, &'static str> {
  //   if self.base == 0 { return Err("UNINITIALIZED NUMBER BASE") }

  //   let result = pool.get_string(s1.len().max(s2.len()) * 4);
  //   let mut s1_iter = s1.char_indices();
  //   let mut s2_iter = s2.char_indices();

  //   let mut f_len_diff: usize = 0;
  //   let mut p1last: usize = 0;
  //   let mut p2last: usize = 0;


  //   let ((i1, i1_ok), (i2, i2_ok)) = loop {
  //     let mut c1 = s1_iter.next();
  //     let mut c2 = s2_iter.next();
  //     if c1.is_none() {
  //       break ((&s1[p1last..s1.len()], false), loop {
  //         if c2.is_none() { break (&s2[p2last..s2.len()], false) }
  //         if Some(c2.unwrap().1) == self.delim {
  //           let p2 = p2last;
  //           p2last = c2.unwrap().0 + self.delim.unwrap().len_utf8();
  //           break (&s2[p2..c2.unwrap().0], true)
  //         }
  //         c2 = s2_iter.next();
  //         f_len_diff += 1;
  //       })
  //     } else if Some(c1.unwrap().1) == self.delim {
  //       let p1 = p1last;
  //       p1last = c1.unwrap().0 + self.delim.unwrap().len_utf8();
  //       break ((&s1[p1..c1.unwrap().0], true), loop {
  //         if c2.is_none() { break (&s2[p2last..s2.len()], false) }
  //         if Some(c2.unwrap().1) == self.delim {
  //           let p2 = p2last;
  //           p2last = c2.unwrap().0 + self.delim.unwrap().len_utf8();
  //           break (&s2[p2..c2.unwrap().0], true)
  //         }
  //         c2 = s2_iter.next();
  //         f_len_diff += 1;
  //       })
  //     }

  //     if c2.is_none() {
  //       break ((&s1[p1last..s1.len()], false), loop {
  //         if c1.is_none() { break (&s2[p2last..s2.len()], false) }
  //         if Some(c1.unwrap().1) == self.delim {
  //           let p2 = p2last;
  //           p2last = c1.unwrap().0 + self.delim.unwrap().len_utf8();
  //           break (&s2[p2..c1.unwrap().0], true)
  //         }
  //         c1 = s2_iter.next();
  //         f_len_diff += 1;
  //       })
  //     } else if Some(c2.unwrap().1) == self.delim {
  //       let p1 = p1last;
  //       p1last = c2.unwrap().0 + self.delim.unwrap().len_utf8();
  //       break ((&s1[p1..c2.unwrap().0], true), loop {
  //         if c1.is_none() { break (&s2[p2last..s2.len()], false) }
  //         if Some(c1.unwrap().1) == self.delim {
  //           let p2 = p2last;
  //           p2last = c1.unwrap().0 + self.delim.unwrap().len_utf8();
  //           break (&s2[p2..c1.unwrap().0], true)
  //         }
  //         c1 = s2_iter.next();
  //         f_len_diff += 1;
  //       })
  //     }
  //   };




  //   // /// f2 is always longer than f1 and includes a radix point
  //   // /// i2 if always longer than i1
  //   // fn add(&self, mut neg: bool,
  //   //        i1: &str, i2: &str,
  //   //        f1: &str, f2: &str,
  //   //        f_len_diff: usize,
  //   //        float: bool, mut r: Digits
  //   // ) -> Result<Digits, &'static str>




  //   Err("not finished yet")
  // }


  fn pair_into_next_digits(&self, s1_iter: &mut Chars, s2_iter: &mut Chars, tmp1_digits: &mut Digits, tmp2_digits: &mut Digits) -> Option<&'static str> {
    let Some(negc) = self.negc else { return Some("MATH NEGC UNINITIALIZED") };
    let Some(radix) = self.radix else { return Some("MATH RADIX UNINITIALIZED") };
    let Some(delim) = self.delim else { return Some("MATH DELIM UNINITIALIZED") };

    let (mut i_len1, mut i_len2, mut f_len1, mut f_len2) = (0,0,0,0);

    let (mut alive1, mut alive2) = (true, true);
    let (mut frac1, mut frac2) = (false, false);

    let get_next = | alive1: bool, alive2: bool, iter1: &mut Chars, iter2: &mut Chars, len1: &mut usize, len2: &mut usize |
    (if alive1 { *len1 += 1; iter1.next() } else { None },
     if alive2 { *len2 += 1; iter2.next() } else { None });

    loop {
      let (c1opt, c2opt) = get_next(alive1, alive2, s1_iter, s2_iter, &mut i_len1, &mut i_len2);
      if let Some(c1) = c1opt { (alive1, frac1) = push_next!(c1, s1_iter, tmp1_digits, negc, radix, delim); }
      if let Some(c2) = c2opt { (alive2, frac2) = push_next!(c2, s2_iter, tmp2_digits, negc, radix, delim); }
      if (c1opt, c2opt) == (None, None) { break }
    }
    if frac1 { tmp1_digits.push(Digit{ digit: radix, neg: false }) }
    if frac2 { tmp2_digits.push(Digit{ digit: radix, neg: false }) }
    (alive1, alive2) = (frac1, frac2);
    loop {
      let (c1opt, c2opt) = get_next(alive1, alive2, s1_iter, s2_iter, &mut f_len1, &mut f_len2);
      if let Some(c1) = c1opt {
        (alive1, frac1) = push_next!(c1, s1_iter, tmp1_digits, negc, radix, delim);
        if frac1 { return Some("INVALID NUMBER STRING") }
      }
      if let Some(c2) = c2opt {
        (alive2, frac2) = push_next!(c2, s2_iter, tmp2_digits, negc, radix, delim);
        if frac2 { return Some("INVALID NUMBER STRING") }
      }
      if (c1opt, c2opt) == (None, None) { break None }
    }
  }

  fn add_digits(&self, c1_val: i32, c2_val: i32, mut carry: i32, mut neg: bool, r: &mut Digits) -> (i32, bool) {
    let radius = &self.base / 2 + 1;
    let mut sum = c1_val + c2_val + carry;
    if sum < -radius {
      neg = false;
      carry = -1;
      sum += self.base;
    } else if sum == -radius {
      sum = radius;
      if neg { neg = false; carry = -1 }
      else   { neg = true;  carry =  0 }
    } else if sum < 0 {
      sum = -sum;
      neg = true;
    } else if sum > 0 && sum < radius {
      neg = false;
    } else if sum == radius {
      if neg { neg = false; carry = 0 }
      else   { neg = true;  carry = 1 }
    } else { // sum > radius
      neg = true;
      carry = 1;
      sum = self.base - sum;
    }
    r.push(Digit{ digit: self.digits[sum as usize], neg });
    (carry, neg)
  }

  // /// f2 is always longer than f1
  // fn add(&self, mut neg: bool, d1: Digits, d2: Digits, f_len_diff: usize, float: bool, r: &mut Digits) -> Option<&'static str> {
  //   let Some(radix) = self.radix else { return Some("MATH RADIX UNINITIALIZED") };

  //   let mut r_iter = r.iter_mut().rev();
  //   let mut carry: i32 = 0;
  //   let mut d1_iter = d1.iter().rev();
  //   let mut d2_iter = d2.iter().rev();
  //   if float {
  //     // tail end of d2
  //     for _ in 0..f_len_diff {
  //       let Some(ref c2) = d2_iter.next()       else { return Some("INVALID NUMBER STRING") };
  //       let Some(i) = self.d_idx.get(&c2.digit) else { return Some("INVALID NUMBER STRING") };
  //       let c2val = if c2.neg { -(*i) } else { *i };
  //       (carry, neg) = self.add_digits(0, c2val, carry, neg, &mut r_iter);
  //     }
  //     // add f1 and f2
  //     loop {
  //       let Some(ref c1) = d1_iter.next()       else { return Some("INVALID NUMBER STRING") };
  //       if c1.digit == radix                         { d2_iter.next(); break }
  //       let Some(i) = self.d_idx.get(&c1.digit) else { return Some("INVALID NUMBER STRING") };
  //       let c1val = if c1.neg { -(*i) } else { *i };
  //       let Some(ref c2) = d2_iter.next()       else { return Some("INVALID NUMBER STRING") };
  //       let Some(i) = self.d_idx.get(&c2.digit) else { return Some("INVALID NUMBER STRING") };
  //       let c2val = if c2.neg { -(*i) } else { *i };
  //       (carry, neg) = self.add_digits(c1val, c2val, carry, neg, &mut r_iter);
  //     }
  //     *r_iter.next().unwrap() = Digit{ digit: radix, neg: false };
  //   }
  //   match loop {
  //     let Some(ref c1) = d1_iter.next()       else { break 2 };
  //     let Some(i) = self.d_idx.get(&c1.digit) else { return Some("INVALID NUMBER STRING") };
  //     let c1val = if c1.neg { -(*i) } else { *i };
  //     let Some(ref c2) = d2_iter.next()       else { break 1 };
  //     let Some(i) = self.d_idx.get(&c2.digit) else { return Some("INVALID NUMBER STRING") };
  //     let c2val = if c2.neg { -(*i) } else { *i };
  //     (carry, neg) = self.add_digits(c1val, c2val, carry, neg, &mut r_iter);
  //   } {
  //     1 => loop {
  //       let Some(ref c1) = d1_iter.next()       else { break };
  //       let Some(i) = self.d_idx.get(&c1.digit) else { return Some("INVALID NUMBER STRING") };
  //       let c1val = if c1.neg { -(*i) } else { *i };
  //       (carry, neg) = self.add_digits(c1val, 0, carry, neg, &mut r_iter);
  //     },
  //     _ => loop {
  //       let Some(ref c2) = d2_iter.next()       else { break };
  //       let Some(i) = self.d_idx.get(&c2.digit) else { return Some("INVALID NUMBER STRING") };
  //       let c2val = if c2.neg { -(*i) } else { *i };
  //       (carry, neg) = self.add_digits(0, c2val, carry, neg, &mut r_iter);
  //     },
  //   }

  //   self.add_digits(0, 0, carry, neg, &mut r_iter);
  //   None
  // }

  // add radix point retainment
  fn add(&self, mut neg: Option<bool>, d1: &mut Vec<i32>, d2: &mut Vec<i32>, f_len1: usize, f_len2: usize, r: &mut Digits, radix: char) {
    for (i1, i2) in d1.iter().zip(d2.iter()) {
      if neg.is_some() { break }
      if i1 + i2 < 0 { neg = Some(true) }
      if i1 + i2 > 0 { neg = Some(false) }
    }
    let mut neg = if let Some(v) = neg { v } else { false };
    let mut carry: i32 = 0;
    let mut d1_iter = d1.iter().rev();
    let mut d2_iter = d2.iter().rev();
    if f_len2 > 0 {
      // tail end of d2
      for _ in f_len1..f_len2 {
        let i = d2_iter.next().expect("Bad Math::add() call");
        (carry, neg) = self.add_digits(0, *i, carry, neg, r);
      }
      // add f1 and f2
      for _ in 0..f_len1 {
        let i1 = d1_iter.next().expect("Bad Math::add() call");
        let i2 = d2_iter.next().expect("Bad Math::add() call");
        (carry, neg) = self.add_digits(*i1, *i2, carry, neg, r);
      }
      r.push(Digit{ digit: radix, neg: false });
    }
    match loop {
      let Some(i1) = d1_iter.next() else { break 2 };
      let Some(i2) = d2_iter.next() else { break 1 };
      (carry, neg) = self.add_digits(*i1, *i2, carry, neg, r);
    } {
      1 => loop {
        let Some(i) = d1_iter.next() else { break };
        (carry, neg) = self.add_digits(*i, 0, carry, neg, r);
      },
      _ => loop {
        let Some(i) = d2_iter.next() else { break };
        (carry, neg) = self.add_digits(0, *i, carry, neg, r);
      }
    }
    self.add_digits(0, 0, carry, neg, r);
  }

  pub fn sum(&self, s1: &String, s2: &String, state: &mut CognitionState) -> Result<String, &'static str> {
    if self.base == 0 { return Err("UNINITIALIZED NUMBER BASE") }

    let Some(negc) = self.negc else { return Err("MATH NEGC UNINITIALIZED") };
    let Some(radix) = self.radix else { return Err("MATH RADIX UNINITIALIZED") };
    let Some(delim) = self.delim else { return Err("MATH DELIM UNINITIALIZED") };

    if s1.len() == 0 { return Ok(state.string_copy(s2)) }
    if s2.len() == 0 { return Ok(state.string_copy(s1)) }

    let mut result = state.pool.get_string(s1.len().max(s2.len()) * 4);

    let (mut tmp1_ints, mut tmp2_ints) = (state.pool.get_ints(s1.len()), state.pool.get_ints(s2.len()));
    let (mut s1_iter, mut s2_iter) = (s1.chars(), s2.chars());

    loop {
      let (mut i_len1, mut i_len2, mut f_len1, mut f_len2) = (0,0,0,0);

      let (mut alive1, mut alive2) = (true, true);
      let (mut frac1, mut frac2) = (false, false);

      let get_next = | alive1: bool, alive2: bool, iter1: &mut Chars, iter2: &mut Chars, len1: &mut usize, len2: &mut usize |
      (if alive1 { *len1 += 1; iter1.next() } else { None },
       if alive2 { *len2 += 1; iter2.next() } else { None });

      let mut neg = None;
      loop {
        let (c1opt, c2opt) = get_next(alive1, alive2, &mut s1_iter, &mut s2_iter, &mut i_len1, &mut i_len2);
        if let Some(c1) = c1opt { (alive1, frac1) = push_next_int!(self, c1, s1_iter, tmp1_ints, negc, radix, delim); }
        if let Some(c2) = c2opt { (alive2, frac2) = push_next_int!(self, c2, s2_iter, tmp2_ints, negc, radix, delim); }
        if alive1 && alive2 && neg.is_none() {
          let sum = tmp1_ints.last().unwrap() + tmp2_ints.last().unwrap();
          if sum < 0 { neg = Some(true) } else if sum > 0 { neg = Some(false) }
        }
        if (c1opt, c2opt) == (None, None) { break }
      }
      if i_len1 > i_len2      { neg = Some(*tmp1_ints.first().unwrap() < 0) }
      else if i_len1 < i_len2 { neg = Some(*tmp2_ints.first().unwrap() < 0) }
      (alive1, alive2) = (frac1, frac2);
      loop {
        let (c1opt, c2opt) = get_next(alive1, alive2, &mut s1_iter, &mut s2_iter, &mut f_len1, &mut f_len2);
        if let Some(c1) = c1opt {
          (alive1, frac1) = push_next_int!(self, c1, s1_iter, tmp1_ints, negc, radix, delim);
          if frac1 { return Err("INVALID NUMBER STRING") }
        }
        if let Some(c2) = c2opt {
          (alive2, frac2) = push_next_int!(self, c2, s2_iter, tmp2_ints, negc, radix, delim);
          if frac2 { return Err("INVALID NUMBER STRING") }
        }
        if alive1 && alive2 && neg.is_none() {
          let sum = tmp1_ints.last().unwrap() + tmp2_ints.last().unwrap();
          if sum < 0 { neg = Some(true) } else if sum > 0 { neg = Some(false) }
        }
        if (c1opt, c2opt) == (None, None) { break }
      }
      let n_o_d = tmp1_ints.len().max(tmp2_ints.len()) + 2;
      let mut tmp_digits = state.pool.get_digits(n_o_d);
      for _ in 0..n_o_d { tmp_digits.push(Digit{ digit: self.digits[0], neg: false }) }
      if f_len1 > f_len2 { self.add(neg, &mut tmp2_ints, &mut tmp1_ints, f_len2, f_len1, &mut tmp_digits, radix) }
      else               { self.add(neg, &mut tmp1_ints, &mut tmp2_ints, f_len1, f_len2, &mut tmp_digits, radix) }

      while let Some(d) = tmp_digits.pop() {
        if d.digit == self.digits[0] { continue }
        if d.neg { result.push(negc) }
        result.push(d.digit);
      }
      result.push(delim);
      state.pool.add_digits(tmp_digits);

      break
    }
    result.pop();
    Ok(result)
  }
}
