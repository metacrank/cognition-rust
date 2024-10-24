macro_rules! get_char {
  ($state:ident,$c:pat,$w:ident) => {
    let cur = $state.current();
    let Some(v) = cur.stack.last() else { return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    if v.value_stack_ref().len() != 1 { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let word_v = &v.value_stack_ref()[0];
    if !word_v.is_word() { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let s = &word_v.vword_ref().str_word;
    let mut iter = s.chars();
    let Some($c) = iter.next() else { return $state.eval_error("BAD ARGUMENT TYPE", $w) };
    if iter.next().is_some() { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let v = cur.stack.pop().unwrap();
    $state.pool.add_val(v);
  };
}

macro_rules! get_char_option {
  ($state:ident,$c:pat,$w:ident) => {
    let cur = $state.current();
    let Some(v) = cur.stack.last() else { return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    if v.value_stack_ref().len() != 1 { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let word_v = &v.value_stack_ref()[0];
    if !word_v.is_word() { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let s = &word_v.vword_ref().str_word;
    let mut iter = s.chars();
    let tmp = iter.next();
    let $c = tmp;
    if tmp.is_some() && iter.next().is_some() { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let v = cur.stack.pop().unwrap();
    $state.pool.add_val(v);
  };
}

macro_rules! get_int {
  ($state:ident,$w:ident) => {{
    let cur = $state.current();
    let Some(v) = cur.stack.last() else { return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    let stack = v.value_stack_ref();
    if stack.len() != 1 {
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word_val = &stack[0];
    let Value::Word(vword) = word_val else {
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    };
    let Some(ref math) = cur.math else {
      return $state.eval_error("MATH BASE ZERO", $w)
    };
    if math.base() == 0 {
      return $state.eval_error("MATH BASE ZERO", $w)
    }
    let i = match math.stoi(&vword.str_word) {
      Ok(i) => if i > i32::MAX as isize || i < 0 {
        return $state.eval_error("OUT OF BOUNDS", $w)
      } else { i as i32 },
      Err(e) => return $state.eval_error(e, $w),
    };
    let v = cur.stack.pop().unwrap();
    $state.pool.add_val(v);
    i
  }};
}

macro_rules! get_2_ints {
  ($state:ident,$w:ident) => {{
    let cur = $state.current();
    let Some(v2) = cur.stack.pop() else { return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    let Some(v1) = cur.stack.pop() else {
      cur.stack.push(v2);
      return $state.eval_error("TOO FEW ARGUMENTS", $w);
    };
    let stack1 = v1.value_stack_ref();
    if stack1.len() != 1 {
      cur.stack.push(v1); cur.stack.push(v2);
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word_val1 = &stack1[0];
    let Value::Word(vword1) = word_val1 else {
      cur.stack.push(v1); cur.stack.push(v2);
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    };
    let stack2 = v2.value_stack_ref();
    if stack2.len() != 1 {
      cur.stack.push(v1); cur.stack.push(v2);
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word_val2 = &stack2[0];
    let Value::Word(vword2) = word_val2 else {
      cur.stack.push(v1); cur.stack.push(v2);
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    };
    let Some(ref math) = cur.math else {
      cur.stack.push(v1); cur.stack.push(v2);
      return $state.eval_error("MATH BASE ZERO", $w)
    };
    if math.base() == 0 {
      cur.stack.push(v1); cur.stack.push(v2);
      return $state.eval_error("MATH BASE ZERO", $w)
    }
    let i1 = match math.stoi(&vword1.str_word) {
      Ok(i) => if i > i32::MAX as isize || i < 0 {
        cur.stack.push(v1); cur.stack.push(v2);
        return $state.eval_error("OUT OF BOUNDS", $w)
      } else { i as i32 },
      Err(e) => {
        cur.stack.push(v1); cur.stack.push(v2);
        return $state.eval_error(e, $w)
      },
    };
    let i2 = match math.stoi(&vword2.str_word) {
      Ok(i) => if i > i32::MAX as isize || i < 0 {
        cur.stack.push(v1); cur.stack.push(v2);
        return $state.eval_error("OUT OF BOUNDS", $w)
      } else { i as i32 },
      Err(e) => {
        cur.stack.push(v1); cur.stack.push(v2);
        return $state.eval_error(e, $w)
      },
    };
    $state.pool.add_val(v1);
    $state.pool.add_val(v2);
    (i1, i2)
  }};
}

pub mod combinators;
pub mod cranker;
pub mod errors;
pub mod io;
pub mod math;
pub mod misc;
pub mod multithreading;
pub mod parser;
pub mod stackops;
pub mod strings;

use crate::CognitionState;

pub fn add_builtins(state: &mut CognitionState) {
  combinators::add_words(state);
  cranker::add_words(state);
  errors::add_words(state);
  io::add_words(state);
  math::add_words(state);
  misc::add_words(state);
  multithreading::add_words(state);
  parser::add_words(state);
  stackops::add_words(state);
  strings::add_words(state);
}
