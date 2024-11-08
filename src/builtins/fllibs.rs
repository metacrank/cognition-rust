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
  let stack = &mut state.current().stack;
  let filename = &v1.value_stack_ref().first().unwrap().vword_ref().str_word;
  let lib_name = &v2.value_stack_ref().first().unwrap().vword_ref().str_word;
  unsafe {
    let Ok(lib) = libloading::Library::new(filename) else {
      stack.push(v1);
      stack.push(v2);
      return state.eval_error("INVALID FILENAME", w)
    };
    let fllib_add_words: libloading::Symbol<AddWordsFn> = match lib.get(b"add_words\0") {
      Ok(f) => f,
      Err(_) => {
        stack.push(v1);
        stack.push(v2);
        return state.eval_error("INVALID FLLIB", w)
      },
    };
    fllib_add_words(&mut state, &Arc::new(lib), lib_name);
  }
  state.pool.add_val(v1);
  state.pool.add_val(v2);
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

pub fn cog_get_fllibs(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let mut vstack = state.pool.get_vstack(DEFAULT_STACK_SIZE);
  let mut cur_v = state.pop_cur();
  for s in cur_v.metastack_container().fllibs.keys() {
    let mut vword = state.pool.get_vword(s.len());
    vword.str_word.push_str(s);
    vstack.container.stack.push(Value::Word(vword));
  }
  cur_v.metastack_container().stack.push(Value::Stack(vstack));
  state.push_cur(cur_v)
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
    let mut vword = state.pool.get_vword(library.lib_name.len());
    vword.str_word.push_str(&library.lib_name);
    vstack.push(Value::Word(vword))
  }
  state.current().stack.push(v);
  state.pool.add_val(vfirst);
  state
}

pub fn add_builtins(state: &mut CognitionState) {
  add_builtin!(state, "fllib?", cog_fllib_questionmark);
  add_builtin!(state, "fllib", cog_fllib);
  add_builtin!(state, "fllib-filename", cog_fllib_filename);
  add_builtin!(state, "get-fllibs", cog_get_fllibs);
  add_builtin!(state, "name", cog_name);
  add_builtin!(state, "library", cog_library);
}
