pub mod custom;

pub use crate::custom::*;
use cognition::*;
use cognition::builtins::io::FileCustom;
use std::thread;
use std::time::Duration;
use std::process::{Command, Stdio};
use std::ffi::OsString;
use time::DurationCustom;

pub fn cog_sleep(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) };
  let duration = match v.value_stack_ref().first().unwrap() {
    Value::Custom(vcustom) => match unsafe { vcustom.custom.as_custom_ref::<DurationCustom>() } {
      Some(d) => if d.neg { Duration::ZERO } else { d.duration.clone() },
      None => return state.eval_error("BAD ARGUMENT TYPE", w)
    },
    _ => {
      let i = get_unsigned!(state, w, isize, ACTIVE) as usize;
      if i > u64::MAX as usize { return state.eval_error("OUT OF BOUNDS", w) }
      Duration::from_secs(i as u64)
    }
  };
  let v = state.current().stack.pop().unwrap();
  state.pool.add_val(v);
  thread::sleep(duration);
  state
}

struct ChildStdio {
  stdin: Stdio,
  stdout: Stdio,
  stderr: Stdio
}
impl ChildStdio {
  fn default() -> Self {
    Self{
      stdin: Stdio::inherit(),
      stdout: Stdio::inherit(),
      stderr: Stdio::inherit()
    }
  }
}

fn stdio_from_value(v: &Value) -> Option<Stdio> {
  match v {
    Value::Word(vw) => {
      match vw.str_word.as_str() {
        "inherit" => Some(Stdio::inherit()),
        "piped" => Some(Stdio::piped()),
        "null" => Some(Stdio::null()),
        _ => panic!("Bad stdio value in cognition process::spawn() argument")
      }
    },
    Value::Custom(vc) => {
      const ERR: &str = "null FileCustom on stack";
      unsafe{ vc.custom.as_custom_ref::<FileCustom>() }.expect(ERR).file.as_ref().expect(ERR)
        .try_clone().map_or(None, |f| Some(f.into()))
    },
    _ => panic!("Bad stdio value in cognition process::spawn() argument")
  }
}

fn push_all(state: &mut CognitionState, vio: Value, venv: Value, vcur: Value, varg: Value, vcmd: Value) {
  state.current().stack.push(vcmd);
  state.current().stack.push(varg);
  state.current().stack.push(vcur);
  state.current().stack.push(venv);
  state.current().stack.push(vio);
}

fn bad_argument(mut state: CognitionState, w: Option<&Value>, vio: Value, venv: Value, vcur: Value, varg: Value, vcmd: Value) -> CognitionState {
  push_all(&mut state, vio, venv, vcur, varg, vcmd);
  state.eval_error("BAD ARGUMENT TYPE", w)
}

fn bad_file_io(mut state: CognitionState, w: Option<&Value>, vio: Value, venv: Value, vcur: Value, varg: Value, vcmd: Value) -> CognitionState {
  push_all(&mut state, vio, venv, vcur, varg, vcmd);
  state.eval_error("BAD FILE DESCRIPTOR", w)
}

fn get_spawn_args(mut state: CognitionState, w: Option<&Value>)
                      -> Result<(CognitionState, Value, Value, Value, Value, Value, ChildStdio), CognitionState> {
  if state.current_ref().stack.len() < 5 { return Err(state.eval_error("TOO FEW ARGUMENTS", w)) }
  let stack = &mut state.current().stack;
  let vio = stack.pop().unwrap();
  let venv = stack.pop().unwrap();
  let vcur = stack.pop().unwrap();
  let varg = stack.pop().unwrap();
  let vcmd = stack.pop().unwrap();
  let bad_argument = |state, w, vio, venv, vcur, varg, vcmd|
  Err(bad_argument(state, w, vio, venv, vcur, varg, vcmd));
  if vcmd.value_stack_ref().len() != 1 || vcur.value_stack_ref().len() != 1 {
    return bad_argument(state, w, vio, venv, vcur, varg, vcmd)
  }
  let vcmd_w = vcmd.value_stack_ref().first().unwrap();
  let vcur_w = vcur.value_stack_ref().first().unwrap();
  if !vcmd_w.is_word() || !vcur_w.is_word() {
    return bad_argument(state, w, vio, venv, vcur, varg, vcmd)
  }
  let venv_pred = |v: &Value| {
    if !v.is_macro() && !v.is_stack() { return false }
    if v.value_stack_ref().len() != 2 { return false }
    let v1 = v.value_stack_ref().first().unwrap();
    let v2 = v.value_stack_ref().last().unwrap();
    v1.is_word() && v2.is_word()
  };
  if !venv.value_stack_ref().iter().all(venv_pred) {
    return bad_argument(state, w, vio, venv, vcur, varg, vcmd)
  }
  if !varg.value_stack_ref().iter().all(|v| v.is_word()) {
    return bad_argument(state, w, vio, venv, vcur, varg, vcmd)
  }
  let vio_pred = |v: &Value| {
    if let Value::Word(vw) = v {
      let s = vw.str_word.as_str();
      s == "inherit" || s == "piped" || s == "null"
    } else if let Value::Custom(ref vc) = v {
      vc.custom.is_custom::<FileCustom>()
    } else {
      false
    }
  };
  let stdio = 'stdio: {
    if vio.value_stack_ref().len() == 0 {
      break 'stdio ChildStdio::default()
    } else if vio.value_stack_ref().len() == 3 {
      if vio.value_stack_ref().iter().all(vio_pred) {
        let Some(stdin) = stdio_from_value(&vio.value_stack_ref()[0]) else {
          return Err(bad_file_io(state, w, vio, venv, vcur, varg, vcmd))
        };
        let Some(stdout) = stdio_from_value(&vio.value_stack_ref()[1]) else {
          return Err(bad_file_io(state, w, vio, venv, vcur, varg, vcmd))
        };
        let Some(stderr) = stdio_from_value(&vio.value_stack_ref()[2]) else {
          return Err(bad_file_io(state, w, vio, venv, vcur, varg, vcmd))
        };
        break 'stdio ChildStdio { stdin, stdout, stderr }
      }
    }
    return bad_argument(state, w, vio, venv, vcur, varg, vcmd)
  };
  Ok((state, vcmd, varg, vcur, venv, vio, stdio))
}

// "command" [arg1,arg2,...] "current-dir" [[env,val],[env,val],...] [<stdin,stdout,stderr>] spawn -- [ (child) ]
pub fn cog_spawn(state: CognitionState, w: Option<&Value>) -> CognitionState {
  let (mut state, vcmd, varg, vcur, venv, vio, stdio) = match get_spawn_args(state, w) {
    Ok(results) => results, Err(state) => return state
  };
  let mut binding = Command::new(&vcmd.value_stack_ref().first().unwrap().vword_ref().str_word);
  let command = binding
    .current_dir(std::path::Path::new(&vcur.value_stack_ref().first().unwrap().vword_ref().str_word))
    .stdin(stdio.stdin).stdout(stdio.stdout).stderr(stdio.stderr);
  for v in varg.value_stack_ref().iter() {
    command.arg(&v.vword_ref().str_word);
  }
  for vpair in venv.value_stack_ref().iter() {
    let (vvar, vval) = (vpair.value_stack_ref().first().unwrap(), vpair.value_stack_ref().last().unwrap());
    command.env(OsString::from(&vvar.vword_ref().str_word), OsString::from(&vval.vword_ref().str_word));
  }
  let Ok(child) = command.spawn() else {
    push_all(&mut state, vio, venv, vcur, varg, vcmd);
    return state.eval_error("FAILED TO SPAWN PROCESS", w)
  };
  state.pool.add_val(vcmd);
  state.pool.add_val(varg);
  state.pool.add_val(vcur);
  state.pool.add_val(venv);
  let vcustom = get_child_custom(&mut state.pool, child);
  state.push_quoted(Value::Custom(vcustom));
  state
}

// [ (child) ] wait -- [ (child) ] [ <exit-code> ]
pub fn cog_wait(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut v = get_custom!(state, w);
  let custom = v.value_stack().first_mut().unwrap().vcustom_mut().custom.as_any_mut();
  let Some(child_custom) = custom.downcast_mut::<ChildCustom>() else {
    state.current().stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let Ok(exit_status) = child_custom.child.wait() else {
    state.current().stack.push(v);
    return state.eval_error("CHILD NOT RUNNING", w)
  };
  state.current().stack.push(v);
  if let Some(code) = exit_status.code() {
    let Some(math) = state.get_math() else { return state.eval_error("MATH BASE ZERO", w) };
    match math.math().itos(code as isize, &mut state) {
      Ok(s) => {
        let mut v = state.pool.get_vword(s.len());
        v.str_word.push_str(&s);
        state.pool.add_string(s);
        state.set_math(math);
        state.push_quoted(Value::Word(v))
      },
      Err(e) => return state.with_math(math).eval_error(e, w)
    }
  } else {
    let vstack = state.pool.get_vstack(0);
    state.current().stack.push(Value::Stack(vstack));
  }
  state
}

pub fn cog_try_wait(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

pub fn cog_clear_pools(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  clear_pools(&mut state.pool);
  state
}

#[no_mangle]
pub extern fn add_words(state: &mut CognitionState, lib: &Library) {
  ensure_foreign_library!(state, lib);
  add_word!(state, lib, "sleep", cog_sleep);
  add_word!(state, lib, "spawn", cog_spawn);
  add_word!(state, lib, "wait", cog_wait);
  add_word!(state, lib, "clear-pools", cog_sleep);
}
