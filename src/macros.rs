#![allow(unused_imports)]

pub const RED: &[u8] = b"\x1B[31m";
pub const HBLK: &[u8] = b"\x1B[90m";
//pub const COLOR_RESET: &[u8] = b"\x1B[39m";
pub const COLOR_RESET: &[u8] = b"\x1B[0m";

pub const DEFAULT_STACK_SIZE: usize = 24;
pub const DEFAULT_STRING_LENGTH: usize = 24;
pub const DEFAULT_WORD_TABLE_SIZE: usize = 576;

#[macro_export]
macro_rules! default_fprint_error {
  ($e:literal) => {
    print!("Value::fprint(): error: ");
    println!($e);
  }
}
#[macro_export]
macro_rules! fwrite_check_byte {
  ($f:ident,$s:expr,$n:ident) => {
    let n = $n;
    match $f.write($s) {
      Ok(1) => {},
      Ok(_) => { $crate::default_fprint_error!("not all bytes could be written");
                 $crate::default_fprint_error!("wrote {n} bytes"); },
      Err(e) => { $crate::default_fprint_error!("{e}"); },
    }
  }
}

#[macro_export]
macro_rules! fwrite_check {
  ($f:ident,$s:expr) => {
    let s: &[u8] = $s;
    for n in 0..s.len() {
      $crate::fwrite_check_byte!($f, &s[n..n+1], n);
    }
  }
}
// pub(crate) use fwrite_check;

#[macro_export]
macro_rules! fwrite_check_pretty {
  ($f:ident,$s:expr) => {
    let s: &[u8] = $s;
    for n in 0..s.len() {
      match s[n] {
        b'\n' => {
          $crate::fwrite_check_byte!($f, b"\\", n);
          $crate::fwrite_check_byte!($f, b"n", n);
        },
        b'\t' => {
          $crate::fwrite_check_byte!($f, b"\\", n);
          $crate::fwrite_check_byte!($f, b"t", n);
        },
        _ => {
          $crate::fwrite_check_byte!($f, &s[n..n+1], n);
        },
      }
    }
  }
}
// pub(crate) use fwrite_check_pretty;

#[macro_export]
macro_rules! build_macro {
  ($state:ident) => {
    $state.pool.get_vmacro($crate::macros::DEFAULT_STACK_SIZE)
  };
  ($state:ident,$name:literal, $f:ident) => {{
    let mut m = build_macro!($state);
    let $crate::Value::Macro(vmacro) = &mut m else { panic!("Pool::get_vmacro() failed") };

    let mut v = $state.pool.get_vfllib($f);
    let $crate::Value::FLLib(vfllib) = &mut v else { panic!("Pool::get_vfllib() failed") };
    vfllib.str_word = Some(String::from($name));

    vmacro.macro_stack.push(v);
    m
  }};
  ($state:ident,($fi:ident),*,$fn:ident) => {
    let mut m = build_macro!($state, ($fi),*);
    let $crate::Value::Macro(vmacro) = &mut m else { panic!("Pool::get_vmacro() failed") };

    let v = $state.pool.get_vfllib($fn);
    vmacro.macro_stack.push(v);
    m
  }
}

/// add_word!(state: CognitionState, name: &'static str, f1, f2, ... fn: CognitionFunction);
/// will mutate state (whether or not passed in as &mut) and insert a macro word into state
/// current stack's word_table. If only one CognitionFunction parameter was given, then the
/// resulting fllib str_word value will be derived from 'name'. Otherwise, it will be None.
/// Currently, add_word! only takes one CognitionFunction parameter.
#[macro_export]
macro_rules! add_word {
  ($state:ident,$name:literal,$f:ident) => {
    let m = build_macro!($state, $name, $f);
    $state.current().add_word(m, $name);
  };
  ($state:ident,$name:literal,($f:ident),*) => {
    let m = build_macro!($state, ($f),*)
    $state.current().add_word(m, $name);
  }
}
// pub(crate) use add_word;
