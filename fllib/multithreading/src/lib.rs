use cognition::*;
use std::thread;
use std::io::Write;
use std::mem;

struct CogStateWrapper { cogstate: CognitionState }
unsafe impl Send for CogStateWrapper {}

struct ThreadHandler { handle: Option<thread::JoinHandle<CogStateWrapper>> }
impl Custom for ThreadHandler {
  fn printfunc(&self, f: &mut dyn Write) {
    if self.handle.is_none() {
      fwrite_check_pretty!(f, b"(void thread)");
    } else {
      fwrite_check_pretty!(f, b"(thread)");
    }
  }
  fn copyfunc(&self) -> Box<dyn CustomAny> {
    Box::new(ThreadHandler{ handle: None })
  }
}

// [ ] spawn -> [ (thread) ]
// Takes a stack and turns it into a new cognition instance running in another thread
// Returns a custom thread handler type
pub fn cog_spawn(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  println!("called spawn");
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if !v.is_stack() {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  for val in v.vstack_mut().container.stack.iter_mut() {
    if !(val.is_stack() || val.is_macro()) {
      let new_val = state.pool.get_vstack(1);
      let old_val = mem::replace(val, Value::Stack(new_val));
      val.vstack_mut().container.stack.push(old_val);
    }
  }
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
  let vhandler = state.pool.get_vcustom(Box::new(ThreadHandler{ handle: Some(handle) }));
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
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  let custom = v_stack.first_mut().unwrap().vcustom_mut().custom.as_mut().unwrap();
  if let Some(handler) = custom.as_any_mut().downcast_mut::<ThreadHandler>() {
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
        let mut true_v = state.pool.get_vword(1);
        true_v.str_word.push('t');
        state.push_quoted(Value::Word(true_v));
        while let Some(arg) = cogstate.args.pop() {
          state.pool.add_val(arg)
        }
        while let Some(lib) = cogstate.fllibs.pop() {
          state.fllibs.push(lib)
        }
      } else {
        let empty = state.pool.get_vstack(0);
        state.current().stack.push(Value::Stack(empty));
        let empty = state.pool.get_vstack(0);
        state.current().stack.push(Value::Stack(empty));
        let mut result = state.pool.get_vstack(1);
        result.container.stack.push(Value::Word(state.pool.get_vword(0)));
        state.current().stack.push(Value::Stack(result));
      }
      state.pool.add_val(v);
      return state;
    }
  }
  stack.push(v);
  state.eval_error("BAD ARGUMENT TYPE", w)
}

#[no_mangle]
pub extern fn add_words(state: &mut CognitionState) {
  add_word!(state, "spawn", cog_spawn);
  add_word!(state, "thread", cog_thread);
}