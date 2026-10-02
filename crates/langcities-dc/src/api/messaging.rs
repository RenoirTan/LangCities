use std::sync::Arc;

use langcities_messaging::common::{receiver::MessageReceiver, sender::MessageSender};

use crate::message::event::{
    EntryFieldDeleteEvent, EntryFieldUpdateEvent, SoundChangeDeleteEvent, SoundChangeUpdateEvent,
};

// Only for testing purposes
#[allow(unused)]
pub(crate) struct DummyApiRxs {
    pub efue_rx: Box<dyn MessageReceiver<EntryFieldUpdateEvent>>,
    pub scue_rx: Box<dyn MessageReceiver<SoundChangeUpdateEvent>>,
    pub efde_rx: Box<dyn MessageReceiver<EntryFieldDeleteEvent>>,
    pub scde_rx: Box<dyn MessageReceiver<SoundChangeDeleteEvent>>,
}

#[derive(Clone)]
pub struct ApiMxs {
    pub efue_tx: Arc<dyn MessageSender<EntryFieldUpdateEvent>>,
    pub scue_tx: Arc<dyn MessageSender<SoundChangeUpdateEvent>>,
    pub efde_tx: Arc<dyn MessageSender<EntryFieldDeleteEvent>>,
    pub scde_tx: Arc<dyn MessageSender<SoundChangeDeleteEvent>>,
}
