use crate::*;

pub fn cog_compose(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v2) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(v1) = stack.last_mut() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  v2.value_stack().reverse(); // TODO: avoid this reversion
  while let Some(v) = v2.value_stack().pop() {
    v1.value_stack().push(v);
  }
  state.pool.add_val(v2);
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

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "eval", EVAL);
  add_word!(state, "return", RETURN);
  add_word!(state, "compose", cog_compose);
  add_word!(state, "cast", cog_cast);
}
