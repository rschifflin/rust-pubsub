use std::collections::HashMap;

pub struct Pubsub<'a, C:'a, P> {
  pub context: &'a mut C,
  listeners: HashMap<String, Vec<fn(context: &mut C, payload: P) -> Vec<Event<P>>>>,
  event_queue: Vec<Event<P>>
}

#[deriving(Clone)]
pub struct Event<P> {
  pub channel: String,
  pub payload: P
}

impl<'a, C, P: Clone> Pubsub<'a, C, P> {
  pub fn new(context: &mut C) -> Pubsub<C, P> {
    Pubsub {
      context: context,
      listeners: HashMap::new(),
      event_queue: Vec::new()
    }
  }

  pub fn publish(&mut self, event: Event<P>) {
    self.event_queue.push(event.clone());
    self.process_queue();
  }

  pub fn subscribe(&mut self, channel: String, listener: fn(&mut C, P) -> Vec<Event<P>>) {
    if !(Pubsub::try_existing(self.listeners.find_mut(&channel), listener)) {
      let mut v = Vec::new();
      v.push(listener);
      self.listeners.insert(channel, v);
    }
  }

  fn process_event(&mut self, event: Event<P>)  {
    let listeners_opt = self.listeners.find(&(event.channel));
    let ref mut context = self.context;

    match listeners_opt {
      Some(listeners) => for listener in listeners.iter() {
        self.event_queue = self.event_queue + (*listener)(*context, event.payload.clone());
      },
      None => ()
    }
  }

  fn process_queue(&mut self) {
    let event_opt = self.event_queue.pop();
    match event_opt {
      Some(event) => self.process_event(event),
      None => ()
    }
    if self.event_queue.len() > 0 { self.process_queue(); }
  }

  fn try_existing(existing: Option<&mut Vec<fn(&mut C, P) -> Vec<Event<P>>>>, listener: fn(&mut C, P) -> Vec<Event<P>>) -> bool {
    match existing {
      Some(existing_vec) => {
        existing_vec.push(listener);
        true
      }
      None => false
    }
  }
}

#[test]
fn no_listeners_should_not_change() {
  struct TestContext {
    data: int
  }

  let mut test_context = TestContext { data: 0 };
  let mut pubsub: Pubsub<TestContext, String> = Pubsub::new(&mut test_context);
  let event = Event {
    payload: "test payload".to_string(),
    channel: "test channel".to_string()
  };
  pubsub.publish(event);

  assert!(pubsub.context.data == 0)
}

#[test]
fn noop_listener_should_not_change() {
  struct TestContext {
    data: int
  }

  let mut test_context = TestContext { data: 0 };
  let mut pubsub: Pubsub<TestContext, String> = Pubsub::new(&mut test_context);
  let event = Event {
    payload: "test payload".to_string(),
    channel: "test channel".to_string()
  };

  fn noop_listener(context: &mut TestContext, msg: String) -> Vec<Event<String>> {
    Vec::new()
  }

  pubsub.subscribe("test channel".to_string(), noop_listener);
  pubsub.publish(event);

  assert!(pubsub.context.data == 0)
}

#[test]
fn listener_on_same_channel_should_change() {
  struct TestContext {
    data: int
  }

  let mut test_context = TestContext { data: 0 };
  let mut pubsub: Pubsub<TestContext, String> = Pubsub::new(&mut test_context);
  let event = Event {
    payload: "test payload".to_string(),
    channel: "test channel".to_string()
  };

  fn listener(context: &mut TestContext, msg: String) -> Vec<Event<String>> {
    context.data += 1;
    Vec::new()
  };

  pubsub.subscribe("test channel".to_string(), listener);
  pubsub.publish(event);

  assert!(pubsub.context.data == 1)
}

#[test]
fn listener_on_different_channel_should_not_change() {
  struct TestContext {
    data: int
  }

  let mut test_context = TestContext { data: 0 };
  let mut pubsub: Pubsub<TestContext, String> = Pubsub::new(&mut test_context);
  let event = Event {
    payload: "test payload".to_string(),
    channel: "test channel".to_string()
  };

  fn listener(context: &mut TestContext, msg: String) -> Vec<Event<String>> {
    context.data += 1;
    Vec::new()
  };

  pubsub.subscribe("diff channel".to_string(), listener);
  pubsub.publish(event);

  assert!(pubsub.context.data == 0)
}

#[test]
fn listener_can_trigger_more_events() {
  struct TestContext {
    data: int
  }
  let mut test_context = TestContext { data: 0 };
  let mut pubsub: Pubsub<TestContext, String> = Pubsub::new(&mut test_context);
  let event = Event {
    payload: "test payload".to_string(),
    channel: "test channel".to_string()
  };

  fn listener_with_triggers(context: &mut TestContext, msg: String) -> Vec<Event<String>> {
    context.data += 1;
    Vec::from_elem(1, Event {
        channel: "test channel 2".to_string(),
        payload: "payload".to_string()
    })
  };

  fn plain_listener(context: &mut TestContext, msg: String) -> Vec<Event<String>> {
    context.data += 1;
    Vec::new()
  }

  pubsub.subscribe("test channel".to_string(), listener_with_triggers);
  pubsub.subscribe("test channel 2".to_string(), plain_listener);
  pubsub.publish(event);

  assert!(pubsub.context.data == 2)
}
