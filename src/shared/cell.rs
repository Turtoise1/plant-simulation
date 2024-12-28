use std::{
    collections::HashMap,
    fmt::Debug,
    io::Write,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex, RwLock,
    },
    thread,
};

use crate::{
    engine::cell_renderer::CellRenderer,
    model::{cell::BiologicalCell, entity::Entity},
};
use cgmath::Point3;

#[derive(Debug)]
pub struct Cell {
    pub bio: Arc<RwLock<BiologicalCell>>,
    pub renderer: Arc<RwLock<CellRenderer>>,
    events: Arc<EventSystem>,
}

impl Cell {
    pub fn new(position: Point3<f32>, volume: f32, events: Arc<EventSystem>) -> Self {
        let bio = Arc::new(RwLock::new(BiologicalCell::new(
            &position,
            volume,
            Arc::clone(&events),
        )));
        let renderer = Arc::new(RwLock::new(CellRenderer::new(
            &position,
            &volume,
            bio.read().unwrap().entity_id(),
            Arc::clone(&events),
        )));
        let bio_clone = Arc::clone(&bio);
        let renderer_clone = Arc::clone(&renderer);
        let events_clone = Arc::clone(&events);
        Cell::handle_events(events_clone, bio_clone, renderer_clone);
        Self {
            bio,
            renderer,
            events,
        }
    }

    fn handle_events(
        events: Arc<EventSystem>,
        bio: Arc<RwLock<BiologicalCell>>,
        renderer: Arc<RwLock<CellRenderer>>,
    ) {
        let cell_id = {
            let bio = bio.read().unwrap();
            bio.entity_id()
        };
        events.subscribe(cell_id, move |event| {
            if event.id == cell_id {
                match event.event_type {
                    CellEventType::UpdatePosition(new_pos) => {
                        renderer.write().unwrap().position = new_pos;
                        let bio = bio.write().unwrap();
                        let mut pos = bio.position.write().unwrap();
                        *pos = new_pos;
                    }
                }
            };
        });
    }
}

pub struct EventSystem {
    subscribers: Mutex<HashMap<u64, Sender<Arc<CellEvent>>>>,
}

impl Debug for EventSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "event system with {} subscribers",
            self.subscribers.lock().unwrap().len()
        )
    }
}

impl EventSystem {
    pub fn new() -> Self {
        EventSystem {
            subscribers: Mutex::new(HashMap::new()),
        }
    }

    /// subscribe to cell events to handle them.
    /// The given id can be used to directly address this subscriber with a notification.
    pub fn subscribe<F>(&self, id: u64, handler: F)
    where
        F: Fn(Arc<CellEvent>) + Send + 'static,
    {
        let (sender, receiver) = channel();
        self.subscribers.lock().unwrap().insert(id, sender);
        thread::spawn(move || {
            for event in receiver {
                handler(event);
            }
        });
    }

    /// notifies the cell specified by the id given in the event
    pub fn notify(&self, event: Arc<CellEvent>) {
        let subscribers = self.subscribers.lock().unwrap();
        let sender = subscribers.get(&event.id);
        match sender {
            Option::Some(sender) => {
                let success = sender.send(Arc::clone(&event));
                if success.is_err() {
                    println!(
                        "{:?} could not be sent! Error: {}",
                        event,
                        success.unwrap_err()
                    );
                }
            }
            None => {
                println!(
                    "In this event system, no subscriber with id {} could be found!",
                    event.id,
                );
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CellEvent {
    /// The id of the cell that should be updated.
    pub id: u64,
    /// The type of the cell event and potentially data.
    pub event_type: CellEventType,
}

#[derive(Clone, Debug)]
pub enum CellEventType {
    /// The position of a cell will be updated to the given f32.
    UpdatePosition(Point3<f32>),
}
