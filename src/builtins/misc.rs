use crate::*;
use std::{thread, time};

pub fn cog_nop(state: CognitionState, _: Option<&Value>) -> CognitionState { state }

pub fn cog_exit(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let word_v = v.value_stack_ref().first().unwrap();
  if !word_v.is_word() {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let code = state.string_copy(&word_v.vword_ref().str_word);
  state.exit_code = Some(code);
  state.exited = true;
  state
}

pub fn cog_reset(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  if let Some(wt) = state.current().word_table.take() {
    state.pool.add_word_table(wt)
  }
  builtins::add_builtins(&mut state);
  if let Some(cranks) = state.current().cranks.take() {
    state.pool.add_cranks(cranks)
  }
  if let Some(math) = state.current().math.take() {
    state.pool.add_math(math)
  }
  if let Some(faliases) = state.current().faliases.take() {
    state.pool.add_faliases(faliases);
  }
  state.current().faliases = state.default_faliases();
  if let Some(delims) = state.current().delims.take() {
    state.pool.add_string(delims)
  }
  if let Some(ignored) = state.current().ignored.take() {
    state.pool.add_string(ignored)
  }
  if let Some(singlets) = state.current().singlets.take() {
    state.pool.add_string(singlets)
  }
  let cur = state.current();
  cur.dflag = false;
  cur.iflag = true;
  cur.sflag = true;
  state
}

pub fn cog_getargs(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let mut vstack = state.pool.get_vstack(state.args.len());
  for s in state.args.iter() {
    let mut vword = state.pool.get_vword(s.vword_ref().str_word.len());
    vword.str_word.push_str(&s.vword_ref().str_word);
    vstack.container.stack.push(Value::Word(vword));
  }
  state.current().stack.push(Value::Stack(vstack));
  state
}

pub fn cog_setargs(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().iter().any(|x| !x.is_word()) { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let tmpstack = state.args;
  state.args = match v {
    Value::Stack(mut vstack) => {
      let tmp = vstack.container.stack;
      vstack.container.stack = tmpstack;
      state.pool.add_vstack(vstack);
      tmp
    },
    Value::Macro(mut vmacro) => {
      let tmp = vmacro.macro_stack;
      vmacro.macro_stack = tmpstack;
      state.pool.add_vmacro(vmacro);
      tmp
    },
    _ => bad_value_err!(),
  };
  state
}

pub fn cog_sleep(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let i = get_unsigned!(state, w, isize, ACTIVE) as usize;
  if i > u64::MAX as usize {
    return state.eval_error("OUT OF BOUNDS", w);
  } else {
    let v = state.current().stack.pop().unwrap();
    state.pool.add_val(v);
  }
  thread::sleep(time::Duration::from_secs(i as u64));
  state
}
pub fn cog_msleep(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let i = get_unsigned!(state, w, isize, ACTIVE) as usize;
  if i > u64::MAX as usize {
    return state.eval_error("OUT OF BOUNDS", w);
  } else {
    let v = state.current().stack.pop().unwrap();
    state.pool.add_val(v);
  }
  thread::sleep(time::Duration::from_millis(i as u64));
  state
}
pub fn cog_usleep(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let i = get_unsigned!(state, w, isize, ACTIVE) as usize;
  if i > u64::MAX as usize {
    return state.eval_error("OUT OF BOUNDS", w);
  } else {
    let v = state.current().stack.pop().unwrap();
    state.pool.add_val(v);
  }
  thread::sleep(time::Duration::from_micros(i as u64));
  state
}
pub fn cog_nanosleep(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let i = get_unsigned!(state, w, isize, ACTIVE) as usize;
  if i > u64::MAX as usize {
    return state.eval_error("OUT OF BOUNDS", w);
  } else {
    let v = state.current().stack.pop().unwrap();
    state.pool.add_val(v);
  }
  thread::sleep(time::Duration::from_nanos(i as u64));
  state
}

pub fn cog_void(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  state.push_quoted(Value::Custom(VCustom::with_void()));
  state
}

pub fn cog_void_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let Value::Custom(vcustom) = v.value_stack_ref().first().unwrap() else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let vword = if vcustom.custom.as_any().downcast_ref::<Void>().is_some() {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('t');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_custom_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let vword = if v.value_stack_ref().first().unwrap().is_custom() {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('t');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_control_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let vword = if v.value_stack_ref().first().unwrap().is_control() {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('t');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_same_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v2 = stack.last().unwrap();
  let v1 = stack.get(stack.len() - 2).unwrap();
  if v1.value_stack_ref().len() != 1 || v2.value_stack_ref().len() != 1 {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let truth = match (v1.value_stack_ref().first().unwrap(), v2.value_stack_ref().first().unwrap()) {
    (Value::Control(vcontrol1),Value::Control(vcontrol2)) => vcontrol1 == vcontrol2,
    (Value::Control(_),Value::FLLib(_)) => false,
    (Value::FLLib(_),Value::Control(_)) => false,
    (Value::FLLib(vfllib1),Value::FLLib(vfllib2)) => vfllib1.fllib == vfllib2.fllib,
    _ => return state.eval_error("BAD ARGUMENT TYPE", w),
  };
  let vword = if truth {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('t');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_var(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut vw = get_word!(state, w);
  let vword = vw.value_stack_ref().first().unwrap().vword_ref();
  match std::env::var(&vword.str_word) {
    Ok(val) => {
      let mut vword = state.pool.get_vword(val.len());
      vword.str_word.push_str(&val);
      state.pool.add_val(vw);
      state.push_quoted(Value::Word(vword));
    },
    Err(_) => {
      let v = vw.value_stack().pop().unwrap();
      state.pool.add_val(v);
      state.current().stack.push(vw);
    }
  }
  state
}

pub fn cog_getp(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(math) = state.current().math.take() else { return state.eval_error("MATH BASE ZERO", w) };
  if math.base() == 0 {
    state.current().math = Some(math);
    return state.eval_error("MATH BASE ZERO", w)
  }
  let array = state.pool.get_capacity();
  if math.base() == 1 && array.iter().any(|x| *x != 0) {
    state.current().math = Some(math);
    return state.eval_error("INVALID NUMBER STRING", w)
  }
  let mut vword = state.pool.get_vword(64);
  for i in array {
    let s = match math.itos(i, &mut state) {
      Ok(s) => s,
      Err(e) => {
        state.current().math = Some(math);
        return state.eval_error(e, w)
      }
    };
    vword.str_word.push_str(&s);
    state.pool.add_string(s);
    vword.str_word.push(math.get_delim().expect("Math delim was None"));
  }
  vword.str_word.pop();
  state.current().math = Some(math);
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_setp(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_word!(state, w);
  let Some(math) = state.current().math.take() else {
    state.current().stack.push(v);
    return state.eval_error("MATH BASE ZERO", w)
  };
  if math.base() == 0 {
    state.current().math = Some(math);
    state.current().stack.push(v);
    return state.eval_error("MATH BASE ZERO", w)
  }
  let mut array: [isize;32] = [0;32];
  let mut idx_beg = 0;
  let mut array_idx = 0;
  let string = &v.value_stack_ref().first().unwrap().vword_ref().str_word;
  for (i, c) in string.char_indices() {
    if array_idx >= 32 { break }
    if c == math.get_delim().unwrap() {
      match math.stoi(&string[idx_beg..i]) {
        Ok(int) => array[array_idx] = int,
        Err(e) => {
          state.current().math = Some(math);
          state.current().stack.push(v);
          return state.eval_error(e, w)
        }
      }
      idx_beg = i + c.len_utf8();
      array_idx += 1;
    }
  }
  if array_idx < 32 {
    match math.stoi(&string[idx_beg..]) {
      Ok(int) => array[array_idx] = int,
      Err(e) => {
        state.current().math = Some(math);
        state.current().stack.push(v);
        return state.eval_error(e, w)
      }
    }
  }
  state.current().math = Some(math);
  state.pool.set_capacity(array);
  state
}

pub fn add_builtins(state: &mut CognitionState) {
  add_builtin!(state, "nothing");
  add_builtin!(state, "nop", cog_nop);
  add_builtin!(state, "ghost", GHOST);
  add_builtin!(state, "return", RETURN);
  add_builtin!(state, "exit", cog_exit);
  add_builtin!(state, "reset", cog_reset);
  add_builtin!(state, "getargs", cog_getargs);
  add_builtin!(state, "setargs", cog_setargs);
  add_builtin!(state, "sleep", cog_sleep);
  add_builtin!(state, "msleep", cog_msleep);
  add_builtin!(state, "μsleep", cog_usleep);
  add_builtin!(state, "nanosleep", cog_nanosleep);
  add_builtin!(state, "void", cog_void);
  add_builtin!(state, "void?", cog_void_questionmark);
  add_builtin!(state, "custom?", cog_custom_questionmark);
  add_builtin!(state, "control?", cog_control_questionmark);
  add_builtin!(state, "same?", cog_same_questionmark);
  add_builtin!(state, "var", cog_var);
  add_builtin!(state, "getp", cog_getp);
  add_builtin!(state, "setp", cog_setp);
}
