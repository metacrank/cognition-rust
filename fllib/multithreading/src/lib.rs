use cognition::*;
use std::thread;
use std::io::Write;

struct CogStateWrapper { cogstate: CognitionState }
unsafe impl Send for CogStateWrapper {}

pub struct ThreadCustom { handle: Option<thread::JoinHandle<CogStateWrapper>> }
impl Custom for ThreadCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    if self.handle.is_none() {
      fwrite_check!(f, b"(void thread)");
    } else {
      fwrite_check!(f, b"(thread)");
    }
  }
  fn copyfunc(&self) -> Box<dyn CustomAny> {
    Box::new(ThreadCustom{ handle: None })
  }
}

// [ ] spawn -> [ (thread) ]
// Takes a stack and turns it into a new cognition instance running in another thread
// Returns a custom thread handler type
pub fn cog_spawn(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if !v.is_stack() {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  ensure_quoted!(state, v.vstack_mut().container.stack);
  let mut metastack = state.pool.get_stack(1);
  metastack.push(v);
  let mut cogstate = CognitionState::new(metastack);
  state.args.reverse();
  state.pool.add_stack(cogstate.args);
  cogstate.args = state.pool.get_stack(state.args.len());
  while let Some(arg) = state.args.pop() {
    cogstate.args.push(arg);
  }
  for arg in cogstate.args.iter() {
    let new_arg = state.value_copy(arg);
    state.args.push(new_arg);
  }
  let wrapper = CogStateWrapper{ cogstate };
  let handle = thread::spawn(move || {
    let copy = wrapper;
    CogStateWrapper{ cogstate: copy.cogstate.crank() }
  });
  let vhandler = VCustom::with_custom(Box::new(ThreadCustom{ handle: Some(handle) }));
  state.push_quoted(Value::Custom(vhandler));
  state
}

// [ (thread) ] thread -> [ ] [ exit_code? ] [ t/f panic'd? ]
// TODO: reclaim more of the cogstate into pool
pub fn cog_thread(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v_stack = v.value_stack();
  if v_stack.len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let Value::Custom(vcustom) = v_stack.first_mut().unwrap() else {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let custom = &mut vcustom.custom;
  if let Some(handler) = custom.as_any_mut().downcast_mut::<ThreadCustom>() {
    if let Some(handle) = handler.handle.take() {
      if let Ok(cogstatewrapper) = handle.join() {
        let mut cogstate = cogstatewrapper.cogstate;
        let stack_v = cogstate.stack.pop().expect("Cognition metastack was empty");
        stack.push(stack_v);
        let mut exit_code_stack = state.pool.get_vstack(1);
        if let Some(code) = cogstate.exit_code {
          let mut code_v = state.pool.get_vword(code.len());
          code_v.str_word.push_str(&code);
          state.pool.add_string(code);
          exit_code_stack.container.stack.push(Value::Word(code_v));
        }
        state.current().stack.push(Value::Stack(exit_code_stack));
        let false_v = state.pool.get_vword(0);
        state.push_quoted(Value::Word(false_v));
        while let Some(arg) = cogstate.args.pop() {
          state.pool.add_val(arg)
        }
      } else {
        let empty = state.pool.get_vstack(0);
        state.current().stack.push(Value::Stack(empty));
        let empty = state.pool.get_vstack(0);
        state.current().stack.push(Value::Stack(empty));
        let mut true_v = state.pool.get_vword(1);
        true_v.str_word.push('t');
        state.push_quoted(Value::Word(true_v));
      }
      state.pool.add_val(v);
      return state;
    }
  }
  stack.push(v);
  state.eval_error("BAD ARGUMENT TYPE", w)
}

#[no_mangle]
pub extern fn add_words(state: &mut CognitionState, lib: &Library, lib_name: &String) {
  add_word!(state, lib, lib_name, "spawn", cog_spawn);
  add_word!(state, lib, lib_name, "thread", cog_thread);
}
