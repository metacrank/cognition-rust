#![allow(unused_imports)]
use cognition::*;
use unicode_segmentation::Graphemes;
use unicode_segmentation::UnicodeSegmentation;

pub fn cog_gcut(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

pub fn cog_gunconcat(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

pub fn cog_gcat(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

pub fn cog_glen(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

pub fn cog_ginsert(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

pub fn cog_greverse(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

pub fn cog_gtoi(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

pub fn cog_itog(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

#[no_mangle]
pub extern fn add_words(state: &mut CognitionState) {
  add_word!(state, "gcut", cog_gcut);
  // add_word!(state, "gunconcat", cog_gunconcat);
  // add_func!(wt, cog_gcat, "gcat");
  // add_func!(wt, cog_glen, "glen");
  // add_func!(wt, cog_ginsert, "ginsert");
  // add_func!(wt, cog_greverse, "greverse");
  // add_func!(wt, cog_gtoi, "gtoi");
  // add_func!(wt, cog_itog, "itog");
}
