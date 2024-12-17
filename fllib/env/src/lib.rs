use cognition::*;
use std::env::*;
use std::path::Path;

pub fn cog_set_var(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let (v1, v2) = get_2_words!(state, w);
  let vword1 = v1.value_stack_ref().first().unwrap().vword_ref();
  let vword2 = v2.value_stack_ref().first().unwrap().vword_ref();
  set_var(&vword1.str_word, &vword2.str_word);
  state.pool.add_val(v1);
  state.pool.add_val(v2);
  state
}

pub fn cog_remove_var(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let vw = get_word!(state, w);
  let vword = vw.value_stack_ref().first().unwrap().vword_ref();
  remove_var(&vword.str_word);
  state.pool.add_val(vw);
  state
}

pub fn cog_current_dir(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Ok(current) = current_dir() else {
    return state.eval_error("INVALID CURRENT DIRECTORY", w)
  };
  let Ok(mut string) = current.into_os_string().into_string() else {
    return state.eval_error("INVALID STRING", w)
  };
  let mut vw = state.pool.get_vword(0);
  std::mem::swap(&mut string, &mut vw.str_word);
  state.pool.add_string(string);
  state.push_quoted(Value::Word(vw));
  state
}

pub fn cog_set_current_dir(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let vw = get_word!(state, w);
  let vword = vw.value_stack_ref().first().unwrap().vword_ref();
  if set_current_dir(Path::new(&vword.str_word)).is_err() {
    state.current().stack.push(vw);
    return state.eval_error("CHANGE DIRECTORY FAILED", w)
  }
  state.pool.add_val(vw);
  state
}

#[no_mangle]
pub extern fn add_words(state: &mut CognitionState, lib: &Library) {
  ensure_foreign_library!(state, lib);
  add_word!(state, lib, "set-var", cog_set_var);
  add_word!(state, lib, "remove-var", cog_remove_var);
  add_word!(state, lib, "current-dir", cog_current_dir);
  add_word!(state, lib, "set-current-dir", cog_set_current_dir);
}
