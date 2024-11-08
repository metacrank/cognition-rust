use cognition::*;
use std::env::{set_var,remove_var};

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

#[no_mangle]
pub extern fn add_words(state: &mut CognitionState, lib: &Library, lib_name: &String) {
  add_word!(state, lib, lib_name, "set-var", cog_set_var);
  add_word!(state, lib, lib_name, "remove-var", cog_remove_var);
}
