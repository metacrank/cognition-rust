use cognition::*;
use std::any::Any;
use std::io::Write;
pub use std::time::{Duration, Instant, SystemTime};

pub const DURATION_CUSTOM_POOL: &str = custom_pool_name!("DurationCustoms");
pub const INSTANT_CUSTOM_POOL: &str = custom_pool_name!("InstantCustoms");
pub const SYSTEM_TIME_CUSTOM_POOL: &str = custom_pool_name!("SystemTimeCustoms");

#[derive(Serialize, Deserialize)]
pub struct DurationCustom { pub duration: Duration, pub neg: bool }
pub struct InstantCustom { pub instant: Instant }
pub struct SystemTimeCustom { pub time: SystemTime }

pub fn get_duration_custom(pool: &mut Pool, duration: Duration, neg: bool) -> VCustom {
  get_from_custom_pool! (
    pool, "DurationCustoms", None, duration_custom, DurationCustom,
    { duration_custom.duration = duration; duration_custom.neg = neg },
    { Box::new(DurationCustom{ duration, neg }) }
  )
}

pub fn get_instant_custom(pool: &mut Pool, instant: Instant) -> VCustom {
  get_from_custom_pool! (
    pool, "InstantCustoms", None, instant_custom, InstantCustom,
    { instant_custom.instant = instant },
    { Box::new(InstantCustom{ instant }) }
  )
}

pub fn get_system_time_custom(pool: &mut Pool, time: SystemTime) -> VCustom {
  get_from_custom_pool! (
    pool, "SystemTimeCustoms", None, time_custom, SystemTimeCustom,
    { time_custom.time = time },
    { Box::new(SystemTimeCustom{ time }) }
  )
}

pub fn clear_pools(pool: &mut Pool) {
  pool.clear_custom_pool(DURATION_CUSTOM_POOL);
  pool.clear_custom_pool(INSTANT_CUSTOM_POOL);
  pool.clear_custom_pool(SYSTEM_TIME_CUSTOM_POOL);
}

#[cognition::custom]
impl Custom for DurationCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(duration)");
  }
  fn copyfunc(&self, state: &mut CognitionState) -> Box<dyn Custom> {
    get_duration_custom(&mut state.pool, self.duration, self.neg).custom
  }
  fn custom_pool(&mut self, pool: &mut Pool) -> CustomPoolPackage {
    CustomPoolPackage::from(pool, DURATION_CUSTOM_POOL, None)
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
    CustomPoolPackage::from(pool, INSTANT_CUSTOM_POOL, None)
  }
}

#[cognition::custom(serde_as_void)]
impl Custom for SystemTimeCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(system time)");
  }
  fn copyfunc(&self, state: &mut CognitionState) -> Box<dyn Custom> {
    get_system_time_custom(&mut state.pool, self.time).custom
  }
  fn custom_pool(&mut self, pool: &mut Pool) -> CustomPoolPackage {
    CustomPoolPackage::from(pool, SYSTEM_TIME_CUSTOM_POOL, None)
  }
}
