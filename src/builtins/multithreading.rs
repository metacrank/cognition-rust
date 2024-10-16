use crate::*;
//use std::thread;
//use std::time::Duration;

// [  ] spawn -> [ (thread) ]
// Takes a stack and turns it into a new cognition instance running in another thread
// Returns a custom thread type
pub fn cog_spawn(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let cur = state.current();
  let Some(v) = cur.stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Value::Stack(ref vstack) = v else {
    cur.stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  if vstack.container.dependent {
    let _new_vstack = state.pool.get_vstack(vstack.container.stack.len());
  }

  let mut metastack = state.pool.get_stack(DEFAULT_STACK_SIZE);
  metastack.push(v);
  let mut cogstate = CognitionState::new(metastack);
  state.args.reverse();
  while let Some(arg) = state.args.pop() {
    cogstate.args.push(arg);
  }
  for arg in cogstate.args.iter() {
    let new_arg = state.string_copy(arg);
    state.args.push(new_arg);
  }
  // let handle = thread::spawn(move || {
  //   cogstate.crank();
  // });
  state
}


// [ (thread) ] thread -> [  ]
// Takes a custom thread object and waits for the thread to finish (exit).
// Returns the current stack on the thread's metastack
pub fn cog_thread(state: CognitionState, _w: Option<&Value>) -> CognitionState { state }


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
