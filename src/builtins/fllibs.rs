use crate::*;
use libloading;

pub fn cog_fllib_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let vword = if v.value_stack_ref().first().unwrap().is_fllib() {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('t');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_fllib(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let (v1, v2) = get_2_words!(state, w);
  let lib_name = &v1.value_stack_ref().first().unwrap().vword_ref().str_word;
  let filename = &v2.value_stack_ref().first().unwrap().vword_ref().str_word;

  match unsafe { state.load_fllib(lib_name, filename) } {
    Some(e) => {
      state.current().stack.push(v1);
      state.current().stack.push(v2);
      return state.eval_error(e, w)
    },
    None => {
      state.pool.add_val(v1);
      state.pool.add_val(v2);
    }
  }
  state
}

pub fn cog_unload(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_word!(state, w);
  let name = &v.value_stack_ref().first().unwrap().vword_ref().str_word;

  if let Some(libraries) = &mut state.fllibs { libraries.remove(name); }
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

pub fn cog_fllibs(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let mut vstack = state.pool.get_vstack(DEFAULT_STACK_SIZE);
  if let Some(libs) = state.fllibs.take() {
    for (k, l) in libs.iter() {
      let mut vword_name = state.pool.get_vword(k.len());
      vword_name.str_word.push_str(k);
      let mut vword_path = state.pool.get_vword(l.library.lib_path.len());
      vword_path.str_word.push_str(&l.library.lib_path);
      let mut tmp_vstack = state.pool.get_vstack(2);
      tmp_vstack.container.stack.push(Value::Word(vword_name));
      tmp_vstack.container.stack.push(Value::Word(vword_path));
      vstack.container.stack.push(Value::Stack(tmp_vstack));
    }
    state.fllibs = Some(libs)
  }
  state.current().stack.push(Value::Stack(vstack));
  state
}

pub fn cog_name(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let vstack = v.value_stack();
  if vstack.len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let vfirst = vstack.pop().unwrap();
  if !vfirst.is_fllib() {
    vstack.push(vfirst);
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  if let Some(ref s) = vfirst.vfllib_ref().str_word {
    let mut vword = state.pool.get_vword(s.len());
    vword.str_word.push_str(s);
    vstack.push(Value::Word(vword))
  }
  state.current().stack.push(v);
  state.pool.add_val(vfirst);
  state
}

pub fn cog_set_name(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let vname = get_word!(state, w);
  let stack = &mut state.current().stack;
  let mut v = stack.pop().unwrap();
  let vstack = v.value_stack();
  if vstack.len() != 1 {
    stack.push(v);
    stack.push(vname);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let vfirst = vstack.first_mut().unwrap();
  if !vfirst.is_fllib() {
    stack.push(v);
    stack.push(vname);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  if let Some(s) = vfirst.vfllib_mut().str_word.take() {
    state.pool.add_string(s);
  }
  let name = &vname.value_stack_ref().first().unwrap().vword_ref().str_word;
  vfirst.vfllib_mut().str_word = Some(state.string_copy(name));
  state.current().stack.push(v);
  state.pool.add_val(vname);
  state
}

pub fn cog_library(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let vstack = v.value_stack();
  if vstack.len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let vfirst = vstack.pop().unwrap();
  if !vfirst.is_fllib() {
    vstack.push(vfirst);
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  if let Some(ref library) = vfirst.vfllib_ref().library {
    let mut vword = state.pool.get_vword(library.lib_path.len());
    vword.str_word.push_str(&library.lib_path);
    vstack.push(Value::Word(vword));
    let mut vword = state.pool.get_vword(library.lib_name.len());
    vword.str_word.push_str(&library.lib_name);
    vstack.push(Value::Word(vword));
  }
  state.current().stack.push(v);
  state.pool.add_val(vfirst);
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

pub fn add_builtins(state: &mut CognitionState) {
  add_builtin!(state, "fllib?", cog_fllib_questionmark);
  add_builtin!(state, "fllib", cog_fllib);
  add_builtin!(state, "unload", cog_unload);
  add_builtin!(state, "fllib-filename", cog_fllib_filename);
  add_builtin!(state, "fllibs", cog_fllibs);
  add_builtin!(state, "name", cog_name);
  add_builtin!(state, "set-name", cog_set_name);
  add_builtin!(state, "library", cog_library);
  add_builtin!(state, "same?", cog_same_questionmark);
}
