use crate::*;

pub fn cog_cd(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if !v.is_stack() {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  ensure_quoted!(state, v.vstack_mut().container.stack);
  state.stack.push(v);
  state
}

pub fn cog_ccd(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut stack_v = state.stack.pop().expect("Cognition metastack was empty");
  let stack = &mut stack_v.vstack_mut().container.stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if !v.is_stack() {
    stack.push(v);
    state.stack.push(stack_v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  state.pool.add_val(stack_v);
  ensure_quoted!(state, v.vstack_mut().container.stack);
  state.stack.push(v);
  state
}

pub fn cog_uncd(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let child = state.stack.pop().expect("Cognition metastack was empty");
  if state.stack.len() == 0 {
    let mut new_stack = state.pool.get_vstack(DEFAULT_STACK_SIZE);
    state.contain_copy_attributes(&child.vstack_ref().container, &mut new_stack.container);
    state.stack.push(Value::Stack(new_stack));
  }
  state.current().stack.push(child);
  state
}

pub fn cog_uncdf(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let child = state.stack.pop().expect("Cognition metastack was empty");
  if state.stack.len() == 0 {
    if let Some(chroot) = state.chroots.pop() {
      let tmpstack = state.stack;
      state.stack = chroot;
      state.pool.add_stack(tmpstack);
    } else {
      let mut new_stack = state.pool.get_vstack(DEFAULT_STACK_SIZE);
    state.contain_copy_attributes(&child.vstack_ref().container, &mut new_stack.container);
      state.stack.push(Value::Stack(new_stack));
    }
  }
  state.current().stack.push(child);
  state
}

pub fn cog_qstack(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let child = state.pop_cur();
  let mut new_stack = state.pool.get_vstack(DEFAULT_STACK_SIZE);
  state.contain_copy_attributes(&child.vstack_ref().container, &mut new_stack.container);
  state.stack.push(Value::Stack(new_stack));
  state.current().stack.push(child);
  state
}

pub fn cog_root(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let mut v = state.pop_cur();
  while let Some(mut new_v) = state.stack.pop() {
    new_v.vstack_mut().container.stack.push(v);
    v = new_v;
  }
  state.stack.push(v);
  state
}

pub fn cog_su(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  state = cog_root(state, None);
  while let Some(chroot) = state.chroots.pop() {
    let cur_v = state.pop_cur();
    let tmpstack = state.stack;
    state.stack = chroot;
    state.pool.add_stack(tmpstack);
    state.current().stack.push(cur_v);
    state = cog_root(state, None);
  }
  state
}

pub fn cog_chroot(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if !v.is_stack() {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  ensure_quoted!(state, v.vstack_mut().container.stack);
  let tmpstack = state.stack;
  state.chroots.push(tmpstack);
  let tmpstack = state.pool.get_stack(DEFAULT_STACK_SIZE);
  state.stack = tmpstack;
  state.stack.push(v);
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "cd", cog_cd);
  add_word!(state, "ccd", cog_ccd);
  add_word!(state, "uncd", cog_uncd);
  add_word!(state, "uncdf", cog_uncdf);
  add_word!(state, "qstack", cog_qstack);
  add_word!(state, "root", cog_root);
  add_word!(state, "su", cog_su);
  add_word!(state, "chroot", cog_chroot);
}
