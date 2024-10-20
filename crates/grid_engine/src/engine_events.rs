use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use crate::grid_view::GridView;

pub trait EventNameTrait: Eq + PartialEq + Hash + Debug {}
impl<T: Eq + PartialEq + Hash + Debug> EventNameTrait for T {}

pub trait EventValueTrait: Eq + Hash {}
impl<T: Eq + Hash> EventValueTrait for T {}

type Function<EventValue> = dyn FnMut(&GridView, &EventValue) -> () + Send + 'static + Sync;

struct ListenerFunction<Event: EventValueTrait> {
    id: String,
    function: Box<Function<Event>>,
}

impl<EventValue: EventValueTrait> Debug for ListenerFunction<EventValue> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListenerFunction")
            .field("id", &self.id)
            .finish()
    }
}

#[derive(Debug)]
pub struct EventListener<EventName: EventNameTrait, EventValue: EventValueTrait> {
    listeners: HashMap<EventName, Vec<ListenerFunction<EventValue>>>,
}

impl<EventName: EventNameTrait, EventValue: EventValueTrait> Default
    for EventListener<EventName, EventValue>
{
    fn default() -> Self {
        EventListener {
            listeners: HashMap::new(),
        }
    }
}

impl<EventName: EventNameTrait, EventValue: EventValueTrait> EventListener<EventName, EventValue> {
    pub fn add_listener(
        &mut self,
        event: EventName,
        function: Box<Function<EventValue>>,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let listener = ListenerFunction {
            id: id.clone(),
            function,
        };
        self.listeners
            .entry(event)
            .or_insert_with(Vec::new)
            .push(listener);
        id
    }

    pub fn remove_listener(&mut self, event: EventName, id: &str) {
        if let Some(listeners) = self.listeners.get_mut(&event) {
            listeners.retain(|listener| listener.id != id);

            if listeners.is_empty() {
                self.listeners.remove(&event);
            }
        }
    }

    pub fn trigger_event(
        &mut self,
        grid: &GridView,
        event_name: EventName,
        event_value: EventValue,
    ) {
        if let Some(listeners) = self.listeners.get_mut(&event_name) {
            for listener in listeners {
                (listener.function)(&grid, &event_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, sync::Mutex};

    use crate::grid_engine::GridEngine;

    use super::*;

    #[test]
    fn test_add_listener() {
        let mut event_listener: EventListener<String, i32> = EventListener::default();
        let event_name = "event1".to_string();
        let listener_id = event_listener.add_listener(
            event_name.clone(),
            Box::new(|_, value| {
                println!("Event1 triggered with value: {}", value);
            }),
        );

        // Should add the listener to the event
        assert_eq!(event_listener.listeners.len(), 1);
        assert_eq!(event_listener.listeners.get(&event_name).unwrap().len(), 1);
        assert_eq!(
            event_listener.listeners.get(&event_name).unwrap()[0].id,
            listener_id
        );

        let listener_2_id = event_listener.add_listener(
            event_name.clone(),
            Box::new(|_, value| {
                println!("Event1, listener 2, triggered with value: {}", value);
            }),
        );

        // Add listener to the same event
        assert_eq!(event_listener.listeners.len(), 1);
        let event_1_listeners = event_listener.listeners.get(&event_name).unwrap();
        assert_eq!(event_1_listeners.len(), 2);
        // Asserts that they are inserted in order
        assert_eq!(&event_1_listeners[1].id, &listener_2_id);

        let event_name2 = "event2".to_string();
        event_listener.add_listener(
            event_name2.clone(),
            Box::new(|_, value| {
                println!("Event2 triggered with value: {}", value);
            }),
        );

        // adding a listener to a new event don't affect other events
        assert_eq!(event_listener.listeners.len(), 2);
        let event_1_listeners_again = event_listener.listeners.get(&event_name).unwrap();
        assert_eq!(event_1_listeners_again.len(), 2);

        // new event listener successful added
        let event_2_listeners = event_listener.listeners.get(&event_name2).unwrap();
        assert_eq!(event_2_listeners.len(), 1);
    }

    #[test]
    fn test_remove_listener() {
        let mut event_listener: EventListener<String, i32> = EventListener::default();
        let event_name = "event1".to_string();
        let listener_id = event_listener.add_listener(
            event_name.clone(),
            Box::new(|_, value| {
                println!("Event1 triggered with value: {}", value);
            }),
        );
        assert_eq!(event_listener.listeners.len(), 1);
        event_listener.remove_listener(event_name.clone(), &listener_id);
        assert_eq!(event_listener.listeners.len(), 0);
    }

    #[test]
    fn test_trigger_event() {
        let value = Arc::new(Mutex::new(0));
        let mut event_listener: EventListener<String, i32> = EventListener::default();
        let add_event = "add".to_string();

        let grid_view = GridEngine::new(10, 10).get_grid_view();

        let value_clone = value.clone();
        event_listener.add_listener(
            add_event.clone(),
            Box::new(move |_, v| {
                *value_clone.lock().unwrap() += v;
                // *value_clone.borrow_mut() += v;
            }),
        );

        event_listener.trigger_event(&grid_view, add_event.clone(), 1);

        assert_eq!(*value.lock().unwrap(), 1);

        let value_clone = value.clone();
        event_listener.add_listener(
            add_event.clone(),
            Box::new(move |_, v| {
                *value_clone.lock().unwrap() += v;
            }),
        );
        event_listener.trigger_event(&grid_view, add_event.clone(), 2);
        assert_eq!(*value.lock().unwrap(), 5);

        let subtract_event = "subtract".to_string();
        let value_clone = value.clone();
        event_listener.add_listener(
            subtract_event.clone(),
            Box::new(move |_, v| {
                *value_clone.lock().unwrap() -= v;
            }),
        );
        event_listener.trigger_event(&grid_view, subtract_event.clone(), 5);
        assert_eq!(*value.lock().unwrap(), 0);
    }
}
