pub mod custom;

use crate::custom::*;
use cognition::{*, builtins::stackops::cog_swap};
use std::thread;
use std::sync::{mpsc, mpsc::TryRecvError, Arc};

fn new_cogstate(state: &mut CognitionState, v: Value) -> CognitionState {
  let mut stack = state.pool.get_stack(DEFAULT_STACK_SIZE);
  stack.push(v);
  let mut family = state.pool.get_family();
  for member in state.family.iter() { family.push(member.clone()) }
  let mut args = state.pool.get_stack(state.args.len());
  state.args.reverse();
  while let Some(arg) = state.args.pop() { args.push(arg) }
  for arg in args.iter() {
    let new_arg = state.value_copy(arg);
    state.args.push(new_arg);
  }
  let fllibs = match state.fllibs.take() {
    Some(state_libs) => {
      let mut libs = ForeignLibraries::with_capacity(state_libs.len());
      for (k, v) in state_libs.iter() {
        libs.insert(state.string_copy(k), state.foreign_library_copy(v));
      }
      state.fllibs = Some(state_libs);
      Some(libs)
    },
    None => None
  };
  let mut builtins = state.pool.get_functions(BUILTINS_SIZE);
  for func in state.builtins.iter() { builtins.push(func.clone()) }

  CognitionState {
    chroots: Vec::new(),
    stack, family,
    parser: None,
    exited: false,
    exit_code: None,
    args, fllibs, builtins,
    serde: state.serde.clone(),
    pool: cognition::pool::Pool::new()
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
  state.ensure_quoted(&mut v.vstack_mut().container.stack);
  let wrapper = CogStateWrapper(new_cogstate(&mut state, v));
  let handle = thread::spawn(move || {
    let copy = wrapper;
    CogStateWrapper(copy.0.crank())
  });
  let vcustom = get_thread_custom(&mut state.pool, Some(handle));
  state.push_quoted(Value::Custom(vcustom));
  state
}

fn reclaim_memory(state: &mut CognitionState, mut cogstate: CognitionState) {
  for chroot in cogstate.chroots.into_iter() {
    state.pool.add_stack(chroot);
  }
  state.pool.add_stack(cogstate.stack);
  state.pool.add_family(cogstate.family);
  for arg in cogstate.args.into_iter() {
    state.pool.add_val(arg);
  }
  if let Some(fllibs) = &mut cogstate.fllibs {
    if state.fllibs.is_none() {
      state.fllibs = Some(ForeignLibraries::with_capacity(fllibs.len()));
    }
    let state_fllibs = state.fllibs.as_mut().unwrap();
    for (key, fllib) in fllibs.drain() {
      if state_fllibs.get(&key).is_none() {
        state_fllibs.insert(key, fllib);
      } else {
        state.pool.add_string(key);
        if let Some(lib) = Arc::into_inner(fllib.library) {
          state.pool.add_string(lib.lib_name);
          state.pool.add_string(lib.lib_path);
        }
        state.pool.add_functions(fllib.functions);
      }
    }
  }
  state.pool.add_functions(cogstate.builtins);
  state.pool.absorb(&mut cogstate.pool);
}

fn join_states(state: &mut CognitionState, mut cogstate: CognitionState) {
  let stack_v = cogstate.stack.pop().expect("Cognition metastack was empty");
  state.current().stack.push(stack_v);
  let mut exit_code_stack = state.pool.get_vstack(1);
  if let Some(code) = cogstate.exit_code.take() {
    let mut code_v = state.pool.get_vword(code.len());
    code_v.str_word.push_str(&code);
    state.pool.add_string(code);
    exit_code_stack.container.stack.push(Value::Word(code_v));
  }
  state.current().stack.push(Value::Stack(exit_code_stack));
  let false_v = state.pool.get_vword(0);
  state.push_quoted(Value::Word(false_v));

  reclaim_memory(state, cogstate)
}

// [ (thread) ] thread -> [ ] [ exit_code? ] [ t/f panic'd? ]
pub fn cog_thread(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut v = get_custom!(state, w);
  let custom = &mut v.value_stack().first_mut().unwrap().vcustom_mut().custom;
  if 'return_ok: {
    if let Some(handler) = custom.as_any_mut().downcast_mut::<ThreadCustom>() {
      let mut handle_lock = handler.handle.as_ref().expect("Uninitialized ThreadCustom").lock();
      match &mut handle_lock {
        Ok(handle) => {
          if let Some(handle) = handle.take() {
            drop(handle_lock);
            if let Ok(cogstatewrapper) = handle.join() {
              join_states(&mut state, cogstatewrapper.0);
            } else {
              let empty = state.pool.get_vstack(0);
              state.current().stack.push(Value::Stack(empty));
              let empty = state.pool.get_vstack(0);
              state.current().stack.push(Value::Stack(empty));
              let mut true_v = state.pool.get_vword(1);
              true_v.str_word.push('t');
              state.push_quoted(Value::Word(true_v));
            }
            break 'return_ok true
          } else {
            state = state.eval_error("NULL THREAD", w);
            break 'return_ok false
          }
        },
        Err(_) => {
          state = state.eval_error("POISONED THREAD", w);
          break 'return_ok false
        }
      }
    }
    state = state.eval_error("BAD ARGUMENT TYPE", w);
    false
  } {
    state.pool.add_val(v);
  } else {
    state.current().stack.push(v);
  }
  state
}

pub fn cog_channel(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let (tx, rx) = mpsc::channel();
  let send_vcustom = get_send_custom(&mut state.pool, Some(tx));
  let recv_vcustom = get_recv_custom(&mut state.pool, Some(rx));
  state.push_quoted(Value::Custom(send_vcustom));
  state.push_quoted(Value::Custom(recv_vcustom));
  state
}

pub fn cog_send(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let vtx = get_custom!(state, w);
  let v = state.current().stack.pop().unwrap();
  if v.value_stack_ref().len() != 1 {
    state.current().stack.push(v);
    state.current().stack.push(vtx);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  state.current().stack.push(v);
  let vcustom = vtx.value_stack_ref().first().unwrap().vcustom_ref();
  let Some(send_custom) = vcustom.custom.as_any().downcast_ref::<SendCustom>() else {
    state.current().stack.push(vtx);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  };
  let sender = send_custom.tx.as_ref().expect("uninitialized SendCustom on stack");
  let vdata = state.current().stack.pop().unwrap();
  if let Err(e) = sender.send(ValueWrapper(vdata)) {
    state.current().stack.push(e.0.0);
    state.current().stack.push(vtx);
    return state.eval_error("DISCONNECTED CHANNEL", w)
  }
  state.pool.add_val(vtx);
  state
}

pub fn cog_recv(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let vrx = get_custom!(state, w);
  let vcustom = vrx.value_stack_ref().first().unwrap().vcustom_ref();
  let Some(recv_custom) = vcustom.custom.as_any().downcast_ref::<RecvCustom>() else {
    state.current().stack.push(vrx);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  };
  let receiver = recv_custom.rx.as_ref().expect("uninitialized RecvCustom on stack");
  match receiver.recv() {
    Ok(value) => state.current().stack.push(value.0),
    Err(_) => state.eval_error_mut("DISCONNECTED CHANNEL", w)
  }
  state.current().stack.push(vrx);
  state
}

pub fn cog_try_recv(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let vrx = get_custom!(state, w);
  let vcustom = vrx.value_stack_ref().first().unwrap().vcustom_ref();
  let Some(recv_custom) = vcustom.custom.as_any().downcast_ref::<RecvCustom>() else {
    state.current().stack.push(vrx);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  };
  let receiver = recv_custom.rx.as_ref().expect("uninitialized RecvCustom on stack");
  match receiver.try_recv() {
    Ok(value) => state.current().stack.push(value.0),
    Err(TryRecvError::Empty) => state = builtins::combinators::cog_stack(state, w),
    Err(TryRecvError::Disconnected) => state.eval_error_mut("DISCONNECTED CHANNEL", w)
  }
  state.current().stack.push(vrx);
  state
}

pub fn cog_share(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let shared_vcustom = get_shared_custom(&mut state.pool, Some(v));
  state.push_quoted(Value::Custom(shared_vcustom));
  state
}

pub fn cog_steal(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_custom!(state, w);
  let vcustom = v.value_stack_ref().first().unwrap().vcustom_ref();
  let Some(shared_custom) = vcustom.custom.as_any().downcast_ref::<SharedCustom>() else {
    state.current().stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  };
  let mut lock = shared_custom.value.as_ref().expect("uninitialized SharedCustom on stack").lock();
  match &mut lock {
    Ok(shared_value) => match shared_value.take() {
      Some(val) => {
        state.current().stack.push(val);
        drop(lock);
        state.pool.add_val(v);
        state
      },
      None => {
        drop(lock);
        state.current().stack.push(v);
        state.eval_error("NULL SHARED", w)
      }
    },
    Err(_) => {
      drop(lock);
      state.current().stack.push(v);
      state.eval_error("POISONED SHARED", w)
    }
  }
}

pub fn cog_give(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let vshare = get_custom!(state, w);
  let vcustom = vshare.value_stack_ref().first().unwrap().vcustom_ref();
  let Some(shared_custom) = vcustom.custom.as_any().downcast_ref::<SharedCustom>() else {
    state.current().stack.push(vshare);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  };
  let mut lock = shared_custom.value.as_ref().expect("uninitialized SharedCustom on stack").lock();
  match &mut lock {
    Ok(shared_value) => {
      let v = state.current().stack.pop().unwrap();
      if let Some(v) = shared_value.take() {
        state.pool.add_val(v);
      }
      **shared_value = Some(v);
      drop(lock);
      state.pool.add_val(vshare);
      state
    },
    Err(_) => {
      drop(lock);
      state.current().stack.push(vshare);
      state.eval_error("POISONED SHARED", w)
    }
  }
}

pub fn cog_stolen_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_custom!(state, w);
  let vcustom = v.value_stack_ref().first().unwrap().vcustom_ref();
  let Some(shared_custom) = vcustom.custom.as_any().downcast_ref::<SharedCustom>() else {
    state.current().stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  };
  let lock = shared_custom.value.as_ref().expect("uninitialized SharedCustom on stack").lock();
  let vword = if let Ok(true) = lock.map(|mutex_guard| mutex_guard.is_none()) {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('t');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.current().stack.push(v);
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_poisoned_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_custom!(state, w);
  let vcustom = v.value_stack_ref().first().unwrap().vcustom_ref();
  let Some(shared_custom) = vcustom.custom.as_any().downcast_ref::<SharedCustom>() else {
    state.current().stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  };
  let lock = shared_custom.value.as_ref().expect("uninitialized SharedCustom on stack").lock();
  let vword = if lock.is_err() {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('t');
    vword
  } else {
    state.pool.get_vword(0)
  };
  drop(lock);
  state.current().stack.push(v);
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_clear_poison(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_custom!(state, w);
  let vcustom = v.value_stack_ref().first().unwrap().vcustom_ref();
  let Some(shared_custom) = vcustom.custom.as_any().downcast_ref::<SharedCustom>() else {
    state.current().stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  };
  shared_custom.value.as_ref().expect("uninitialized SharedCustom on stack").clear_poison();
  state.current().stack.push(v);
  state
}

pub fn cog_eval_shared(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  state = cog_swap(state, None);
  let v = get_custom!(state, w);
  let vcustom = v.value_stack_ref().first().unwrap().vcustom_ref();
  let Some(shared_custom) = vcustom.custom.as_any().downcast_ref::<SharedCustom>() else {
    state.current().stack.push(v);
    return cog_swap(state, None).eval_error("BAD ARGUMENT TYPE", w);
  };
  let mut lock = shared_custom.value.as_ref().expect("uninitialized SharedCustom on stack").lock();
  match &mut lock {
    Ok(shared_value) => match shared_value.take() {
      Some(val) => {
        state.current().stack.push(val);
        // evalf never fails because stack is not empty
        state = cog_swap(state, None).evalf(None);
        **shared_value = state.current().stack.pop();
        drop(lock);
        state.pool.add_val(v);
        state
      },
      None => {
        drop(lock);
        state.current().stack.push(v);
        cog_swap(state, None).eval_error("NULL SHARED", w)
      }
    },
    Err(_) => {
      drop(lock);
      state.current().stack.push(v);
      cog_swap(state, None).eval_error("POISONED SHARED", w)
    }
  }
}

pub fn cog_clear_multithreading_pools(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  clear_pools(&mut state.pool);
  state
}

// when we can pull in the cognition 'time' library, define "recv-timeout"

#[no_mangle]
pub extern fn add_words(state: &mut CognitionState, lib: &Library) {
  ensure_foreign_library!(state, lib);
  register_custom!(state, lib, ThreadCustom);
  register_custom!(state, lib, SendCustom);
  register_custom!(state, lib, RecvCustom);
  register_custom!(state, lib, SharedCustom);
  add_word!(state, lib, "spawn", cog_spawn);
  add_word!(state, lib, "thread", cog_thread);
  add_word!(state, lib, "channel", cog_channel);
  add_word!(state, lib, "send", cog_send);
  add_word!(state, lib, "recv", cog_recv);
  add_word!(state, lib, "try-recv", cog_try_recv);
  add_word!(state, lib, "share", cog_share);
  add_word!(state, lib, "steal", cog_steal);
  add_word!(state, lib, "give", cog_give);
  add_word!(state, lib, "stolen?", cog_stolen_questionmark);
  add_word!(state, lib, "poisoned?", cog_poisoned_questionmark);
  add_word!(state, lib, "clear-poison", cog_clear_poison);
  add_word!(state, lib, "eval-shared", cog_eval_shared);
  add_word!(state, lib, "clear-multithreading-pools", cog_clear_multithreading_pools);
}
