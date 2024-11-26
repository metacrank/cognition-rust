use crate::*;
use crate::math::BASE_MAX;

macro_rules! ensure_math {
  ($state:ident) => {
    if $state.current_ref().math.is_none() {
      $state.current().math = Some($state.pool.get_math(0))
    }
  };
}

pub fn cog_base(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let cur = state.current();
  let Some(v) = cur.stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(math) = &mut cur.math else { return state.eval_error("MATH DIGITS UNINITIALIZED", w) };
  let i = if math.base() == 0 {
    let i = v.value_stack_ref().len();
    if i > BASE_MAX as usize { return state.eval_error("OUT OF BOUNDS", w) }
    i as i32
  } else {
    if v.value_stack_ref().len() != 1 {
      return state.eval_error("BAD ARGUMENT TYPE", w)
    }
    let base_v = &v.value_stack_ref()[0];
    if !base_v.is_word() {
      return state.eval_error("BAD ARGUMENT TYPE", w)
    }
    match math.stoi(&base_v.vword_ref().str_word) {
      Ok(i) => if i > BASE_MAX as isize || i < 0 {
        return state.eval_error("MATH DIGITS UNINITIALIZED", w);
      } else { i as i32 },
      Err(e) => return state.eval_error(e, w),
    }
  };
  if let Some(e) = math.set_base(i) {
    return state.eval_error(e, w)
  }
  let v = cur.stack.pop().unwrap();
  state.pool.add_val(v);
  state
}

pub fn cog_negc(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  get_char_option!(state, c, w);
  ensure_math!(state);
  if let Some(c) = c { state.current().math.as_mut().unwrap().set_negc(c) }
  else if let Some(e) = state.current().math.as_mut().unwrap().unset_negc() {
    return state.eval_error(e, w)
  }
  state
}
pub fn cog_radix(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  get_char_option!(state, c, w);
  ensure_math!(state);
  if let Some(c) = c { state.current().math.as_mut().unwrap().set_radix(c) }
  else if let Some(e) = state.current().math.as_mut().unwrap().unset_radix() {
    return state.eval_error(e, w)
  }
  state
}
// Cayley-Dickson delimiter
pub fn cog_cd_delim(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  get_char_option!(state, c, w);
  ensure_math!(state);
  if let Some(c) = c { state.current().math.as_mut().unwrap().set_delim(c) }
  else if let Some(e) = state.current().math.as_mut().unwrap().unset_delim() {
    return state.eval_error(e, w)
  }
  state
}
// Polynomial "radix point"
pub fn cog_meta_radix(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  get_char_option!(state, c, w);
  ensure_math!(state);
  if let Some(c) = c { state.current().math.as_mut().unwrap().set_meta_radix(c) }
  else if let Some(e) = state.current().math.as_mut().unwrap().unset_meta_radix() {
    return state.eval_error(e, w)
  }
  state
}
// Polynomial delimiter
pub fn cog_meta_delim(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  get_char_option!(state, c, w);
  ensure_math!(state);
  if let Some(c) = c { state.current().math.as_mut().unwrap().set_meta_delim(c) }
  else if let Some(e) = state.current().math.as_mut().unwrap().unset_meta_delim() {
    return state.eval_error(e, w)
  }
  state
}

pub fn cog_digits(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut cur_v = state.pop_cur();
  let cur = cur_v.metastack_container();
  let Some(v) = cur.stack.last() else { return state.push_cur(cur_v).eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.push_cur(cur_v).eval_error("BAD ARGUMENT TYPE", w) }
  let word_v = &v.value_stack_ref()[0];
  if !word_v.is_word() { return state.push_cur(cur_v).eval_error("BAD ARGUMENT TYPE", w) }
  let s = &word_v.vword_ref().str_word;
  if s.len() > i32::MAX as usize { return state.push_cur(cur_v).eval_error("OUT OF BOUNDS", w) }
  if cur.math.is_none() {
    cur.math = Some(state.pool.get_math(0))
  }
  if cur.math.as_ref().unwrap().base() != 0 { return state.push_cur(cur_v).eval_error("MATH BASE NONZERO", w) }
  cur.math.as_mut().unwrap().set_digits(s);
  let v = cur.stack.pop().unwrap();
  state.pool.add_val(v);
  state.push_cur(cur_v)
}

pub fn cog_get_base(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if let Some(math) = state.current().math.take() {
    if math.base() == 1 {
      state.current().math = Some(math);
      return state.eval_error("MATH BASE ONE", w)
    } else if math.base() > 1 {
      let one = math.get_digits().get(1).expect("Math missing digits");
      let zero = math.get_digits().first().expect("Math missing digits");
      let mut v = state.pool.get_vword(one.len_utf8() + zero.len_utf8());
      v.str_word.push(one.clone());
      v.str_word.push(zero.clone());
      state.current().math = Some(math);
      state.push_quoted(Value::Word(v));
      return state
    }
  }
  let v = state.pool.get_vstack(0);
  state.current().stack.push(Value::Stack(v));
  state
}

pub fn cog_get_negc(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  if let Some(math) = state.current().math.take() {
    if let Some(c) = math.get_negc() {
      let mut v = state.pool.get_vword(c.len_utf8());
      v.str_word.push(c.clone());
      state.current().math = Some(math);
      state.push_quoted(Value::Word(v));
      return state;
    }
    state.current().math = Some(math);
  }
  let v = state.pool.get_vword(0);
  state.push_quoted(Value::Word(v));
  state
}
pub fn cog_get_radix(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  if let Some(math) = state.current().math.take() {
    if let Some(c) = math.get_radix() {
      let mut v = state.pool.get_vword(c.len_utf8());
      v.str_word.push(c.clone());
      state.current().math = Some(math);
      state.push_quoted(Value::Word(v));
      return state;
    }
    state.current().math = Some(math);
  }
  let v = state.pool.get_vword(0);
  state.push_quoted(Value::Word(v));
  state
}
pub fn cog_get_cd_delim(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  if let Some(math) = state.current().math.take() {
    if let Some(c) = math.get_delim() {
      let mut v = state.pool.get_vword(c.len_utf8());
      v.str_word.push(c.clone());
      state.current().math = Some(math);
      state.push_quoted(Value::Word(v));
      return state;
    }
    state.current().math = Some(math);
  }
  let v = state.pool.get_vword(0);
  state.push_quoted(Value::Word(v));
  state
}
pub fn cog_get_meta_radix(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  if let Some(math) = state.current().math.take() {
    if let Some(c) = math.get_meta_radix() {
      let mut v = state.pool.get_vword(c.len_utf8());
      v.str_word.push(c.clone());
      state.current().math = Some(math);
      state.push_quoted(Value::Word(v));
      return state;
    }
    state.current().math = Some(math);
  }
  let v = state.pool.get_vword(0);
  state.push_quoted(Value::Word(v));
  state
}
pub fn cog_get_meta_delim(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  if let Some(math) = state.current().math.take() {
    if let Some(c) = math.get_meta_delim() {
      let mut v = state.pool.get_vword(c.len_utf8());
      v.str_word.push(c.clone());
      state.current().math = Some(math);
      state.push_quoted(Value::Word(v));
      return state;
    }
    state.current().math = Some(math);
  }
  let v = state.pool.get_vword(0);
  state.push_quoted(Value::Word(v));
  state
}


pub fn cog_get_digits(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  if let Some(math) = state.current().math.take() {
    let digits = math.get_digits();
    let mut v = state.pool.get_vword(digits.len() * 2);
    for d in digits.iter() {
      v.str_word.push(d.clone())
    }
    state.current().math = Some(math);
    state.push_quoted(Value::Word(v));
    return state;
  }
  let v = state.pool.get_vword(0);
  state.push_quoted(Value::Word(v));
  state
}

macro_rules! binary_logic_operation {
  ($name:tt,$a:tt,$b:tt,$op:expr) => {
    pub fn $name(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
      let stack = &mut state.current().stack;
      if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
      let v2 = stack.pop().unwrap();
      let v1 = stack.last_mut().unwrap();
      if v1.value_stack_ref().len() != 1 || v2.value_stack_ref().len() != 1 {
        return state.eval_error("TOO FEW ARGUMENTS", w)
      }
      let str1 = v1.value_stack().first_mut().unwrap();
      let str2 = v2.value_stack_ref().first().unwrap();
      if !str1.is_word() || !str2.is_word() {
        return state.eval_error("TOO FEW ARGUMENTS", w)
      }
      let vword1 = str1.vword_mut();
      let $a = &vword1.str_word;
      let $b = &str2.vword_ref().str_word;
      if $op {
        if vword1.str_word.len() == 0 {
          vword1.str_word.push('t');
        }
      } else {
        vword1.str_word.clear();
      }
      state.pool.add_val(v2);
      state
    }
  }
}

binary_logic_operation!{ cog_equals, a, b, a == b }
binary_logic_operation!{ cog_nequals, a, b, a != b }
binary_logic_operation!{ cog_and, a, b, a.len() != 0 && b.len() != 0 }
binary_logic_operation!{ cog_or, a, b, a.len() != 0 || b.len() != 0 }

pub fn cog_not(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut v = get_word!(state, w);
  let vw = v.value_stack().first_mut().unwrap();
  let str_word = &mut vw.vword_mut().str_word;
  if str_word.len() != 0 {
    str_word.clear()
  } else {
    str_word.push('t');
  }
  state.current().stack.push(v);
  state
}

macro_rules! interim_binary_operation {
  ($name:tt,$a:tt,$b:tt,$operation:expr) => {
    pub fn $name(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
      let ($a, $b) = get_2_ints!(state, w, isize);
      let math = state.get_math().unwrap();
      match math.math().itos(($operation) as isize, &mut state) {
        Ok(s) => {
          state.set_math(math);
          let mut vw = state.pool.get_vword(s.len());
          vw.str_word.push_str(&s);
          state.pool.add_string(s);
          state.push_quoted(Value::Word(vw));
          state
        },
        Err(e) => state.with_math(math).eval_error(e, w)
      }
    }
  }
}

// real integer operations
interim_binary_operation!{ cog_plus, a, b, a+b }
interim_binary_operation!{ cog_minus, a, b, a-b }
interim_binary_operation!{ cog_mul, a, b, a*b }
interim_binary_operation!{ cog_div, a, b, a/b }
interim_binary_operation!{ cog_pow, a, b, a.pow(b.try_into().unwrap()) }

pub fn cog_neg(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  get_word!(state, w, ACTIVE);
  let Some(math) = state.get_math() else { return state.eval_error("MATH BASE ZERO", w) };
  if math.math().base() == 0 { return state.with_math(math).eval_error("MATH BASE ZERO", w) }
  let mut v = state.current().stack.pop().unwrap();
  let vstr = &mut v.value_stack().first_mut().unwrap().vword_mut().str_word;
  math.math().neg(vstr, &mut state);
  state.set_math(math);
  state.current().stack.push(v);
  state
}

macro_rules! interim_comparison_operation {
  ($name:tt,$a:tt,$b:tt,$operation:expr) => {
    pub fn $name(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
      let ($a, $b) = get_2_ints!(state, w, isize);
      let mut vw = state.pool.get_vword(1);
      if $operation { vw.str_word.push('t') }
      state.push_quoted(Value::Word(vw));
      state
    }
  }
}

interim_comparison_operation!{ cog_lthan, a, b, a < b }
interim_comparison_operation!{ cog_leq, a, b, a <= b }
interim_comparison_operation!{ cog_eq, a, b, a == b }
interim_comparison_operation!{ cog_geq, a, b, a >= b }
interim_comparison_operation!{ cog_gthan, a, b, a > b }
interim_comparison_operation!{ cog_neq, a, b, a != b }

pub fn add_builtins(state: &mut CognitionState) {
  add_builtin!(state, "base", cog_base);
  add_builtin!(state, "negc", cog_negc);
  add_builtin!(state, "radix", cog_radix);
  add_builtin!(state, "cd-delim", cog_cd_delim);
  add_builtin!(state, "meta-radix", cog_meta_radix);
  add_builtin!(state, "meta-delim", cog_meta_delim);
  add_builtin!(state, "digits", cog_digits);
  add_builtin!(state, "get-base", cog_get_base);
  add_builtin!(state, "get-negc", cog_get_negc);
  add_builtin!(state, "get-radix", cog_get_radix);
  add_builtin!(state, "get-cd-delim", cog_get_cd_delim);
  add_builtin!(state, "get-meta-radix", cog_get_meta_radix);
  add_builtin!(state, "get-meta-delim", cog_get_meta_delim);
  add_builtin!(state, "get-digits", cog_get_digits);
  add_builtin!(state, "=", cog_equals);
  add_builtin!(state, "!=", cog_nequals);
  add_builtin!(state, "and", cog_and);
  add_builtin!(state, "or", cog_or);
  add_builtin!(state, "not", cog_not);
  add_builtin!(state, "neg", cog_neg);
  add_builtin!(state, "+", cog_plus);
  add_builtin!(state, "-", cog_minus);
  add_builtin!(state, "*", cog_mul);
  add_builtin!(state, "/", cog_div);
  add_builtin!(state, "pow", cog_pow);
  add_builtin!(state, "<", cog_lthan);
  add_builtin!(state, "<=", cog_leq);
  add_builtin!(state, "==", cog_eq);
  add_builtin!(state, ">=", cog_geq);
  add_builtin!(state, ">", cog_gthan);
  add_builtin!(state, "!==", cog_neq);
}
