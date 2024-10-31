use crate::*;
use std::io::*;
use std::fs::File;
use std::fs::OpenOptions;

pub struct FileCustom  { file: File }
pub struct ReadCustom  { reader: Box<dyn Read> }
pub struct WriteCustom { writer: Box<dyn Write> }

impl Custom for FileCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(file)");
  }
  fn copyfunc(&self) -> Box<dyn CustomAny> {
    match self.file.try_clone() {
      Ok(file) => Box::new(FileCustom{ file }),
      Err(_)   => Box::new(Void{}),
    }
  }
}
impl Custom for ReadCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(reader)");
  }
  fn copyfunc(&self) -> Box<dyn CustomAny> {
    Box::new(Void{})
  }
}
impl Custom for WriteCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(writer)");
  }
  fn copyfunc(&self) -> Box<dyn CustomAny> {
    Box::new(Void{})
  }
}

pub fn questionmark(state: &CognitionState, f: &mut dyn Write) {
  fwrite_check!(f, GRN);
  println!("STACK:");
  fwrite_check!(f, COLOR_RESET);
  let cur = state.current_ref();
  for v in cur.stack.iter() { v.fprint(f, "\n"); }
  fwrite_check!(f, b"\n");
}

pub fn cog_questionmark(state: CognitionState, _: Option<&Value>) -> CognitionState {
  questionmark(&state, &mut stdout());
  state
}

pub fn cog_period(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  v.print("\n");
  state.pool.add_val(v);
  state
}

pub fn cog_print(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().iter().any(|x| !x.is_word()) {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  for wv in v.value_stack_ref().iter() {
    if let Err(e) = stdout().write_all(wv.vword_ref().str_word.as_bytes()) {
      let _ = stderr().write(format!("{e}").as_bytes()); }}
  state.pool.add_val(v);
  state
}

pub fn cog_read(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut vword = state.pool.get_vword(DEFAULT_STRING_LENGTH);
  if stdin().read_line(&mut vword.str_word).is_err() {
    state.pool.add_vword(vword);
    return state.eval_error("READ FAILED", w);
  }
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_stdout(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let boxed_custom = Box::new(WriteCustom{ writer: Box::new(stdout()) });
  let vcustom = state.pool.get_vcustom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_stdin(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let boxed_custom = Box::new(ReadCustom{ reader: Box::new(stdin()) });
  let vcustom = state.pool.get_vcustom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_stderr(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let boxed_custom = Box::new(WriteCustom{ writer: Box::new(stderr()) });
  let vcustom = state.pool.get_vcustom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_fopen(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_word!(state, w);
  let Ok(file) = File::open(&v.value_stack_ref().first().unwrap().vword_ref().str_word) else {
    state.current().stack.push(v);
    return state.eval_error("INVALID FILENAME", w);
  };
  let boxed_custom = Box::new(ReadCustom{ reader: Box::new(file) });
  let vcustom = state.pool.get_vcustom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_file(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_word!(state, w);
  let string = &v.value_stack_ref().first().unwrap().vword_ref().str_word;
  let boxed_custom = if let Ok(file) =
    OpenOptions::new().write(true).read(true).create(true).open(string) {
    Box::new(FileCustom{ file })
  } else {
    state.current().stack.push(v);
    return state.eval_error("INVALID FILENAME", w);
  };
  let vcustom = state.pool.get_vcustom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_file_new(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_word!(state, w);
  let string = &v.value_stack_ref().first().unwrap().vword_ref().str_word;
  let boxed_custom = if let Ok(file) =
    OpenOptions::new().write(true).read(true).create_new(true).open(string) {
    Box::new(FileCustom{ file })
  } else {
    state.current().stack.push(v);
    return state.eval_error("INVALID FILENAME", w);
  };
  let vcustom = state.pool.get_vcustom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_file_append(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_word!(state, w);
  let string = &v.value_stack_ref().first().unwrap().vword_ref().str_word;
  let boxed_custom = if let Ok(file) =
    OpenOptions::new().append(true).read(true).create(true).open(string) {
    Box::new(FileCustom{ file })
  } else {
    state.current().stack.push(v);
    return state.eval_error("INVALID FILENAME", w);
  };
  let vcustom = state.pool.get_vcustom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_fquestionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let val = v.value_stack().first_mut().unwrap();
  match val {
    Value::Custom(vcustom) => {
      let Some(custom) = &mut vcustom.custom else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      };
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        questionmark(&state, &mut file.file);
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        questionmark(&state, &mut *writer.writer);
      } else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v);
    },
    Value::Word(vword) => {
      if let Ok(mut file) = File::create_new(&vword.str_word) {
        questionmark(&state, &mut file);
      } else if let Ok(mut file) = File::open(&vword.str_word) {
        questionmark(&state, &mut file);
      } else {
        stack.push(v);
        return state.eval_error("INVALID FILENAME", w)
      };
      state.pool.add_val(v);
    },
    _ => return {
      stack.push(v);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

pub fn cog_fperiod(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let mut v = stack.pop().unwrap();
  if v.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let val = v.value_stack().first_mut().unwrap();
  match val {
    Value::Custom(vcustom) => {
      let Some(custom) = &mut vcustom.custom else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      };
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        let print_v = stack.pop().unwrap();
        print_v.fprint(&mut file.file, "\n");
        state.pool.add_val(print_v);
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        let print_v = stack.pop().unwrap();
        print_v.fprint(&mut *writer.writer, "\n");
        state.pool.add_val(print_v);
      } else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v);
    },
    Value::Word(vword) => {
      if let Ok(mut file) = File::create_new(&vword.str_word) {
        let print_v = stack.pop().unwrap();
        print_v.fprint(&mut file, "\n");
        state.pool.add_val(print_v);
      } else if let Ok(mut file) = File::open(&vword.str_word) {
        let print_v = stack.pop().unwrap();
        print_v.fprint(&mut file, "\n");
        state.pool.add_val(print_v);
      } else {
        stack.push(v);
        return state.eval_error("INVALID FILENAME", w)
      };
      state.pool.add_val(v);
    },
    _ => return {
      stack.push(v);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

pub fn cog_fwrite(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let mut v = stack.pop().unwrap();
  let v2 = stack.last().unwrap();
  if v.value_stack_ref().len() != 1 || v2.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  if !v2.value_stack_ref().first().unwrap().is_word() {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let val = v.value_stack().first_mut().unwrap();
  match val {
    Value::Custom(vcustom) => {
      let Some(custom) = &mut vcustom.custom else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      };
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(file.file, &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        state.pool.add_val(print_v);
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(writer.writer, &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        state.pool.add_val(print_v);
      } else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v);
    },
    Value::Word(vword) => {
      if let Ok(mut file) = File::create(&vword.str_word) {
        let print_v = stack.pop().unwrap();
        fwrite_check!(file, &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        state.pool.add_val(print_v);
      } else {
        stack.push(v);
        return state.eval_error("INVALID FILENAME", w)
      };
      state.pool.add_val(v);
    },
    _ => return {
      stack.push(v);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

pub fn cog_fprint(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let mut v = stack.pop().unwrap();
  let v2 = stack.last().unwrap();
  if v.value_stack_ref().len() != 1 || v2.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  if !v2.value_stack_ref().first().unwrap().is_word() {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let val = v.value_stack().first_mut().unwrap();
  match val {
    Value::Custom(vcustom) => {
      let Some(custom) = &mut vcustom.custom else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      };
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(file.file, &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        state.pool.add_val(print_v);
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(writer.writer, &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        state.pool.add_val(print_v);
      } else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v);
    },
    Value::Word(vword) => {
      if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(&vword.str_word) {
        let print_v = stack.pop().unwrap();
        fwrite_check!(file, &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        state.pool.add_val(print_v);
      } else {
        stack.push(v);
        return state.eval_error("INVALID FILENAME", w)
      };
      state.pool.add_val(v);
    },
    _ => return {
      stack.push(v);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

pub fn cog_fread(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let val = v.value_stack().first_mut().unwrap();
  match val {
    Value::Custom(vcustom) => {
      let Some(custom) = &mut vcustom.custom else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      };
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        let mut vword = if let Ok(metadata) = file.file.metadata() {
          state.pool.get_vword(metadata.len() as usize)
        } else {
          stack.push(v);
          return state.eval_error("NO FILE METADATA", w)
        };
        if let Err(e) = file.file.read_to_string(&mut vword.str_word) {
          let _ = stderr().write(format!("{e}").as_bytes());
        }
        state.push_quoted(Value::Word(vword));
      } else if let Some(reader) = custom.as_any_mut().downcast_mut::<ReadCustom>() {
        let mut vword = state.pool.get_vword(DEFAULT_STRING_LENGTH);
        if let Err(e) = reader.reader.read_to_string(&mut vword.str_word) {
          let _ = stderr().write(format!("{e}").as_bytes());
        }
        state.push_quoted(Value::Word(vword));
      } else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v);
    },
    Value::Word(vword) => {
      if let Ok(mut file) = File::open(&vword.str_word) {
        let mut vword = if let Ok(metadata) = file.metadata() {
          state.pool.get_vword(metadata.len() as usize)
        } else {
          stack.push(v);
          return state.eval_error("NO FILE METADATA", w)
        };
        if let Err(e) = file.read_to_string(&mut vword.str_word) {
          let _ = stderr().write(format!("{e}").as_bytes());
        }
        state.push_quoted(Value::Word(vword));
      } else {
        stack.push(v);
        return state.eval_error("INVALID FILENAME", w)
      };
      state.pool.add_val(v);
    },
    _ => return {
      stack.push(v);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "?", cog_questionmark);
  add_word!(state, ".", cog_period);
  add_word!(state, "print", cog_print);
  add_word!(state, "read", cog_read);
  add_word!(state, "stdout", cog_stdout);
  add_word!(state, "stdin", cog_stdin);
  add_word!(state, "stderr", cog_stderr);
  add_word!(state, "stdout", cog_stdout);
  add_word!(state, "fopen", cog_fopen);
  add_word!(state, "file", cog_file);
  add_word!(state, "file-new", cog_file_new);
  add_word!(state, "file-append", cog_file_append);
  add_word!(state, "f?", cog_fquestionmark);
  add_word!(state, "f.", cog_fperiod);
  add_word!(state, "fwrite", cog_fwrite);
  add_word!(state, "fprint", cog_fprint);
  add_word!(state, "fread", cog_fread);
}
