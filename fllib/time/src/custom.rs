use cognition::*;
use std::any::Any;
use std::io::Write;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize)]
pub struct DurationCustom { pub duration: Duration }
pub struct InstantCustom { pub instant: Instant }

pub fn get_duration_custom(pool: &mut Pool, duration: Duration) -> VCustom {
  get_from_custom_pool! (
    pool, "DurationCustoms", None, duration_custom, DurationCustom,
    { duration_custom.duration = duration },
    { VCustom::with_custom(Box::new(DurationCustom{ duration })) }
  )
}

pub fn get_instant_custom(pool: &mut Pool, instant: Instant) -> VCustom {
  get_from_custom_pool! (
    pool, "InstantCustoms", None, instant_custom, InstantCustom,
    { instant_custom.instant = instant },
    { VCustom::with_custom(Box::new(InstantCustom{ instant })) }
  )
}

pub fn clear_pools(pool: &mut Pool) {
  pool.clear_custom_pool(custom_pool_name!("DurationCustoms"));
  pool.clear_custom_pool(custom_pool_name!("InstantCustoms"));
}

#[cognition::custom]
impl Custom for DurationCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(duration)");
  }
  fn copyfunc(&self, state: &mut CognitionState) -> Box<dyn Custom> {
    get_duration_custom(&mut state.pool, self.duration).custom
  }
  fn custom_pool(&mut self, pool: &mut Pool) -> CustomPoolPackage {
    CustomPoolPackage::from(pool, custom_pool_name!("DurationCustoms"), None)
  }
}

#[cognition::custom(serde_as_void)]
impl Custom for InstantCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(instant)");
  }
  fn copyfunc(&self, state: &mut CognitionState) -> Box<dyn Custom> {
    get_instant_custom(&mut state.pool, self.instant).custom
  }
  fn custom_pool(&mut self, pool: &mut Pool) -> CustomPoolPackage {
    CustomPoolPackage::from(pool, custom_pool_name!("InstantCustoms"), None)
  }
}
