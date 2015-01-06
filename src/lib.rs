use std::hash::Hash;
use std::collections::hash_map::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

#[derive(Clone)]
pub struct Event<Channel, Payload> {
  pub channel: Channel,
  pub payload: Payload
}

pub struct Pubsub<'a, Context:'a, Channel: Hash + Eq + Clone, Payload: Clone> {
  pub context: &'a mut Context,
  listeners: HashMap<Channel, Vec<fn(context: &mut Context, payload: Payload) -> Vec<Event<Channel, Payload>>>>,
  event_queue: Vec<Event<Channel, Payload>>
}

impl<'a, Context, Channel: Hash + Eq + Clone, Payload: Clone> Pubsub<'a, Context, Channel, Payload> {
  pub fn new(context: &mut Context) -> Pubsub<Context, Channel, Payload> {
    Pubsub {
      context: context,
      listeners: HashMap::new(),
      event_queue: Vec::new()
    }
  }

  pub fn publish(&mut self, event: Event<Channel, Payload>) {
    self.event_queue.push(event.clone());
    self.process_queue();
  }

  pub fn subscribe(&mut self, channel: Channel, listener: fn(&mut Context, Payload) -> Vec<Event<Channel, Payload>>) {
    if !(Pubsub::try_existing(self.listeners.get_mut(&channel), listener)) {
      let mut v = Vec::new();
      v.push(listener);
      self.listeners.insert(channel, v);
    }
  }

  fn process_event(&mut self, event: Event<Channel, Payload>)  {
    let listeners_entry = self.listeners.entry(&event.channel);
    let ref mut context = self.context;

    match listeners_entry {
      Occupied(mut listeners) => for listener in listeners.get_mut().iter() {
        let head = self.event_queue.clone();
        let tail = (*listener)(*context, event.payload.clone());
        self.event_queue = head + tail.as_slice();
      },
      Vacant(_) => ()
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

  fn try_existing(existing: Option<&mut Vec<fn(&mut Context, Payload) -> Vec<Event<Channel, Payload>>>>, listener: fn(&mut Context, Payload) -> Vec<Event<Channel, Payload>>) -> bool {
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
  let mut pubsub: Pubsub<TestContext, String, String> = Pubsub::new(&mut test_context);
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
  let mut pubsub: Pubsub<TestContext, String, String> = Pubsub::new(&mut test_context);
  let event = Event {
    payload: "test payload".to_string(),
    channel: "test channel".to_string()
  };

  fn noop_listener(context: &mut TestContext, msg: String) -> Vec<Event<String, String>> {
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
  let mut pubsub: Pubsub<TestContext, String, String> = Pubsub::new(&mut test_context);
  let event = Event {
    payload: "test payload".to_string(),
    channel: "test channel".to_string()
  };

  fn listener(context: &mut TestContext, msg: String) -> Vec<Event<String, String>> {
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
  let mut pubsub: Pubsub<TestContext, String, String> = Pubsub::new(&mut test_context);
  let event = Event {
    payload: "test payload".to_string(),
    channel: "test channel".to_string()
  };

  fn listener(context: &mut TestContext, msg: String) -> Vec<Event<String, String>> {
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
  let mut pubsub: Pubsub<TestContext, String, String> = Pubsub::new(&mut test_context);
  let event = Event {
    payload: "test payload".to_string(),
    channel: "test channel".to_string()
  };

  fn listener_with_triggers(context: &mut TestContext, msg: String) -> Vec<Event<String, String>> {
    context.data += 1;
    vec![Event {
        channel: "test channel 2".to_string(),
        payload: "payload".to_string()
    }]
  };

  fn plain_listener(context: &mut TestContext, msg: String) -> Vec<Event<String, String>> {
    context.data += 1;
    Vec::new()
  }

  pubsub.subscribe("test channel".to_string(), listener_with_triggers);
  pubsub.subscribe("test channel 2".to_string(), plain_listener);
  pubsub.publish(event);

  assert!(pubsub.context.data == 2)
}
