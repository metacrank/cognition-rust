use cognition::*;
use std::any::Any;
use std::io::Write;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};

pub struct CogStateWrapper(pub CognitionState);
unsafe impl Send for CogStateWrapper {}
pub struct ValueWrapper(pub Value);
unsafe impl Send for ValueWrapper {}

type OptionHandle = Option<thread::JoinHandle<CogStateWrapper>>;
pub type ThreadCustomHandle = Option<Arc<Mutex<OptionHandle>>>;
pub type SharedCustomValue = Option<Arc<Mutex<Option<Value>>>>;

pub struct ThreadCustom { pub handle: ThreadCustomHandle }
pub struct SendCustom { pub tx: Option<Sender<ValueWrapper>> }
pub struct RecvCustom { pub rx: Option<Receiver<ValueWrapper>> }
pub struct SharedCustom { pub value: SharedCustomValue }

pub fn get_thread_custom_clone(pool: &mut Pool, handle: ThreadCustomHandle) -> VCustom {
  get_from_custom_pool! (
    pool, "ThreadCustomClones", None, thread_custom, ThreadCustom,
    { thread_custom.handle = handle },
    { Box::new(ThreadCustom{ handle }) }
  )
}

pub fn get_thread_custom(pool: &mut Pool, option: OptionHandle) -> VCustom {
  get_from_custom_pool! (
    pool, "ThreadCustoms", None, thread_custom, ThreadCustom,
    {
      const ERROR: &str = "null or non-unique thread custom in custom pool";
      let arc = thread_custom.handle.as_mut().expect(ERROR);
      let mutex = Arc::<Mutex<OptionHandle>>::get_mut(arc).expect(ERROR);
      mutex.clear_poison();
      *mutex.get_mut().unwrap() = option
    },
    {
      let handle = Some(Arc::new(Mutex::new(option)));
      Box::new(ThreadCustom{ handle })
    }
  )
}

pub fn get_send_custom(pool: &mut Pool, tx: Option<Sender<ValueWrapper>>) -> VCustom {
  get_from_custom_pool! (
    pool, "SendCustoms", None, send_custom, SendCustom,
    { send_custom.tx = tx },
    { Box::new(SendCustom{ tx }) }
  )
}

pub fn get_recv_custom(pool: &mut Pool, rx: Option<Receiver<ValueWrapper>>) -> VCustom {
  get_from_custom_pool! (
    pool, "RecvCustoms", None, recv_custom, RecvCustom,
    { recv_custom.rx = rx },
    { Box::new(RecvCustom{ rx }) }
  )
}

pub fn get_shared_custom_clone(pool: &mut Pool, value: SharedCustomValue) -> VCustom {
  get_from_custom_pool! (
    pool, "SharedCustomClones", None, shared_custom, SharedCustom,
    { shared_custom.value = value },
    { Box::new(SharedCustom{ value }) }
  )
}

pub fn get_shared_custom(pool: &mut Pool, option: Option<Value>) -> VCustom {
  get_from_custom_pool! (
    pool, "SharedCustoms", None, shared_custom, SharedCustom,
    {
      const ERROR: &str = "null or non-unique shared custom in custom pool";
      let arc = shared_custom.value.as_mut().expect(ERROR);
      let mutex = Arc::<Mutex<Option<Value>>>::get_mut(arc).expect(ERROR);
      mutex.clear_poison();
      *mutex.get_mut().unwrap() = option
    },
    {
      let value = Some(Arc::new(Mutex::new(option)));
      Box::new(SharedCustom{ value })
    }
  )
}

pub fn clear_pools(pool: &mut Pool) {
  pool.clear_custom_pool(custom_pool_name!("ThreadCustoms"));
  pool.clear_custom_pool(custom_pool_name!("ThreadCustomClones"));
  pool.clear_custom_pool(custom_pool_name!("SendCustoms"));
  pool.clear_custom_pool(custom_pool_name!("RecvCustoms"));
  pool.clear_custom_pool(custom_pool_name!("SharedCustoms"));
  pool.clear_custom_pool(custom_pool_name!("SharedCustomClones"));
}

#[custom(serde_as_void)]
impl Custom for ThreadCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    match self.handle.as_ref().map_or(None, |h| Some(h.lock())) {
      Some(Ok(handle)) => if handle.is_some() {
        fwrite_check!(f, b"(thread)");
      } else {
        fwrite_check!(f, b"(null thread)");
      },
      Some(Err(_)) => {
        fwrite_check!(f, b"(poisoned thread)");
      },
      None => {
        fwrite_check!(f, b"(uninitialized thread)");
      }
    }
  }
  fn copyfunc(&self, state: &mut CognitionState) -> Box<dyn Custom> {
    get_thread_custom_clone(&mut state.pool, self.handle.clone()).custom
  }
  fn custom_pool(&mut self, pool: &mut Pool) -> CustomPoolPackage {
    if let Some(ref arc) = self.handle {
      if Arc::<Mutex<OptionHandle>>::strong_count(arc) == 1 {
        drop(arc.lock().map_or(None, |mut v| v.take()));
        return CustomPoolPackage::from(pool, custom_pool_name!("ThreadCustoms"), None)
      }
    }
    drop(self.handle.take());
    CustomPoolPackage::from(pool, custom_pool_name!("ThreadCustomClones"), None)
  }
}

#[custom(serde_as_void)]
impl Custom for SendCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(sender)");
  }
  fn copyfunc(&self, state: &mut CognitionState) -> Box<dyn Custom> {
    get_send_custom(&mut state.pool, self.tx.clone()).custom
  }
  fn custom_pool(&mut self, pool: &mut Pool) -> CustomPoolPackage {
    drop(self.tx.take());
    CustomPoolPackage::from(pool, custom_pool_name!("SendCustoms"), None)
  }
}

#[custom(serde_as_void)]
impl Custom for RecvCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    fwrite_check!(f, b"(receiver)");
  }
  fn copyfunc(&self, state: &mut CognitionState) -> Box<dyn Custom> {
    state.eval_error_mut("RECEIVER CANNOT BE DUPLICATED", None);
    Box::new(Void{})
  }
  fn custom_pool(&mut self, pool: &mut Pool) -> CustomPoolPackage {
    drop(self.rx.take());
    CustomPoolPackage::from(pool, custom_pool_name!("RecvCustoms"), None)
  }
}

impl Serialize for SharedCustom {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer {
    if let Some(ref value) = self.value {
      let lock = value.lock();
      match lock {
        Ok(ok) => serializer.serialize_some(&*ok),
        Err(_) => serializer.serialize_some(&None::<Value>)
      }
    } else {
      serializer.serialize_none()
    }
  }
}

impl<'de> CognitionDeserialize<'de> for SharedCustom {
  fn cognition_deserialize<D>(deserializer: D, state: &mut CognitionState) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
    Self: Sized,
  {
    let v = Option::<Option<Value>>::cognition_deserialize(deserializer, state)?;
    match v {
      Some(v) => {
        let mut vcustom = get_shared_custom(&mut state.pool, v);
        let shared_custom = vcustom.custom.as_any_mut().downcast_mut::<SharedCustom>().unwrap();
        let mut new_shared_custom = SharedCustom{ value: None };
        std::mem::swap(shared_custom, &mut new_shared_custom);
        state.pool.add_vcustom(vcustom);
        Ok(new_shared_custom)
      },
      None => Ok(SharedCustom{ value: None })
    }
  }
}

#[custom(cognition_serde)]
impl Custom for SharedCustom {
  fn printfunc(&self, f: &mut dyn Write) {
    match self.value.as_ref().map_or(None, |v| Some(v.lock())) {
      Some(Ok(value)) => if value.is_some() {
        fwrite_check!(f, b"(shared)");
      } else {
        fwrite_check!(f, b"(null shared)");
      },
      Some(Err(_)) => {
        fwrite_check!(f, b"(poisoned shared)");
      },
      None => {
        fwrite_check!(f, b"(uninitialized shared)");
      }
    }
  }
  fn copyfunc(&self, state: &mut CognitionState) -> Box<dyn Custom> {
    get_shared_custom_clone(&mut state.pool, self.value.clone()).custom
  }
  fn custom_pool(&mut self, pool: &mut Pool) -> CustomPoolPackage {
    if let Some(ref arc) = self.value {
      if Arc::<Mutex<Option<Value>>>::strong_count(&arc) == 1 {
        if let Some(v) = arc.lock().map_or(None, |mut v| v.take()) {
          pool.add_val(v);
        }
        return CustomPoolPackage::from(pool, custom_pool_name!("SharedCustoms"), None)
      }
    }
    drop(self.value.take());
    CustomPoolPackage::from(pool, custom_pool_name!("SharedCustomClones"), None)
  }
}
