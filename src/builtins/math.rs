use crate::*;

pub fn cog_base(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let cur = state.current();
  let Some(v) = cur.stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  match &mut cur.math {
    Some(math) => {
      if math.base() == 0 {
        let i = v.value_stack_ref().len();
        if i > i32::MAX as usize {
          cur.stack.push(v);
          return state.eval_error("OUT OF BOUNDS", w)
        }
        if let Some(e) = math.set_base(i as i32) {
          cur.stack.push(v);
          return state.eval_error(e, w)
        }
      } else {
        if v.value_stack_ref().len() != 1 {
          cur.stack.push(v);
          return state.eval_error("BAD ARGUMENT TYPE", w)
        }
        let base_v = &v.value_stack_ref()[0];
        if !base_v.is_word() {
          cur.stack.push(v);
          return state.eval_error("BAD ARGUMENT TYPE", w)
        }
        match math.stoi(&base_v.vword_ref().str_word) {
          Ok(i) => {
            if i > i32::MAX as isize || i < -i32::MAX as isize {
              cur.stack.push(v);
              return state.eval_error("OUT OF BOUNDS", w)
            }
            if let Some(e) = math.set_base(i as i32) {
              cur.stack.push(v);
              return state.eval_error(e, w)
            }
          },
          Err(e) => {
            cur.stack.push(v);
            return state.eval_error(e, w)
          },
        }
      }
    },
    None => {
      cur.stack.push(v);
      return state.eval_error("MATH DIGITS UNINITIALIZED", w)
    },
  }
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "base", cog_base);
}
