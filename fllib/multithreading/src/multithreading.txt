use crate::*;
use std::thread;
//use std::time::Duration;

//static EXAMPLE_TH: ThreadHandler;

struct ThreadHandler { handle: thread::JoinHandle<CognitionState> }
impl Custom for ThreadHandler {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check_pretty!(f, b"(thread)");
  }
  fn copyfunc(&self) -> Box<dyn Custom + Send + Sync> {
    Box::new(Void{})
  }
}

unsafe impl Send for VCustom {}
unsafe impl Sync for VCustom {}

// [  ] spawn -> [ (thread) ]
// Takes a stack and turns it into a new cognition instance running in another thread
// Returns a custom thread handler type
pub fn cog_spawn(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if !v.is_stack() {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let new_v = state.value_copy(&v);
  state.pool.add_val(v);
  let mut metastack = state.pool.get_stack(DEFAULT_STACK_SIZE);
  metastack.push(new_v);
  let mut cogstate = CognitionState::new(metastack);
  state.args.reverse();
  while let Some(arg) = state.args.pop() {
    cogstate.args.push(arg);
  }
  for arg in cogstate.args.iter() {
    let new_arg = state.string_copy(arg);
    state.args.push(new_arg);
  }
  let handle = thread::spawn(move || {
    cogstate.crank()
  });
  let thread_v = Value::Custom(state.pool.get_vcustom(Box::new(ThreadHandler{ handle })));
  state.push_quoted(thread_v);
  state
}

// [ (thread) ] thread -> [  ]
// Takes a custom thread handler object and waits for the thread to finish (exit).
// Returns the current stack on the thread's metastack, or errors if thread panicked
pub fn cog_thread(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v_stack = v.value_stack_ref();
  if v_stack.len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  let _thread_v = v_stack.first().unwrap();
  // if thread_v.vcustom_ref().custom.is::<ThreadHandler>() {

  //}

  state
}


// [ (thread) ] [  ] send -> [ (thread) ]
// Pops the stack and sends it to the other thread
pub fn cog_send(state: CognitionState, _w: Option<&Value>) -> CognitionState { state }


// [ (thread) ] recv -> [ (thread) ] [  ]
// Waits until the thread sends a message, and then pushes it to the stack
pub fn cog_recv(state: CognitionState, _w: Option<&Value>) -> CognitionState { state }


// [ (thread) ] try-recv -> [ (thread) ] [  ]
// Checks if the thread has sent a message, and if so returns it on the stack
// If not, pushes "NO MESSAGE RECEIVED" to the error stack
pub fn cog_try_recv(state: CognitionState, _w: Option<&Value>) -> CognitionState { state }

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "spawn", cog_spawn);
}
