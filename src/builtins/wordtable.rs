use crate::*;

pub fn cog_def(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v_body) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(v_name) = stack.pop() else {
    stack.push(v_body);
    return state.eval_error("TOO FEW ARGUMENTS", w)
  };
  let v_name_stack = v_name.value_stack_ref();
  if v_name_stack.len() != 1 {
    stack.push(v_name);
    stack.push(v_body);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  let Value::Word(name_vword) = v_name_stack.first().unwrap() else {
    stack.push(v_name);
    stack.push(v_body);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  };
  let name = state.string_copy(&name_vword.str_word);
  state.pool.add_val(v_name);

  state.def(v_body, name);
  state
}

pub fn cog_undef(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_word!(state, w);
  let Some(wt) = &mut state.current().word_table else { return state.eval_error("UNDEFINED WORD", w) };
  let Some(result) = wt.remove(&v.value_stack_ref().first().unwrap().vword_ref().str_word) else {
    return state.eval_error("UNDEFINED WORD", w)
  };
  state.pool.add_word_def(result);
  state.pool.add_val(v);
  state
}

pub fn cog_wordlist(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let v = if let Some(wt) = state.current().word_table.take() {
    let mut v = state.pool.get_vstack(wt.len());
    for k in wt.keys() {
      let mut word = state.pool.get_vword(k.len());
      word.str_word.push_str(k);
      v.container.stack.push(Value::Word(word));
    }
    state.current().word_table = Some(wt);
    v
  } else { state.pool.get_vstack(0) };
  state.current().stack.push(Value::Stack(v));
  state
}

pub fn cog_unglue(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let cur = state.current();
  let Some(v) = cur.stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let word_v = v.value_stack_ref().first().unwrap();
  if !word_v.is_word() { return state.eval_error("BAD ARGUMENT TYPE", w) }
  if cur.word_table.is_none() { return state.eval_error("UNDEFINED WORD", w) }
  let Some(wd) = cur.word_table.as_ref().unwrap().get(&word_v.vword_ref().str_word) else {
    return state.eval_error("UNDEFINED WORD", w)
  };
  let new_wd = wd.clone();
  let v = cur.stack.pop().unwrap();
  state.pool.add_val(v);
  let new_v = state.value_copy(&*new_wd);
  state.current().stack.push(new_v);
  state
}

pub fn cog_def_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut v = get_word!(state, w);
  let s = &mut v.value_stack().first_mut().unwrap().vword_mut().str_word;
  if let Some(wt) = &mut state.current().word_table {
    if wt.contains_key(&*s) {
      if s.len() == 0 { s.push('t') }
      state.current().stack.push(v);
      return state
    }
  }
  s.truncate(0);
  state.current().stack.push(v);
  state
}

pub fn cog_bequeath(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let cur = state.current();
  let stack = &mut cur.stack;
  let Some(mut v_words) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(mut v_child) = stack.pop() else {
    stack.push(v_words);
    return state.eval_error("TOO FEW ARGUMENTS", w)
  };
  let v_words_stack = v_words.value_stack();
  if !v_child.is_stack() || v_words_stack.iter().any(|x| !x.is_word()) {
    stack.push(v_child);
    stack.push(v_words);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  if v_words_stack.len() == 0 {
    state.pool.add_val(v_words);
    state.current().stack.push(v_child);
    return state
  }
  let closure = |x: &Value| if cur.word_table.is_some() {
    cur.word_table.as_mut().unwrap().get(&x.vword_ref().str_word).is_some()
  } else { false } || cur.isfalias(&x);

  if v_words_stack.iter().all(closure) {
    let v_child_container = &mut v_child.vstack_mut().container;
    if v_child_container.word_table.is_none() {
      v_child_container.word_table = Some(state.pool.get_word_table(v_words_stack.len() * 2));
    }
    let wt = v_child_container.word_table.as_mut().unwrap();
    for name_v in v_words_stack.iter() {
      let name = state.string_copy(&name_v.vword_ref().str_word);
      if let Some(wd) = state.current().word_table.as_mut().unwrap().get(&name) {
        wt.insert(name, wd.clone());
      } else {
        if v_child_container.faliases.is_none() {
          v_child_container.faliases = Some(state.pool.get_faliases(DEFAULT_FALIASES_SIZE));
        }
        v_child_container.faliases.as_mut().unwrap().insert(name);
      }
    }
    state.pool.add_val(v_words);
    state.current().stack.push(v_child);
    return state
  }
  state.current().stack.push(v_child);
  state.current().stack.push(v_words);
  state.eval_error("UNDEFINED WORD", w)
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "def", cog_def);
  add_word!(state, "undef", cog_undef);
  add_word!(state, "unglue", cog_unglue);
  add_word!(state, "def?", cog_def_questionmark);
  add_word!(state, "wordlist", cog_wordlist);
  add_word!(state, "bequeath", cog_bequeath);
}
