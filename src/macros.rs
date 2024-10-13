pub const RED: &[u8] = b"\x1B[31m";
pub const HBLK: &[u8] = b"\x1B[90m";
//pub const COLOR_RESET: &[u8] = b"\x1B[39m";
pub const COLOR_RESET: &[u8] = b"\x1B[0m";

pub const DEFAULT_STACK_SIZE: usize = 24;
pub const DEFAULT_STRING_LENGTH: usize = 24;

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

macro_rules! add_func {
  ($wt:ident,$f:ident,$name:literal) => {
    //TODO: push $f into $wt with key derived from $name
  }
}
