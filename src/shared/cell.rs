use std::{
    fmt::Debug,
    sync::{Arc, Mutex, RwLock},
    thread::{self, Thread},
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
    pub fn new(position: Point3<f32>, volume: f32) -> Self {
        let events = Arc::new(EventSystem::new());
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
        events.subscribe(Box::new(move |event| {
            let cell_id = {
                let bio = bio.read().unwrap();
                bio.entity_id()
            };
            if event.id == cell_id {
                match event.event_type {
                    CellEventType::UpdatePosition(new_pos) => {
                        renderer.write().unwrap().position = new_pos;
                        let bio = bio.write().unwrap();
                        let mut pos = bio.position.write().unwrap();
                        *pos = new_pos;
                    }
                }
            }
        }));
    }
}

type Callback<E> = Box<dyn Fn(&E)>;

pub struct EventSystem {
    subscribers: Mutex<Vec<Callback<CellEvent>>>,
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
            subscribers: Mutex::new(Vec::new()),
        }
    }
    pub fn subscribe(&self, callback: Callback<CellEvent>) {
        self.subscribers.lock().unwrap().push(callback);
    }
    pub fn notify(&self, event: &CellEvent) {
        for callback in self.subscribers.lock().unwrap().iter() {
            callback(event);
        }
    }
}

pub struct CellEvent {
    /// The id of the cell that should be updated.
    pub id: u64,
    /// The type of the cell event and potentially data.
    pub event_type: CellEventType,
}

pub enum CellEventType {
    /// The position of a cell will be updated to the given f32.
    UpdatePosition(Point3<f32>),
}
