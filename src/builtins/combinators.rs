use crate::*;

pub fn cog_eval(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() == 0 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  state.control.eval();
  state
}

pub fn cog_quote(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let mut wrapper = state.pool.get_vstack(1);
  wrapper.container.stack.push(v);
  state.current().stack.push(Value::Stack(wrapper));
  state
}

pub fn cog_child(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let mut cur_v = state.pop_cur();
  let cur = cur_v.metastack_container();
  let mut vstack = state.pool.get_vstack(0);
  state.contain_copy_attributes(cur, &mut vstack.container);
  cur.stack.push(Value::Stack(vstack));
  state.push_cur(cur_v)
}

pub fn cog_stack(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let vstack = state.pool.get_vstack(DEFAULT_STACK_SIZE);
  state.current().stack.push(Value::Stack(vstack));
  state
}

pub fn cog_macro(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let vmacro = state.pool.get_vmacro(DEFAULT_STACK_SIZE);
  state.current().stack.push(Value::Macro(vmacro));
  state
}

pub fn cog_sub(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let mut vstack = state.pool.get_vstack(DEFAULT_STACK_SIZE);
  vstack.container.faliases = state.default_faliases();
  state.current().stack.push(Value::Stack(vstack));
  super::add_builtins(&mut state);
  state
}

pub fn cog_cast(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v2) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(v1) = stack.pop() else {
    stack.push(v2);
    return state.eval_error("TOO FEW ARGUMENTS", w)
  };
  let v2stack = v2.value_stack();
  if v2stack.len() == 1 {
    if let Some(Value::Word(vword)) = v2stack.first() {
      if vword.str_word.as_str() == "VMACRO" {
        if let Value::Stack(mut vstack) = v1 {
          let mut new_v1 = state.pool.get_vmacro(0);
          let tmpstack = new_v1.macro_stack;
          new_v1.macro_stack = vstack.container.stack;
          vstack.container.stack = tmpstack;
          state.pool.add_val(Value::Stack(vstack));
          state.current().stack.push(Value::Macro(new_v1));
        } else { stack.push(v1) }
        state.pool.add_val(v2);
        return state
      } else if vword.str_word.as_str() == "VSTACK" {
        if let Value::Macro(mut vmacro) = v1 {
          let mut new_v1 = state.pool.get_vstack(0);
          let tmpstack = new_v1.container.stack;
          new_v1.container.stack = vmacro.macro_stack;
          vmacro.macro_stack = tmpstack;
          state.pool.add_val(Value::Macro(vmacro));
          state.current().stack.push(Value::Stack(new_v1));
        } else { stack.push(v1) }
        state.pool.add_val(v2);
        return state
      }
    }
  }
  stack.push(v1);
  stack.push(v2);
  state.eval_error("BAD ARGUMENT TYPE", w)
}

pub fn cog_compose(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v2) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(v1) = stack.last_mut() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  v2.value_stack().reverse();
  while let Some(v) = v2.value_stack().pop() {
    v1.value_stack().push(v);
  }
  state.pool.add_val(v2);
  state
}

pub fn cog_prepose(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v2) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(v1) = stack.last_mut() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  std::mem::swap(v1.value_stack(), v2.value_stack());
  v2.value_stack().reverse();
  while let Some(v) = v2.value_stack().pop() {
    v1.value_stack().push(v);
  }
  state.pool.add_val(v2);
  state
}

pub fn cog_displace(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() < 4 { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let (i, j) = get_2_unsigned!(state, w, isize, ACTIVE);
  let i = i as usize;
  let j = j as usize;
  let stack = &mut state.current().stack;
  let j_val = stack.pop().unwrap();
  let i_val = stack.pop().unwrap();
  let mut v2 = stack.pop().unwrap();
  let mut v1 = stack.pop().unwrap();
  if i > v1.value_stack_ref().len() || j > v1.value_stack_ref().len() {
    stack.push(v1);
    stack.push(v2);
    stack.push(i_val);
    stack.push(j_val);
    return state.eval_error("OUT OF BOUNDS", w)
  }
  let iter = v1.value_stack().splice(i..j, v2.value_stack().drain(..));
  let mut tmp_stack = state.pool.get_stack(j - i);
  for v in iter { tmp_stack.push(v) }
  state.pool.add_stack(std::mem::replace(v2.value_stack(), tmp_stack));
  state.pool.add_val(i_val);
  state.pool.add_val(j_val);
  state.current().stack.push(v1);
  state.current().stack.push(v2);
  state
}

pub fn cog_invert(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.last_mut() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  v.value_stack().reverse();
  state
}

pub fn cog_dip(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let vdip = state.current().stack.pop().unwrap();
  let v = state.current().stack.pop().unwrap();
  state.current().stack.push(vdip);
  if let Some(wd) = state.get_evalf_val(w) {
    state = state.evalstack(wd, w, false)
  }
  state.current().stack.push(v);
  state
}

pub fn cog_if(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  if stack.len() < 3 { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v2 = stack.pop().unwrap();
  let v1 = stack.pop().unwrap();
  let v_truth = stack.pop().unwrap();
  if v_truth.value_stack_ref().len() != 1 {
    stack.push(v_truth);
    stack.push(v1);
    stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let Value::Word(vword_truth) = v_truth.value_stack_ref().first().unwrap() else {
    stack.push(v_truth);
    stack.push(v1);
    stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let truth = vword_truth.str_word.len() > 0;
  state.pool.add_val(v_truth);
  if truth {
    state.current().stack.push(v1);
    state.pool.add_val(v2);
  } else {
    state.current().stack.push(v2);
    state.pool.add_val(v1);
  }
  state.control.eval();
  state
}

pub fn cog_split(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let i = get_unsigned!(state, w, isize, ACTIVE) as usize;
  let stack = &mut state.current().stack;
  let i_val = stack.pop().unwrap();
  let mut v1 = stack.pop().unwrap();
  if i > v1.value_stack_ref().len() {
    stack.push(v1);
    stack.push(i_val);
    return state.eval_error("OUT OF BOUNDS", w)
  }
  state.pool.add_val(i_val);
  let mut new_v = if let Value::Stack(ref vstack) = v1 {
    let mut new_vstack = state.pool.get_vstack(vstack.container.stack.len() - i);
    state.contain_copy_attributes(&vstack.container, &mut new_vstack.container);
    Value::Stack(new_vstack)
  } else {
    Value::Macro(state.pool.get_vmacro(v1.vmacro_ref().macro_stack.len() - i))
  };
  let v1stack = v1.value_stack();
  let new_v_stack = new_v.value_stack();
  for _ in i..v1stack.len() {
    new_v_stack.push(v1stack.pop().unwrap());
  }
  new_v_stack.reverse();
  state.current().stack.push(v1);
  state.current().stack.push(new_v);
  state
}

pub fn cog_vat(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let i = get_unsigned!(state, w, isize, ACTIVE) as usize;
  let stack = &mut state.current().stack;
  let i_val = stack.pop().unwrap();
  let v1 = stack.pop().unwrap();
  if i >= v1.value_stack_ref().len() {
    stack.push(v1);
    stack.push(i_val);
    return state.eval_error("OUT OF BOUNDS", w)
  }
  state.pool.add_val(i_val);
  let v_new = state.value_copy(&v1.value_stack_ref()[i]);
  state.current().stack.push(v1);
  state.push_quoted(v_new);
  state
}

pub fn cog_substack(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() < 3 { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let (i, j) = get_2_unsigned!(state, w, isize, ACTIVE);
  if i < 0 || j < 0 { return state.eval_error("OUT OF BOUNDS", w) }
  let i = i as usize;
  let j = j as usize;
  let stack = &mut state.current().stack;
  let j_val = stack.pop().unwrap();
  let i_val = stack.pop().unwrap();
  let mut v1 = stack.pop().unwrap();
  let length = v1.value_stack_ref().len();
  if i > length || j > length {
    stack.push(v1);
    stack.push(i_val);
    stack.push(j_val);
    return state.eval_error("OUT OF BOUNDS", w)
  }
  state.pool.add_val(i_val);
  state.pool.add_val(j_val);

  let v1stack = v1.value_stack();
  for _ in j..length {
    state.pool.add_val(v1stack.pop().unwrap())
  }
  let i = if i > v1stack.len() { v1stack.len() } else { i };
  for v in v1stack.drain(..i) {
    state.pool.add_val(v);
  }
  state.current().stack.push(v1);
  state
}

// Empty stack is pushed after elements which
// retains the properties of the original stack
pub fn cog_uncompose(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(mut v1) = state.current().stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v1stack = v1.value_stack();
  while let Some(v) = v1stack.pop() {
    state.push_quoted(v)
  }
  state.current().stack.push(v1);
  state
}

pub fn cog_decompose(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(Value::Stack(mut vstack)) = state.current().stack.pop() else {
    return state.eval_error("TOO FEW ARGUMENTS", w)
  };
  state.ensure_quoted(&mut vstack.container.stack);
  state.current().stack.push(Value::Stack(vstack));
  state
}

pub fn cog_size(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() == 0 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let length = state.current_ref().stack.last().unwrap().value_stack_ref().len();
  let Some(mathborrower) = state.get_math() else { return state.eval_error("MATH BASE ZERO", w) };
  if mathborrower.math().base() == 0 { return state.with_math(mathborrower).eval_error("MATH BASE ZERO", w) }
  if length > isize::MAX as usize { return state.with_math(mathborrower).eval_error("OUT OF BOUNDS", w) }
  match mathborrower.math().itos(length as isize, &mut state) {
    Ok(s) => {
      state.set_math(mathborrower);
      let mut v = state.pool.get_vword(s.len());
      v.str_word.push_str(&s);
      state.pool.add_string(s);
      state.push_quoted(Value::Word(v));
      state
    },
    Err(e) => {
      return state.with_math(mathborrower).eval_error(e, w)
    }
  }
}

pub fn cog_type(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let mut vword = state.pool.get_vword(6);
  if v.is_stack() {
    vword.str_word.push_str("VSTACK")
  } else {
    vword.str_word.push_str("VMACRO")
  }
  state.current().stack.push(v);
  state.push_quoted(Value::Word(vword));
  state
}

pub fn add_builtins(state: &mut CognitionState) {
  add_builtin!(state, "eval", cog_eval);
  add_builtin!(state, "quote", cog_quote);
  add_builtin!(state, "child", cog_child);
  add_builtin!(state, "stack", cog_stack);
  add_builtin!(state, "macro", cog_macro);
  add_builtin!(state, "sub", cog_sub);
  add_builtin!(state, "cast", cog_cast);
  add_builtin!(state, "compose", cog_compose);
  add_builtin!(state, "prepose", cog_prepose);
  add_builtin!(state, "displace", cog_displace);
  add_builtin!(state, "invert", cog_invert);
  add_builtin!(state, "if", cog_if);
  add_builtin!(state, "dip", cog_dip);
  add_builtin!(state, "split", cog_split);
  add_builtin!(state, "vat", cog_vat);
  add_builtin!(state, "substack", cog_substack);
  add_builtin!(state, "uncompose", cog_uncompose);
  add_builtin!(state, "decompose", cog_decompose);
  add_builtin!(state, "size", cog_size);
  add_builtin!(state, "type", cog_type);
}
