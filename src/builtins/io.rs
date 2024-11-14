use crate::*;
use std::io::{self, Read, BufRead, Seek};
use std::fs::File;

macro_rules! trait_any {
  () => {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
  }
}
macro_rules! impl_any {
  () => {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
  }
}

pub trait WriteAny: Any + io::Write {
  trait_any!();
  fn as_write_mut(&mut self) -> &mut dyn Write;
}
impl<T: Any + io::Write> WriteAny for T {
  impl_any!();
  fn as_write_mut(&mut self) -> &mut dyn Write { self }
}

pub trait ReadAny: Any + io::Read { trait_any!(); }
impl<T: Any + io::Read> ReadAny for T { impl_any!(); }

pub trait ReadWriteAny: io::Read + io::Write + Any { trait_any!(); }
impl<T: Any + io::Read + io::Write> ReadWriteAny for T { impl_any!(); }

// Always unwrap Options
pub struct ReadWriteCustom { pub stream: Option<Box<dyn ReadWriteAny>> }
pub struct FileCustom  { pub file: Option<File> }
pub struct ReadCustom  { pub reader: Option<Box<dyn ReadAny>> }
pub struct WriteCustom { pub writer: Option<Box<dyn WriteAny>> }
pub struct BufReadCustom  { pub bufreader: Option<io::BufReader<Box<dyn ReadAny>>> }
pub struct BufWriteCustom { pub bufwriter: Option<io::BufWriter<Box<dyn WriteAny>>> }

#[cognition_macros::custom(serde_as_void)]
impl Custom for ReadWriteCustom {
  fn printfunc(&self, f: &mut dyn io::Write) {
    let read_write_any = ReadWriteAny::as_any(&**self.stream.as_ref().unwrap());
    if read_write_any.downcast_ref::<io::Empty>().is_some() {
      fwrite_check!(f, b"(empty)");
    } else {
      fwrite_check!(f, b"(stream)");
    }
  }
  fn copyfunc(&self, _: &mut CognitionState) -> Box<dyn Custom> {
    let read_write_any = ReadWriteAny::as_any(&**self.stream.as_ref().unwrap());
    if let Some(file) = read_write_any.downcast_ref::<File>() {
      if let Ok(f) = file.try_clone() {
        return Box::new(ReadWriteCustom{ stream: Some(Box::new(f)) })
      }
    } else if read_write_any.downcast_ref::<io::Empty>().is_some() {
      return Box::new(ReadWriteCustom{ stream: Some(Box::new(io::empty())) })
    }
    Box::new(Void{})
  }
}
#[cognition_macros::custom(serde_as_void)]
impl Custom for FileCustom {
  fn printfunc(&self, f: &mut dyn io::Write) {
    fwrite_check!(f, b"(file)");
  }
  fn copyfunc(&self, _: &mut CognitionState) -> Box<dyn Custom> {
    match self.file.as_ref().unwrap().try_clone() {
      Ok(f) => Box::new(FileCustom{ file: Some(f) }),
      Err(_)   => Box::new(Void{}),
    }
  }
}
#[cognition_macros::custom(serde_as_void)]
impl Custom for ReadCustom {
  fn printfunc(&self, f: &mut dyn io::Write) {
    let read_any = (**self.reader.as_ref().unwrap()).as_any();
    if read_any.downcast_ref::<io::Stdin>().is_some() {
      fwrite_check!(f, b"(stdin)");
    } else {
      fwrite_check!(f, b"(reader)");
    }
  }
  fn copyfunc(&self, _: &mut CognitionState) -> Box<dyn Custom> {
    let read_any = (**self.reader.as_ref().unwrap()).as_any();
    if let Some(file) = read_any.downcast_ref::<File>() {
      if let Ok(f) = file.try_clone() {
        return Box::new(ReadCustom{ reader: Some(Box::new(f)) })
      }
    } else if read_any.downcast_ref::<io::Stdin>().is_some() {
      return Box::new(ReadCustom{ reader: Some(Box::new(io::stdin())) })
    } else if read_any.downcast_ref::<io::Empty>().is_some() {
      return Box::new(ReadCustom{ reader: Some(Box::new(io::empty())) })
    }
    Box::new(Void{})
  }
}
#[cognition_macros::custom(serde_as_void)]
impl Custom for WriteCustom {
  fn printfunc(&self, f: &mut dyn io::Write) {
    let read_any = (**self.writer.as_ref().unwrap()).as_any();
    if read_any.downcast_ref::<io::Stdout>().is_some() {
      fwrite_check!(f, b"(stdout)");
    } else if read_any.downcast_ref::<io::Stderr>().is_some() {
      fwrite_check!(f, b"(stderr)");
    } else {
      fwrite_check!(f, b"(writer)");
    }
  }
  fn copyfunc(&self, _: &mut CognitionState) -> Box<dyn Custom> {
    let write_any = (**self.writer.as_ref().unwrap()).as_any();
    if let Some(file) = write_any.downcast_ref::<File>() {
      if let Ok(f) = file.try_clone() {
        return Box::new(WriteCustom{ writer: Some(Box::new(f)) })
      }
    } else if write_any.downcast_ref::<io::Stdout>().is_some() {
      return Box::new(WriteCustom{ writer: Some(Box::new(io::stdout())) })
    } else if write_any.downcast_ref::<io::Stderr>().is_some() {
      return Box::new(WriteCustom{ writer: Some(Box::new(io::stderr())) })
    } else if write_any.downcast_ref::<io::Empty>().is_some() {
      return Box::new(WriteCustom{ writer: Some(Box::new(io::empty())) })
    }
    Box::new(Void{})
  }
}
#[cognition_macros::custom(serde_as_void)]
impl Custom for BufReadCustom {
  fn printfunc(&self, f: &mut dyn io::Write) {
    fwrite_check!(f, b"(bufreader)");
  }
  fn copyfunc(&self, _: &mut CognitionState) -> Box<dyn Custom> {
    let read_any = (**self.bufreader.as_ref().unwrap().get_ref()).as_any();
    if let Some(file) = read_any.downcast_ref::<File>() {
      if let Ok(f) = file.try_clone() {
        return Box::new(BufReadCustom{ bufreader: Some(io::BufReader::new(Box::new(f))) })
      }
    } else if read_any.downcast_ref::<io::Stdin>().is_some() {
      return Box::new(BufReadCustom{ bufreader: Some(io::BufReader::new(Box::new(io::stdin()))) })
    } else if read_any.downcast_ref::<io::Empty>().is_some() {
      return Box::new(BufReadCustom{ bufreader: Some(io::BufReader::new(Box::new(io::empty()))) })
    }
    Box::new(Void{})
  }
}
#[cognition_macros::custom(serde_as_void)]
impl Custom for BufWriteCustom {
  fn printfunc(&self, f: &mut dyn io::Write) {
    fwrite_check!(f, b"(bufwriter)");
  }
  fn copyfunc(&self, _: &mut CognitionState) -> Box<dyn Custom> {
    let write_any = (**self.bufwriter.as_ref().unwrap().get_ref()).as_any();
    if let Some(file) = write_any.downcast_ref::<File>() {
      if let Ok(f) = file.try_clone() {
        return Box::new(BufWriteCustom{ bufwriter: Some(io::BufWriter::new(Box::new(f))) })
      }
    } else if write_any.downcast_ref::<io::Stdout>().is_some() {
      return Box::new(BufWriteCustom{ bufwriter: Some(io::BufWriter::new(Box::new(io::stdout()))) })
    } else if write_any.downcast_ref::<io::Stderr>().is_some() {
      return Box::new(BufWriteCustom{ bufwriter: Some(io::BufWriter::new(Box::new(io::stderr()))) })
    } else if write_any.downcast_ref::<io::Empty>().is_some() {
      return Box::new(BufWriteCustom{ bufwriter: Some(io::BufWriter::new(Box::new(io::empty()))) })
    }
    Box::new(Void{})
  }
}
impl Drop for BufWriteCustom {
  fn drop(&mut self) {
    if let Err(e) = self.bufwriter.as_mut().unwrap().flush() {
      let _ = io::stderr().write(format!("{e}").as_bytes());
    }
  }
}

macro_rules! flush {
  ($f:expr) => {
    if let Err(e) = $f.flush() {
      let _ = io::stderr().write(format!("{e}").as_bytes()); }
  }
}

pub fn questionmark(state: &CognitionState, f: &mut dyn io::Write) {
  fwrite_check!(f, GRN);
  println!("STACK:");
  fwrite_check!(f, COLOR_RESET);
  let cur = state.current_ref();
  for v in cur.stack.iter() { v.fprint(f, "\n"); }
  fwrite_check!(f, b"\n");
  flush!(f);
}

pub fn cog_questionmark(state: CognitionState, _: Option<&Value>) -> CognitionState {
  questionmark(&state, &mut io::stdout());
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
  let mut stdout = io::stdout();
  for wv in v.value_stack_ref().iter() {
    if let Err(e) = stdout.write_all(wv.vword_ref().str_word.as_bytes()) {
      let _ = io::stderr().write(format!("{e}").as_bytes()); }}
  flush!(stdout);
  state.pool.add_val(v);
  state
}

pub fn cog_read(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut vword = state.pool.get_vword(DEFAULT_STRING_LENGTH);
  let stream = io::stdin();
  if stream.read_line(&mut vword.str_word).is_err() {
    state.pool.add_vword(vword);
    return state.eval_error("READ FAILED", w);
  }
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_stdout(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let boxed_custom = Box::new(WriteCustom{ writer: Some(Box::new(io::stdout())) });
  let vcustom = VCustom::with_custom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_stdin(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let boxed_custom = Box::new(ReadCustom{ reader: Some(Box::new(io::stdin())) });
  let vcustom = VCustom::with_custom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_stderr(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let boxed_custom = Box::new(WriteCustom{ writer: Some(Box::new(io::stderr())) });
  let vcustom = VCustom::with_custom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_empty(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let boxed_custom = Box::new(ReadWriteCustom{ stream: Some(Box::new(io::empty())) });
  let vcustom = VCustom::with_custom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_fopen(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_word!(state, w);
  let string = &v.value_stack_ref().first().unwrap().vword_ref().str_word;
  let Ok(file) = File::options().read(true).create(false).open(string) else {
    state.current().stack.push(v);
    return state.eval_error("INVALID FILENAME", w);
  };
  let boxed_custom = Box::new(ReadCustom{ reader: Some(Box::new(file)) });
  let vcustom = VCustom::with_custom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_file(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_word!(state, w);
  let string = &v.value_stack_ref().first().unwrap().vword_ref().str_word;
  let boxed_custom = if let Ok(f) = File::options().write(true).read(true).create(true).open(string) {
    Box::new(FileCustom{ file: Some(f) })
  } else {
    state.current().stack.push(v);
    return state.eval_error("INVALID FILENAME", w);
  };
  let vcustom = VCustom::with_custom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_file_new(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_word!(state, w);
  let string = &v.value_stack_ref().first().unwrap().vword_ref().str_word;
  let boxed_custom = if let Ok(f) = File::options().write(true).read(true).create_new(true).open(string) {
    Box::new(FileCustom{ file: Some(f) })
  } else {
    state.current().stack.push(v);
    return state.eval_error("INVALID FILENAME", w);
  };
  let vcustom = VCustom::with_custom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_file_append(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_word!(state, w);
  let string = &v.value_stack_ref().first().unwrap().vword_ref().str_word;
  let boxed_custom = if let Ok(f) = File::options().append(true).read(true).create(true).open(string) {
    Box::new(FileCustom{ file: Some(f) })
  } else {
    state.current().stack.push(v);
    return state.eval_error("INVALID FILENAME", w);
  };
  let vcustom = VCustom::with_custom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_reader(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.last_mut() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let Value::Custom(vcustom) = v.value_stack().first_mut().unwrap() else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let custom = &mut vcustom.custom;
  if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
    let boxed: Box<dyn ReadAny> = Box::new(file.file.take().unwrap());
    let reader = Some(boxed);
    vcustom.custom = Box::new(ReadCustom{ reader });
  } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
    let boxed: Box<dyn ReadAny> = Box::new(stream.stream.take().unwrap());
    let reader = Some(boxed);
    vcustom.custom = Box::new(ReadCustom{ reader });
  } else if custom.as_any_mut().downcast_mut::<ReadCustom>().is_some() {
  } else if custom.as_any_mut().downcast_mut::<BufReadCustom>().is_some() {
  } else { return state.eval_error("BAD ARGUMENT TYPE", w) };
  state
}

pub fn cog_writer(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.last_mut() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let Value::Custom(vcustom) = v.value_stack().first_mut().unwrap() else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let custom = &mut vcustom.custom;
  if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
    let boxed: Box<dyn WriteAny> = Box::new(file.file.take().unwrap());
    let writer = Some(boxed);
    vcustom.custom = Box::new(WriteCustom{ writer });
  } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
    let boxed: Box<dyn WriteAny> = Box::new(stream.stream.take().unwrap());
    let writer = Some(boxed);
    vcustom.custom = Box::new(WriteCustom{ writer });
  } else if custom.as_any_mut().downcast_mut::<BufReadCustom>().is_some() {
  } else if custom.as_any_mut().downcast_mut::<BufWriteCustom>().is_some() {
  } else { return state.eval_error("BAD ARGUMENT TYPE", w) };
  state
}

pub fn cog_bufreader(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.last_mut() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let Value::Custom(vcustom) = v.value_stack().first_mut().unwrap() else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let custom = &mut vcustom.custom;
  let boxed: Box<dyn ReadAny> = if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
    Box::new(file.file.take().unwrap())
  } else if let Some(reader) = custom.as_any_mut().downcast_mut::<ReadCustom>() {
    reader.reader.take().unwrap()
  } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
     Box::new(stream.stream.take().unwrap())
  } else { return state.eval_error("BAD ARGUMENT TYPE", w) };
  let bufreader = Some(io::BufReader::new(boxed));
  vcustom.custom = Box::new(BufReadCustom{ bufreader });
  state
}

pub fn cog_bufwriter(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.last_mut() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let Value::Custom(vcustom) = v.value_stack().first_mut().unwrap() else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let custom = &mut vcustom.custom;
  let boxed: Box<dyn WriteAny> = if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
    Box::new(file.file.take().unwrap())
  } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
    writer.writer.take().unwrap()
  } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
     Box::new(stream.stream.take().unwrap())
  } else { return state.eval_error("BAD ARGUMENT TYPE", w) };
  let bufwriter = Some(io::BufWriter::new(boxed));
  vcustom.custom = Box::new(BufWriteCustom{ bufwriter });
  state
}

pub fn cog_unbuffer(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.last_mut() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let Value::Custom(vcustom) = v.value_stack().first_mut().unwrap() else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let custom = &mut vcustom.custom;
  if let Some(bufreader) = custom.as_any_mut().downcast_mut::<BufReadCustom>() {
    let boxed_unbuffered = bufreader.bufreader.take().unwrap().into_inner();
    vcustom.custom = Box::new(ReadCustom{ reader: Some(boxed_unbuffered) });
  } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
    let mut bufwriter = bufwriter.bufwriter.take().unwrap();
    let _ = bufwriter.flush();
    match bufwriter.into_inner() {
      Ok(boxed_unbuffered) => vcustom.custom = Box::new(WriteCustom{ writer: Some(boxed_unbuffered) }),
      Err(e) => {
        let _ = io::stderr().write(format!("{e}").as_bytes());
        vcustom.custom = Box::new(Void{})
      }
    }
  } else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  state
}

pub fn cog_stream(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.last_mut() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let Value::Custom(vcustom) = v.value_stack().first_mut().unwrap() else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let custom = &mut vcustom.custom;
  if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
    let boxed: Box<dyn ReadWriteAny> = Box::new(file.file.take().unwrap());
    let stream = Some(boxed);
    vcustom.custom = Box::new(ReadWriteCustom{ stream });
  } else if custom.as_any_mut().downcast_mut::<ReadWriteCustom>().is_some() {
  } else { return state.eval_error("BAD ARGUMENT TYPE", w) };
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
      let custom = &mut vcustom.custom;
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        questionmark(&state, &mut file.file.as_mut().unwrap());
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        questionmark(&state, writer.writer.as_mut().unwrap().as_write_mut());
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        questionmark(&state, stream.stream.as_mut().unwrap().as_write_mut());
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        questionmark(&state, bufwriter.bufwriter.as_mut().unwrap().as_write_mut());
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
      let custom = &mut vcustom.custom;
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        let print_v = stack.pop().unwrap();
        print_v.fprint(&mut file.file.as_mut().unwrap(), "\n");
        flush!(file.file.as_mut().unwrap());
        state.pool.add_val(print_v);
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        let print_v = stack.pop().unwrap();
        print_v.fprint(&mut writer.writer.as_mut().unwrap().as_write_mut(), "\n");
        flush!(writer.writer.as_mut().unwrap());
        state.pool.add_val(print_v);
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        let print_v = stack.pop().unwrap();
        print_v.fprint(&mut stream.stream.as_mut().unwrap().as_write_mut(), "\n");
        flush!(stream.stream.as_mut().unwrap());
        state.pool.add_val(print_v);
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        let print_v = stack.pop().unwrap();
        print_v.fprint(&mut bufwriter.bufwriter.as_mut().unwrap().as_write_mut(), "\n");
        flush!(bufwriter.bufwriter.as_mut().unwrap());
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
        flush!(file);
        state.pool.add_val(print_v);
      } else if let Ok(mut file) = File::open(&vword.str_word) {
        let print_v = stack.pop().unwrap();
        print_v.fprint(&mut file, "\n");
        flush!(file);
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
      let custom = &mut vcustom.custom;
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(file.file.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        flush!(file.file.as_mut().unwrap());
        state.pool.add_val(print_v);
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(writer.writer.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        flush!(writer.writer.as_mut().unwrap());
        state.pool.add_val(print_v);
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(stream.stream.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        flush!(stream.stream.as_mut().unwrap());
        state.pool.add_val(print_v);
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(bufwriter.bufwriter.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        flush!(bufwriter.bufwriter.as_mut().unwrap());
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
        flush!(file);
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
      let custom = &mut vcustom.custom;
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(file.file.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        flush!(file.file.as_mut().unwrap());
        state.pool.add_val(print_v);
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(writer.writer.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        flush!(writer.writer.as_mut().unwrap());
        state.pool.add_val(print_v);
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(stream.stream.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        flush!(stream.stream.as_mut().unwrap());
        state.pool.add_val(print_v);
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(bufwriter.bufwriter.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        flush!(bufwriter.bufwriter.as_mut().unwrap());
        state.pool.add_val(print_v);
      } else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v);
    },
    Value::Word(vword) => {
      if let Ok(mut file) = File::options().append(true).create(true).open(&vword.str_word) {
        let print_v = stack.pop().unwrap();
        fwrite_check!(file, &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        flush!(file);
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
      let custom = &mut vcustom.custom;
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        let mut vword = if let Ok(metadata) = file.file.as_mut().unwrap().metadata() {
          state.pool.get_vword(metadata.len() as usize)
        } else {
          stack.push(v);
          return state.eval_error("NO FILE METADATA", w)
        };
        if let Err(e) = file.file.as_mut().unwrap().read_to_string(&mut vword.str_word) {
          let _ = io::stderr().write(format!("{e}").as_bytes());
        }
        state.push_quoted(Value::Word(vword));
      } else if let Some(reader) = custom.as_any_mut().downcast_mut::<ReadCustom>() {
        let mut vword = state.pool.get_vword(DEFAULT_STRING_LENGTH);
        if let Err(e) = reader.reader.as_mut().unwrap().read_to_string(&mut vword.str_word) {
          let _ = io::stderr().write(format!("{e}").as_bytes());
        }
        state.push_quoted(Value::Word(vword));
      } else if let Some(bufreader) = custom.as_any_mut().downcast_mut::<BufReadCustom>() {
        let mut vword = state.pool.get_vword(DEFAULT_STRING_LENGTH);
        if let Err(e) = bufreader.bufreader.as_mut().unwrap().read_to_string(&mut vword.str_word) {
          let _ = io::stderr().write(format!("{e}").as_bytes());
        }
        state.push_quoted(Value::Word(vword));
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        let mut vword = state.pool.get_vword(DEFAULT_STRING_LENGTH);
        if let Err(e) = stream.stream.as_mut().unwrap().read_to_string(&mut vword.str_word) {
          let _ = io::stderr().write(format!("{e}").as_bytes());
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
          let _ = io::stderr().write(format!("{e}").as_bytes());
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

pub fn cog_read_until(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let mut v = stack.pop().unwrap();
  let v_delim = stack.last().unwrap();
  if v.value_stack_ref().len() != 1 || v_delim.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let Value::Word(delim_word) = v_delim.value_stack_ref().first().unwrap() else {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  if delim_word.str_word.len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let byte = delim_word.str_word.as_bytes()[0].clone();
  let val = v.value_stack().first_mut().unwrap();
  match val {
    Value::Custom(vcustom) => {
      let custom = &mut vcustom.custom;
      if let Some(bufreader) = custom.as_any_mut().downcast_mut::<BufReadCustom>() {
        let mut bytes = Vec::with_capacity(DEFAULT_BUFFER_CAPACITY);
        if let Err(e) = bufreader.bufreader.as_mut().unwrap().read_until(byte, &mut bytes) {
          let _ = io::stderr().write(format!("{e}").as_bytes());
        }

        let mut vword = state.pool.get_vword(0);
        state.pool.add_string(vword.str_word);
        vword.str_word = match String::from_utf8(bytes) {
          Ok(s) => s,
          Err(e) => {
            let _ = io::stderr().write(format!("{e}").as_bytes());
            state.current().stack.push(v);
            return state.eval_error("INVALID STRING", w)
          }
        };
        let c = state.current().stack.pop().unwrap();
        state.pool.add_val(c);
        state.push_quoted(Value::Word(vword));
      } else if let Some(reader) = custom.as_any_mut().downcast_mut::<ReadCustom>(){
        if let Some(stdin) = reader.reader.as_mut().unwrap().as_any_mut().downcast_mut::<io::Stdin>() {
          let mut bytes = Vec::with_capacity(DEFAULT_BUFFER_CAPACITY);
          let mut lock = stdin.lock();
          if let Err(e) = lock.read_until(byte, &mut bytes) {
            let _ = io::stderr().write(format!("{e}").as_bytes());
          }

          let mut vword = state.pool.get_vword(0);
          state.pool.add_string(vword.str_word);
          vword.str_word = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => {
              let _ = io::stderr().write(format!("{e}").as_bytes());
              state.current().stack.push(v);
              return state.eval_error("INVALID STRING", w)
            }
          };
          let c = state.current().stack.pop().unwrap();
          state.pool.add_val(c);
          state.push_quoted(Value::Word(vword));
        } else {
          stack.push(v);
          return state.eval_error("BAD ARGUMENT TYPE", w)
        }
      } else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v);
    },
    _ => return {
      stack.push(v);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

// pub fn cog_skip_until(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
//   let stack = &mut state.current().stack;
//   if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
//   let mut v = stack.pop().unwrap();
//   let v_delim = stack.last().unwrap();
//   if v.value_stack_ref().len() != 1 || v_delim.value_stack_ref().len() != 1 {
//     stack.push(v);
//     return state.eval_error("BAD ARGUMENT TYPE", w)
//   }
//   let Value::Word(delim_word) = v_delim.value_stack_ref().first().unwrap() else {
//     stack.push(v);
//     return state.eval_error("BAD ARGUMENT TYPE", w)
//   };
//   if delim_word.str_word.len() != 1 {
//     stack.push(v);
//     return state.eval_error("BAD ARGUMENT TYPE", w)
//   }
//   let byte = delim_word.str_word.as_bytes()[0].clone();
//   let val = v.value_stack().first_mut().unwrap();
//   match val {
//     Value::Custom(vcustom) => {
//       let custom = &mut vcustom.custom;
//       if let Some(bufreader) = custom.as_any_mut().downcast_mut::<BufReadCustom>() {
//         if let Err(e) = bufreader.bufreader.as_mut().unwrap().skip_until(byte) {
//           let _ = io::stderr().write(format!("{e}").as_bytes());
//         }
//         let c = state.current().stack.pop().unwrap();
//         state.pool.add_val(c);
//       } else if let Some(reader) = custom.as_any_mut().downcast_mut::<ReadCustom>(){
//         if let Some(stdin) = reader.reader.as_mut().unwrap().as_any_mut().downcast_mut::<io::Stdin>() {
//           if let Err(e) = stdin.lock().skip_until(byte) {
//             let _ = io::stderr().write(format!("{e}").as_bytes());
//           }
//           let c = state.current().stack.pop().unwrap();
//           state.pool.add_val(c);
//         } else {
//           stack.push(v);
//           return state.eval_error("BAD ARGUMENT TYPE", w)
//         }
//       } else {
//         stack.push(v);
//         return state.eval_error("BAD ARGUMENT TYPE", w)
//       }
//       state.current().stack.push(v);
//     },
//     _ => return {
//       stack.push(v);
//       state.eval_error("BAD ARGUMENT TYPE", w)
//     },
//   }
//   state
// }

pub fn cog_read_line(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let val = v.value_stack().first_mut().unwrap();
  match val {
    Value::Custom(vcustom) => {
      let custom = &mut vcustom.custom;
      if let Some(bufreader) = custom.as_any_mut().downcast_mut::<BufReadCustom>() {
        let mut vword = state.pool.get_vword(DEFAULT_STRING_LENGTH);
        if let Err(e) = bufreader.bufreader.as_mut().unwrap().read_line(&mut vword.str_word) {
          let _ = io::stderr().write(format!("{e}").as_bytes());
        }
        state.push_quoted(Value::Word(vword));
        state.current().stack.push(v);
      } else if let Some(reader) = custom.as_any_mut().downcast_mut::<ReadCustom>() {
        if let Some(stdin) = reader.reader.as_mut().unwrap().as_any_mut().downcast_mut::<io::Stdin>() {
          let mut vword = state.pool.get_vword(DEFAULT_STRING_LENGTH);
          if let Err(e) = stdin.read_line(&mut vword.str_word) {
            let _ = io::stderr().write(format!("{e}").as_bytes());
          }
          state.push_quoted(Value::Word(vword));
          state.current().stack.push(v);
        } else {
          stack.push(v);
          return state.eval_error("BAD ARGUMENT TYPE", w)
        }
      } else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      };
    },
    _ => return {
      stack.push(v);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

macro_rules! match_seek {
  ($state:ident,$w:ident,$stack:ident,$file:expr,$v:ident,$idxval:ident,$seek:expr) => {
    match $file.seek($seek) {
      Err(_) => {
        $stack.push($v);
        $stack.push($idxval);
        return $state.eval_error("SEEK FAILED", $w)
      },
      Ok(_) => $state.pool.add_val($idxval)
    }
  }
}
macro_rules! try_seek {
  ($state:ident,$w:ident,$stack:ident,$stream:expr,$v:ident,$idxval:ident,$seek:expr) => {
    if let Some(file) = $stream.as_mut().unwrap().as_any_mut().downcast_mut::<File>() {
      match_seek!($state, $w, $stack, file, $v, $idxval, $seek);
    } else if $stream.as_mut().unwrap().as_any_mut().downcast_mut::<io::Empty>().is_some() {
      $state.pool.add_val($idxval)
    } else {
      $stack.push($v);
      $stack.push($idxval);
      return $state.eval_error("NOT SEEKABLE", $w)
    }
  }
}

pub fn cog_seek(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let idx = get_unsigned!(state, w, isize, ACTIVE) as usize;
  let idxval = if idx > u64::MAX as usize {
    return state.eval_error("OUT OF BOUNDS", w);
  } else {
    state.current().stack.pop().unwrap()
  };
  let idx = idx as u64;
  let stack = &mut state.current().stack;
  let mut v = stack.pop().unwrap();
  if v.value_stack_ref().len() != 1 {
    stack.push(v);
    stack.push(idxval);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let val = v.value_stack().first_mut().unwrap();
  match val {
    Value::Custom(vcustom) => {
      let custom = &mut vcustom.custom;
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        match_seek!(state, w, stack, file.file.as_mut().unwrap(), v, idxval, io::SeekFrom::Start(idx));
      } else if let Some(bufreader) = custom.as_any_mut().downcast_mut::<BufReadCustom>() {
        try_seek!(state, w, stack, bufreader.bufreader, v, idxval, io::SeekFrom::Start(idx));
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        try_seek!(state, w, stack, bufwriter.bufwriter, v, idxval, io::SeekFrom::Start(idx));
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        if ReadWriteAny::as_any_mut(stream.stream.as_mut().unwrap()).downcast_mut::<io::Empty>().is_some() {
          state.pool.add_val(idxval)
        } else if let Some(file) = ReadWriteAny::as_any_mut(stream.stream.as_mut().unwrap()).downcast_mut::<File>() {
          match_seek!(state, w, stack, file, v, idxval, io::SeekFrom::Start(idx));
        } else {
          stack.push(v);
          stack.push(idxval);
          return state.eval_error("NOT SEEKABLE", w)
        }
      } else {
        stack.push(v);
        stack.push(idxval);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v);
    },
    _ => return {
      stack.push(v);
      stack.push(idxval);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

pub fn cog_seek_end(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let val = v.value_stack().first_mut().unwrap();
  match val {
    Value::Custom(vcustom) => {
      let custom = &mut vcustom.custom;
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        if file.file.as_mut().unwrap().seek(io::SeekFrom::End(0)).is_err() {
          return state.eval_error("SEEK FAILED", w)
        }
      } else if let Some(bufreader) = custom.as_any_mut().downcast_mut::<BufReadCustom>() {
        if let Some(file) = bufreader.bufreader.as_mut().unwrap().as_any_mut().downcast_mut::<File>() {
          if file.seek(io::SeekFrom::End(0)).is_err() {
            return state.eval_error("SEEK FAILED", w)
          }
        } else {
          stack.push(v);
          return state.eval_error("NOT SEEKABLE", w)
        }
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        if let Some(file) = bufwriter.bufwriter.as_mut().unwrap().as_any_mut().downcast_mut::<File>() {
          if file.seek(io::SeekFrom::End(0)).is_err() {
            return state.eval_error("SEEK FAILED", w)
          }
        } else {
          stack.push(v);
          return state.eval_error("NOT SEEKABLE", w)
        }
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        if let Some(file) = ReadWriteAny::as_any_mut(stream.stream.as_mut().unwrap()).downcast_mut::<File>() {
          if file.seek(io::SeekFrom::End(0)).is_err() {
            return state.eval_error("SEEK FAILED", w)
          }
        } else if ReadWriteAny::as_any(stream.stream.as_ref().unwrap()).downcast_ref::<io::Empty>().is_none() {
          stack.push(v);
          return state.eval_error("NOT SEEKABLE", w)
        }
      } else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v);
    },
    _ => {
      stack.push(v);
      return state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

macro_rules! streampos {
  ($state:ident,$w:ident,$stream:expr,$v:ident) => {
    match $stream.stream_position() {
      Ok(i) => {
        let Some(math) = $state.current().math.take() else {
          $state.current().stack.push($v);
          return $state.eval_error("MATH BASE ZERO", $w)
        };
        if i as usize > isize::MAX as usize {
          $state.current().stack.push($v);
          return $state.eval_error("OUT OF BOUNDS", $w)
        }
        match math.itos(i as isize, &mut $state) {
          Ok(s) => {
            $state.current().math = Some(math);
            let mut vword = $state.pool.get_vword(s.len());
            vword.str_word.push_str(&s);
            $state.pool.add_string(s);
            $state.current().stack.push($v);
            $state.push_quoted(Value::Word(vword))
          },
          Err(e) => {
            $state.current().math = Some(math);
            $state.current().stack.push($v);
            return $state.eval_error(e, $w)
          }
        }
      },
      Err(_) => {
        $state.current().stack.push($v);
        return $state.eval_error("STREAM POSITION FAILED", $w)
      }
    }
  }
}

pub fn cog_streampos(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let val = v.value_stack().first_mut().unwrap();
  match val {
    Value::Custom(vcustom) => {
      let custom = &mut vcustom.custom;
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        streampos!(state, w, file.file.as_mut().unwrap(), v);
      } else if let Some(bufreader) = custom.as_any_mut().downcast_mut::<BufReadCustom>() {
        if let Some(file) = bufreader.bufreader.as_mut().unwrap().as_any_mut().downcast_mut::<File>() {
          streampos!(state, w, file, v);
        } else if let Some(empty) = bufreader.bufreader.as_mut().unwrap().as_any_mut().downcast_mut::<io::Empty>() {
          streampos!(state, w, empty, v);
        } else {
          stack.push(v);
          return state.eval_error("NOT SEEKABLE", w)
        }
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        if let Some(file) = bufwriter.bufwriter.as_mut().unwrap().as_any_mut().downcast_mut::<File>() {
          streampos!(state, w, file, v);
        } else if let Some(empty) = bufwriter.bufwriter.as_mut().unwrap().as_any_mut().downcast_mut::<io::Empty>() {
          streampos!(state, w, empty, v);
        } else {
          stack.push(v);
          return state.eval_error("NOT SEEKABLE", w)
        }
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        if let Some(file) = ReadWriteAny::as_any_mut(stream.stream.as_mut().unwrap()).downcast_mut::<File>() {
          streampos!(state, w, file, v);
        } else if let Some(empty) = ReadWriteAny::as_any_mut(stream.stream.as_mut().unwrap()).downcast_mut::<io::Empty>() {
          streampos!(state, w, empty, v);
        } else {
          stack.push(v);
          return state.eval_error("NOT SEEKABLE", w)
        }
      } else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
    },
    _ => {
      stack.push(v);
      return state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

pub fn cog_seek_relative(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let idx = get_int!(state, w, i64, ACTIVE);
  let stack = &mut state.current().stack;
  let idxval = stack.pop().unwrap();
  let mut v = stack.pop().unwrap();
  if v.value_stack_ref().len() != 1 {
    stack.push(v);
    stack.push(idxval);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let val = v.value_stack().first_mut().unwrap();
  match val {
    Value::Custom(vcustom) => {
      let custom = &mut vcustom.custom;
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        match_seek!(state, w, stack, file.file.as_mut().unwrap(), v, idxval, io::SeekFrom::Current(idx));
      } else if let Some(bufreader) = custom.as_any_mut().downcast_mut::<BufReadCustom>() {
        try_seek!(state, w, stack, bufreader.bufreader, v, idxval, io::SeekFrom::Current(idx));
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        try_seek!(state, w, stack, bufwriter.bufwriter, v, idxval, io::SeekFrom::Current(idx));
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        if ReadWriteAny::as_any_mut(stream.stream.as_mut().unwrap()).downcast_mut::<io::Empty>().is_some() {
          state.pool.add_val(idxval)
        } else if let Some(file) = ReadWriteAny::as_any_mut(stream.stream.as_mut().unwrap()).downcast_mut::<File>() {
          match_seek!(state, w, stack, file, v, idxval, io::SeekFrom::Current(idx));
        } else {
          stack.push(v);
          stack.push(idxval);
          return state.eval_error("NOT SEEKABLE", w)
        }
      } else {
        stack.push(v);
        stack.push(idxval);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v);
    },
    _ => return {
      stack.push(v);
      stack.push(idxval);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

macro_rules! iotype_questionmark {
  ($state:ident,$w:ident,$type:ty) => {{
    let Some(v) = $state.current_ref().stack.last() else {
      return $state.eval_error("TOO FEW ARGUMENTS", $w)
    };
    let vstack = v.value_stack_ref();
    if vstack.len() == 1 {
      if let Value::Custom(vcustom) = vstack.first().unwrap() {
        let custom = &vcustom.custom;
        if custom.as_any().downcast_ref::<$type>().is_some() {
          let mut vw = $state.pool.get_vword(1);
          vw.str_word.push('t');
          $state.push_quoted(Value::Word(vw));
          return $state
        }
      }
    }
    let vw = $state.pool.get_vword(0);
    $state.push_quoted(Value::Word(vw));
    $state
  }}
}

pub fn cog_file_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  iotype_questionmark!(state, w, FileCustom)
}
pub fn cog_reader_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  iotype_questionmark!(state, w, ReadCustom)
}
pub fn cog_writer_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  iotype_questionmark!(state, w, WriteCustom)
}
pub fn cog_bufreader_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  iotype_questionmark!(state, w, BufReadCustom)
}
pub fn cog_bufwriter_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  iotype_questionmark!(state, w, BufWriteCustom)
}
pub fn cog_stream_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  iotype_questionmark!(state, w, ReadWriteCustom)
}

pub fn add_builtins(state: &mut CognitionState) {
  add_builtin!(state, "?", cog_questionmark);
  add_builtin!(state, ".", cog_period);
  add_builtin!(state, "print", cog_print);
  add_builtin!(state, "read", cog_read);
  add_builtin!(state, "stdout", cog_stdout);
  add_builtin!(state, "stdin", cog_stdin);
  add_builtin!(state, "stderr", cog_stderr);
  add_builtin!(state, "stdout", cog_stdout);
  add_builtin!(state, "empty", cog_empty);
  add_builtin!(state, "fopen", cog_fopen);
  add_builtin!(state, "file", cog_file);
  add_builtin!(state, "file-new", cog_file_new);
  add_builtin!(state, "file-append", cog_file_append);
  add_builtin!(state, "reader", cog_reader);
  add_builtin!(state, "writer", cog_writer);
  add_builtin!(state, "bufreader", cog_bufreader);
  add_builtin!(state, "bufwriter", cog_bufwriter);
  add_builtin!(state, "unbuffer", cog_unbuffer);
  add_builtin!(state, "stream", cog_stream);
  add_builtin!(state, "f?", cog_fquestionmark);
  add_builtin!(state, "f.", cog_fperiod);
  add_builtin!(state, "fwrite", cog_fwrite);
  add_builtin!(state, "fprint", cog_fprint);
  add_builtin!(state, "fread", cog_fread);
  add_builtin!(state, "read-until", cog_read_until);
  // add_builtin!(state, "skip-until", cog_skip_until); // currently experimental rust stdlib feature
  add_builtin!(state, "read-line", cog_read_line);
  add_builtin!(state, "seek", cog_seek);
  add_builtin!(state, "seek-end", cog_seek_end);
  add_builtin!(state, "streampos", cog_streampos);
  add_builtin!(state, "seek-relative", cog_seek_relative);
  add_builtin!(state, "file?", cog_file_questionmark);
  add_builtin!(state, "reader?", cog_reader_questionmark);
  add_builtin!(state, "writer?", cog_writer_questionmark);
  add_builtin!(state, "bufreader?", cog_bufreader_questionmark);
  add_builtin!(state, "bufwriter?", cog_bufwriter_questionmark);
  add_builtin!(state, "stream?", cog_stream_questionmark);
}
