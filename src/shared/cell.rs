use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex, RwLock,
    },
    thread,
};

use crate::{
    engine::cell_renderer::{radius_from_volume, CellRenderer},
    model::{cell::BiologicalCell, entity::Entity},
};
use cgmath::{BaseFloat, Point3};

use super::plane::distance;

#[derive(Clone, Debug)]
pub struct CellInformation<T: BaseFloat> {
    pub id: u64,
    pub position: Point3<T>,
    pub radius: T,
}

impl From<BiologicalCell> for CellInformation<f32> {
    fn from(value: BiologicalCell) -> Self {
        Self {
            id: value.entity_id(),
            position: value.position().clone(),
            radius: radius_from_volume(&value.volume()),
        }
    }
}

impl From<CellRenderer> for CellInformation<f32> {
    fn from(value: CellRenderer) -> Self {
        Self {
            id: value.cell_id,
            position: value.position_clone(),
            radius: value.radius_clone(),
        }
    }
}

impl From<Cell> for CellInformation<f32> {
    fn from(value: Cell) -> Self {
        let renderer = value.renderer.read().unwrap().clone();
        renderer.into()
    }
}

#[derive(Clone, Debug)]
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
        Self {
            bio,
            renderer,
            events,
        }
    }
}

pub fn near(pos1: &Point3<f32>, radius1: f32, pos2: &Point3<f32>, radius2: f32) -> bool {
    let dist = distance(pos1, pos2);
    dist < radius1 + radius2
}

pub struct EventSystem {
    subscribers: Mutex<HashMap<u64, RwLock<Vec<Sender<Arc<CellEvent>>>>>>,
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
    /// The given id can be used to directly address all subscribers registered under this id.
    pub fn subscribe<F>(&self, id: u64, handler: F)
    where
        F: Fn(Arc<CellEvent>) + Send + 'static,
    {
        let (sender, receiver) = channel();
        {
            let mut subscribers = self.subscribers.lock().unwrap();
            let previous = subscribers.get(&id);
            if previous.is_none() {
                subscribers.insert(id, RwLock::new(vec![sender]));
            } else {
                let mut previous = previous.unwrap().write().unwrap();
                previous.push(sender);
            }
        }
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
            Option::Some(senders) => {
                let senders = senders.read().unwrap();
                senders.iter().for_each(|sender| {
                    let success = sender.send(Arc::clone(&event));
                    if success.is_err() {
                        println!(
                            "{:?} could not be sent! Error: {}",
                            event,
                            success.unwrap_err()
                        );
                    }
                });
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
    /// The volume of a cell will be updated to the given f32.
    UpdateVolume(f32),
}
