use crate::*;
use std::{thread, time};
use libloading;

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
  let i = get_int!(state, w);
  thread::sleep(time::Duration::from_secs(i as u64));
  state
}
pub fn cog_usleep(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let i = get_int!(state, w);
  thread::sleep(time::Duration::from_micros(i as u64));
  state
}
pub fn cog_nanosleep(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let i = get_int!(state, w);
  thread::sleep(time::Duration::from_nanos(i as u64));
  state
}

pub fn cog_fllib(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v_stack = v.value_stack_ref();
  if v_stack.len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let word_v = v_stack.first().unwrap();
  if !word_v.is_word() {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  unsafe {
    let Ok(lib) = libloading::Library::new(&word_v.vword_ref().str_word) else {
      stack.push(v);
      return state.eval_error("INVALID FILENAME", w)
    };
    let fllib_add_words: libloading::Symbol<unsafe extern fn(&mut CognitionState)> = match lib.get(b"add_words") {
      Ok(f) => f,
      Err(_) => {
        stack.push(v);
        return state.eval_error("INVALID FLLIB", w)
      },
    };
    fllib_add_words(&mut state);
    state.fllibs.push(lib);
  }
  state.pool.add_val(v);
  state
}

pub fn cog_fllib_filename(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v_stack = v.value_stack();
  if v_stack.len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let word_v = v_stack.pop().unwrap();
  if !word_v.is_word() {
    v_stack.push(word_v);
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let mut vword = word_v.vword();
  vword.str_word = libloading::library_filename(vword.str_word).into_string().unwrap();
  v_stack.push(Value::Word(vword));
  stack.push(v);
  state
}

pub fn cog_fllib_count(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut cur_v = state.pop_cur();
  let cur = cur_v.metastack_container();
  if cur.math.is_none() { return state.push_cur(cur_v).eval_error("MATH BASE UNINITIALIZED", w) }
  if cur.math.as_ref().unwrap().base() == 0 { return state.push_cur(cur_v).eval_error("MATH BASE ZERO", w) }
  if cur.math.as_ref().unwrap().base() == 1 { return state.push_cur(cur_v).eval_error("MATH BASE ONE", w) }
  let length = state.fllibs.len();
  if length > isize::MAX as usize { return state.push_cur(cur_v).eval_error("OUT OF BOUNDS", w) }
  match cur.math.as_ref().unwrap().itos(length as isize, &mut state) { // TODO: converts usize to isize
    Ok(s) => {
      let mut v = state.pool.get_vword(s.len());
      v.str_word.push_str(&s);
      state.pool.add_string(s);
      state = state.push_cur(cur_v);
      state.push_quoted(Value::Word(v));
      state
    },
    Err(e) => { return state.push_cur(cur_v).eval_error(e, w) }
  }
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "nothing");
  add_word!(state, "nop", cog_nop);
  add_word!(state, "return", RETURN);
  add_word!(state, "exit", cog_exit);
  add_word!(state, "reset", cog_reset);
  add_word!(state, "getargs", cog_getargs);
  add_word!(state, "setargs", cog_setargs);
  add_word!(state, "sleep", cog_sleep);
  add_word!(state, "usleep", cog_usleep);
  add_word!(state, "nanosleep", cog_nanosleep);
  add_word!(state, "fllib", cog_fllib);
  add_word!(state, "fllib-filename", cog_fllib_filename);
  add_word!(state, "fllib-count", cog_fllib_count);
}
