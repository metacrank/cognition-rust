use cognition::*;
use std::any::Any;
use std::io::Write;
use std::process::Child;

pub struct ChildCustom { pub child: Child }

pub fn get_child_custom(pool: &mut Pool, child: Child) -> VCustom {
  get_from_custom_pool! (
    pool, "ChildCustoms", None, child_custom, ChildCustom,
    { child_custom.child = child },
    { Box::new(ChildCustom{ child }) }
  )
}

pub fn clear_pools(pool: &mut Pool) {
  pool.clear_custom_pool(custom_pool_name!("ChildCustomClones"));
  pool.clear_custom_pool(custom_pool_name!("ChildCustoms"));
}

#[custom(serde_as_void)]
impl Custom for ChildCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(child)");
  }
  fn copyfunc(&self, state: &mut CognitionState) -> Box<dyn Custom> {
    state.eval_error_mut("CHILD CANNOT BE DUPLICATED", None);
    Box::new(Void{})
  }
  fn custom_pool(&mut self, pool: &mut Pool) -> CustomPoolPackage {
    CustomPoolPackage::from(pool, custom_pool_name!("ChildCustoms"), None)
  }
}
