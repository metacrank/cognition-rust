use crate::*;
use std::io::*;
use std::fs::File;
use std::fs::OpenOptions;

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

pub trait WriteAny: Any + Write {
  trait_any!();
  fn as_write_mut(&mut self) -> &mut dyn Write;
}
impl<T: Any + Write> WriteAny for T {
  impl_any!();
  fn as_write_mut(&mut self) -> &mut dyn Write { self }
}

pub trait ReadAny: Any + Read { trait_any!(); }
impl<T: Any + Read> ReadAny for T { impl_any!(); }

pub trait ReadWriteAny: Read + Write + Any { trait_any!(); }
impl<T: Any + Read + Write> ReadWriteAny for T { impl_any!(); }

// Always unwrap Options
pub struct ReadWriteCustom { stream: Option<Box<dyn ReadWriteAny>> }
pub struct FileCustom  { file: Option<File> }
pub struct ReadCustom  { reader: Option<Box<dyn ReadAny>> }
pub struct WriteCustom { writer: Option<Box<dyn WriteAny>> }
pub struct BufReadCustom  { bufreader: Option<BufReader<Box<dyn ReadAny>>> }
pub struct BufWriteCustom { bufwriter: Option<BufWriter<Box<dyn WriteAny>>> }

impl Custom for ReadWriteCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    let read_write_any = ReadWriteAny::as_any(&**self.stream.as_ref().unwrap());
    if read_write_any.downcast_ref::<Empty>().is_some() {
      fwrite_check!(f, b"(empty)");
    } else {
      fwrite_check!(f, b"(stream)");
    }
  }
  fn copyfunc(&self) -> Box<dyn CustomAny> {
    let read_write_any = ReadWriteAny::as_any(&**self.stream.as_ref().unwrap());
    if let Some(file) = read_write_any.downcast_ref::<File>() {
      if let Ok(f) = file.try_clone() {
        return Box::new(ReadWriteCustom{ stream: Some(Box::new(f)) })
      }
    } else if read_write_any.downcast_ref::<Empty>().is_some() {
      return Box::new(ReadWriteCustom{ stream: Some(Box::new(empty())) })
    }
    Box::new(Void{})
  }
}
impl Custom for FileCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(file)");
  }
  fn copyfunc(&self) -> Box<dyn CustomAny> {
    match self.file.as_ref().unwrap().try_clone() {
      Ok(f) => Box::new(FileCustom{ file: Some(f) }),
      Err(_)   => Box::new(Void{}),
    }
  }
}
impl Custom for ReadCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    let read_any = (**self.reader.as_ref().unwrap()).as_any();
    if read_any.downcast_ref::<Stdin>().is_some() {
      fwrite_check!(f, b"(stdin)");
    } else {
      fwrite_check!(f, b"(reader)");
    }
  }
  fn copyfunc(&self) -> Box<dyn CustomAny> {
    let read_any = (**self.reader.as_ref().unwrap()).as_any();
    if let Some(file) = read_any.downcast_ref::<File>() {
      if let Ok(f) = file.try_clone() {
        return Box::new(ReadCustom{ reader: Some(Box::new(f)) })
      }
    } else if read_any.downcast_ref::<Stdin>().is_some() {
      return Box::new(ReadCustom{ reader: Some(Box::new(stdin())) })
    } else if read_any.downcast_ref::<Empty>().is_some() {
      return Box::new(ReadCustom{ reader: Some(Box::new(empty())) })
    }
    Box::new(Void{})
  }
}
impl Custom for WriteCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    let read_any = (**self.writer.as_ref().unwrap()).as_any();
    if read_any.downcast_ref::<Stdout>().is_some() {
      fwrite_check!(f, b"(stdout)");
    } else if read_any.downcast_ref::<Stderr>().is_some() {
      fwrite_check!(f, b"(stderr)");
    } else {
      fwrite_check!(f, b"(writer)");
    }
  }
  fn copyfunc(&self) -> Box<dyn CustomAny> {
    let write_any = (**self.writer.as_ref().unwrap()).as_any();
    if let Some(file) = write_any.downcast_ref::<File>() {
      if let Ok(f) = file.try_clone() {
        return Box::new(WriteCustom{ writer: Some(Box::new(f)) })
      }
    } else if write_any.downcast_ref::<Stdout>().is_some() {
      return Box::new(WriteCustom{ writer: Some(Box::new(stdout())) })
    } else if write_any.downcast_ref::<Stderr>().is_some() {
      return Box::new(WriteCustom{ writer: Some(Box::new(stderr())) })
    } else if write_any.downcast_ref::<Empty>().is_some() {
      return Box::new(WriteCustom{ writer: Some(Box::new(empty())) })
    }
    Box::new(Void{})
  }
}
impl Custom for BufReadCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(bufreader)");
  }
  fn copyfunc(&self) -> Box<dyn CustomAny> {
    let read_any = (**self.bufreader.as_ref().unwrap().get_ref()).as_any();
    if let Some(file) = read_any.downcast_ref::<File>() {
      if let Ok(f) = file.try_clone() {
        return Box::new(BufReadCustom{ bufreader: Some(BufReader::new(Box::new(f))) })
      }
    } else if read_any.downcast_ref::<Stdin>().is_some() {
      return Box::new(BufReadCustom{ bufreader: Some(BufReader::new(Box::new(stdin()))) })
    } else if read_any.downcast_ref::<Empty>().is_some() {
      return Box::new(BufReadCustom{ bufreader: Some(BufReader::new(Box::new(empty()))) })
    }
    Box::new(Void{})
  }
}
impl Custom for BufWriteCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(bufwriter)");
  }
  fn copyfunc(&self) -> Box<dyn CustomAny> {
    let write_any = (**self.bufwriter.as_ref().unwrap().get_ref()).as_any();
    if let Some(file) = write_any.downcast_ref::<File>() {
      if let Ok(f) = file.try_clone() {
        return Box::new(BufWriteCustom{ bufwriter: Some(BufWriter::new(Box::new(f))) })
      }
    } else if write_any.downcast_ref::<Stdout>().is_some() {
      return Box::new(BufWriteCustom{ bufwriter: Some(BufWriter::new(Box::new(stdout()))) })
    } else if write_any.downcast_ref::<Stderr>().is_some() {
      return Box::new(BufWriteCustom{ bufwriter: Some(BufWriter::new(Box::new(stderr()))) })
    } else if write_any.downcast_ref::<Empty>().is_some() {
      return Box::new(BufWriteCustom{ bufwriter: Some(BufWriter::new(Box::new(empty()))) })
    }
    Box::new(Void{})
  }
}
impl Drop for BufWriteCustom {
  fn drop(&mut self) {
    if let Err(e) = self.bufwriter.as_mut().unwrap().flush() {
      let _ = stderr().write(format!("{e}").as_bytes());
    }
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
  let boxed_custom = Box::new(WriteCustom{ writer: Some(Box::new(stdout())) });
  let vcustom = state.pool.get_vcustom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_stdin(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let boxed_custom = Box::new(ReadCustom{ reader: Some(Box::new(stdin())) });
  let vcustom = state.pool.get_vcustom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_stderr(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let boxed_custom = Box::new(WriteCustom{ writer: Some(Box::new(stderr())) });
  let vcustom = state.pool.get_vcustom(boxed_custom);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_empty(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let boxed_custom = Box::new(ReadWriteCustom{ stream: Some(Box::new(empty())) });
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
  let boxed_custom = Box::new(ReadCustom{ reader: Some(Box::new(file)) });
  let vcustom = state.pool.get_vcustom(boxed_custom);
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
  let vcustom = state.pool.get_vcustom(boxed_custom);
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
  let vcustom = state.pool.get_vcustom(boxed_custom);
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
  let vcustom = state.pool.get_vcustom(boxed_custom);
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
  let Some(custom) = &mut vcustom.custom else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
    let boxed: Box<dyn ReadAny> = Box::new(file.file.take().unwrap());
    let reader = Some(boxed);
    vcustom.custom = Some(Box::new(ReadCustom{ reader }));
  } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
    let boxed: Box<dyn ReadAny> = Box::new(stream.stream.take().unwrap());
    let reader = Some(boxed);
    vcustom.custom = Some(Box::new(ReadCustom{ reader }));
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
  let Some(custom) = &mut vcustom.custom else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
    let boxed: Box<dyn WriteAny> = Box::new(file.file.take().unwrap());
    let writer = Some(boxed);
    vcustom.custom = Some(Box::new(WriteCustom{ writer }));
  } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
    let boxed: Box<dyn WriteAny> = Box::new(stream.stream.take().unwrap());
    let writer = Some(boxed);
    vcustom.custom = Some(Box::new(WriteCustom{ writer }));
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
  let Some(custom) = &mut vcustom.custom else { return state.eval_error("BAD ARGUMENT TYPE", w) };
  let boxed: Box<dyn ReadAny> = if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
    Box::new(file.file.take().unwrap())
  } else if let Some(reader) = custom.as_any_mut().downcast_mut::<ReadCustom>() {
    reader.reader.take().unwrap()
  } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
     Box::new(stream.stream.take().unwrap())
  } else { return state.eval_error("BAD ARGUMENT TYPE", w) };
  let bufreader = Some(BufReader::new(boxed));
  vcustom.custom = Some(Box::new(BufReadCustom{ bufreader }));
  state
}

pub fn cog_bufwriter(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.last_mut() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let Value::Custom(vcustom) = v.value_stack().first_mut().unwrap() else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let Some(custom) = &mut vcustom.custom else { return state.eval_error("BAD ARGUMENT TYPE", w) };
  let boxed: Box<dyn WriteAny> = if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
    Box::new(file.file.take().unwrap())
  } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
    writer.writer.take().unwrap()
  } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
     Box::new(stream.stream.take().unwrap())
  } else { return state.eval_error("BAD ARGUMENT TYPE", w) };
  let bufwriter = Some(BufWriter::new(boxed));
  vcustom.custom = Some(Box::new(BufWriteCustom{ bufwriter }));
  state
}

pub fn cog_unbuffer(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.last_mut() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let Value::Custom(vcustom) = v.value_stack().first_mut().unwrap() else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let Some(custom) = &mut vcustom.custom else { return state.eval_error("BAD ARGUMENT TYPE", w) };
  if let Some(bufreader) = custom.as_any_mut().downcast_mut::<BufReadCustom>() {
    let boxed_unbuffered = bufreader.bufreader.take().unwrap().into_inner();
    vcustom.custom = Some(Box::new(ReadCustom{ reader: Some(boxed_unbuffered) }));
  } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
    let mut bufwriter = bufwriter.bufwriter.take().unwrap();
    let _ = bufwriter.flush();
    match bufwriter.into_inner() {
      Ok(boxed_unbuffered) => vcustom.custom = Some(Box::new(WriteCustom{ writer: Some(boxed_unbuffered) })),
      Err(e) => {
        let _ = stderr().write(format!("{e}").as_bytes());
        vcustom.custom = Some(Box::new(Void{}))
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
  let Some(custom) = &mut vcustom.custom else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
    let boxed: Box<dyn ReadWriteAny> = Box::new(file.file.take().unwrap());
    let stream = Some(boxed);
    vcustom.custom = Some(Box::new(ReadWriteCustom{ stream }));
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
      let Some(custom) = &mut vcustom.custom else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      };
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
      let Some(custom) = &mut vcustom.custom else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      };
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        let print_v = stack.pop().unwrap();
        print_v.fprint(&mut file.file.as_mut().unwrap(), "\n");
        state.pool.add_val(print_v);
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        let print_v = stack.pop().unwrap();
        print_v.fprint(&mut writer.writer.as_mut().unwrap().as_write_mut(), "\n");
        state.pool.add_val(print_v);
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        let print_v = stack.pop().unwrap();
        print_v.fprint(&mut stream.stream.as_mut().unwrap().as_write_mut(), "\n");
        state.pool.add_val(print_v);
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        let print_v = stack.pop().unwrap();
        print_v.fprint(&mut bufwriter.bufwriter.as_mut().unwrap().as_write_mut(), "\n");
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
        fwrite_check!(file.file.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        state.pool.add_val(print_v);
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(writer.writer.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        state.pool.add_val(print_v);
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(stream.stream.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        state.pool.add_val(print_v);
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(bufwriter.bufwriter.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
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
        fwrite_check!(file.file.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        state.pool.add_val(print_v);
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(writer.writer.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        state.pool.add_val(print_v);
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(stream.stream.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
        state.pool.add_val(print_v);
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        let print_v = stack.pop().unwrap();
        fwrite_check!(bufwriter.bufwriter.as_mut().unwrap(), &print_v.value_stack_ref().first().unwrap().vword_ref().str_word.as_bytes());
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
        let mut vword = if let Ok(metadata) = file.file.as_mut().unwrap().metadata() {
          state.pool.get_vword(metadata.len() as usize)
        } else {
          stack.push(v);
          return state.eval_error("NO FILE METADATA", w)
        };
        if let Err(e) = file.file.as_mut().unwrap().read_to_string(&mut vword.str_word) {
          let _ = stderr().write(format!("{e}").as_bytes());
        }
        state.push_quoted(Value::Word(vword));
      } else if let Some(reader) = custom.as_any_mut().downcast_mut::<ReadCustom>() {
        let mut vword = state.pool.get_vword(DEFAULT_STRING_LENGTH);
        if let Err(e) = reader.reader.as_mut().unwrap().read_to_string(&mut vword.str_word) {
          let _ = stderr().write(format!("{e}").as_bytes());
        }
        state.push_quoted(Value::Word(vword));
      } else if let Some(bufreader) = custom.as_any_mut().downcast_mut::<BufReadCustom>() {
        let mut vword = state.pool.get_vword(DEFAULT_STRING_LENGTH);
        if let Err(e) = bufreader.bufreader.as_mut().unwrap().read_to_string(&mut vword.str_word) {
          let _ = stderr().write(format!("{e}").as_bytes());
        }
        state.push_quoted(Value::Word(vword));
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        let mut vword = state.pool.get_vword(DEFAULT_STRING_LENGTH);
        if let Err(e) = stream.stream.as_mut().unwrap().read_to_string(&mut vword.str_word) {
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
      let Some(custom) = &mut vcustom.custom else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      };
      let Some(bufreader) = custom.as_any_mut().downcast_mut::<BufReadCustom>() else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      };
      let mut bytes = Vec::with_capacity(DEFAULT_BUFFER_CAPACITY);
      if let Err(e) = bufreader.bufreader.as_mut().unwrap().read_until(byte, &mut bytes) {
        let _ = stderr().write(format!("{e}").as_bytes());
      }

      let mut vword = state.pool.get_vword(0);
      state.pool.add_string(vword.str_word);
      vword.str_word = match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => {
          let _ = stderr().write(format!("{e}").as_bytes());
          state.current().stack.push(v);
          return state.eval_error("INVALID STRING", w)
        }
      };
      let c = state.current().stack.pop().unwrap();
      state.pool.add_val(c);
      state.push_quoted(Value::Word(vword));
      state.current().stack.push(v);
    },
    _ => return {
      stack.push(v);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

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
      let Some(custom) = &mut vcustom.custom else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      };
      let Some(bufreader) = custom.as_any_mut().downcast_mut::<BufReadCustom>() else {
        stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      };
      let mut vword = state.pool.get_vword(DEFAULT_STRING_LENGTH);
      if let Err(e) = bufreader.bufreader.as_mut().unwrap().read_line(&mut vword.str_word) {
        let _ = stderr().write(format!("{e}").as_bytes());
      }
      state.push_quoted(Value::Word(vword));
      state.current().stack.push(v);
    },
    _ => return {
      stack.push(v);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

// TODO: add seeking with the Seek trait
pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "?", cog_questionmark);
  add_word!(state, ".", cog_period);
  add_word!(state, "print", cog_print);
  add_word!(state, "read", cog_read);
  add_word!(state, "stdout", cog_stdout);
  add_word!(state, "stdin", cog_stdin);
  add_word!(state, "stderr", cog_stderr);
  add_word!(state, "stdout", cog_stdout);
  add_word!(state, "empty", cog_empty);
  add_word!(state, "fopen", cog_fopen);
  add_word!(state, "file", cog_file);
  add_word!(state, "file-new", cog_file_new);
  add_word!(state, "file-append", cog_file_append);
  add_word!(state, "reader", cog_reader);
  add_word!(state, "writer", cog_writer);
  add_word!(state, "bufreader", cog_bufreader);
  add_word!(state, "bufwriter", cog_bufwriter);
  add_word!(state, "unbuffer", cog_unbuffer);
  add_word!(state, "stream", cog_stream);
  add_word!(state, "f?", cog_fquestionmark);
  add_word!(state, "f.", cog_fperiod);
  add_word!(state, "fwrite", cog_fwrite);
  add_word!(state, "fprint", cog_fprint);
  add_word!(state, "fread", cog_fread);
  add_word!(state, "read-until", cog_read_until);
  add_word!(state, "read-line", cog_read_line);
}
