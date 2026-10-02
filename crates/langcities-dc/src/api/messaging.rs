use std::sync::Arc;

use langcities_messaging::common::sender::MessageSender;

use crate::message::event::{
    EntryFieldDeleteEvent, EntryFieldUpdateEvent, SoundChangeDeleteEvent, SoundChangeUpdateEvent,
};

#[derive(Clone)]
pub struct ApiMxs {
    pub efue_tx: Arc<dyn MessageSender<EntryFieldUpdateEvent>>,
    pub scue_tx: Arc<dyn MessageSender<SoundChangeUpdateEvent>>,
    pub efde_tx: Arc<dyn MessageSender<EntryFieldDeleteEvent>>,
    pub scde_tx: Arc<dyn MessageSender<SoundChangeDeleteEvent>>,
}
