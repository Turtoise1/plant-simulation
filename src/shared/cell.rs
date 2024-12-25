use std::{
    fmt::Debug,
    sync::{Arc, Mutex, RwLock},
};

use crate::{engine::cell_renderer::CellRenderer, model::cell::BiologicalCell};

use super::point::Point3;

#[derive(Debug)]
pub struct Cell {
    pub bio: Arc<RwLock<BiologicalCell>>,
    pub renderer: Arc<RwLock<CellRenderer>>,
    events: Arc<EventSystem>,
}

impl Cell {
    pub fn new(position: Point3<f32>, volume: f32) -> Self {
        let events = Arc::new(EventSystem::new());
        let renderer = Arc::new(RwLock::new(CellRenderer::new(
            &position,
            &volume,
            Arc::clone(&events),
        )));
        let bio = Arc::new(RwLock::new(BiologicalCell::new(
            &position,
            volume,
            Arc::clone(&events),
        )));
        let bio_clone = Arc::clone(&bio);
        let renderer_clone = Arc::clone(&renderer);
        Cell::handle_events(Arc::clone(&events), bio_clone, renderer_clone);
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
        events.subscribe(Box::new(move |e| match e {
            CellEvent::FromBio(e) => match e {
                BiologicalCellEvent::UpdatePosition(pos) => {
                    renderer.write().unwrap().position.set(pos);
                }
            },
            CellEvent::FromRenderer(e) => match e {
                CellRendererEvent::UpdatePosition(pos) => {
                    bio.write().unwrap().position.write().unwrap().set(pos);
                }
            },
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

pub enum CellEvent {
    FromBio(BiologicalCellEvent),
    FromRenderer(CellRendererEvent),
}

pub enum BiologicalCellEvent {
    UpdatePosition(Point3<f32>),
}

pub enum CellRendererEvent {
    UpdatePosition(Point3<f32>),
}
