use std::{collections::VecDeque, sync::RwLock};

use edsdk_sys::{EdsBaseRef, EdsDirectoryItemRef, EdsObjectEvent, EdsVoid};

pub enum Event {
    DirItemCreated(EdsDirectoryItemRef),
}

pub(crate) unsafe extern "C" fn object_event_handler(
    event: EdsObjectEvent,
    object: EdsBaseRef,
    context: *mut EdsVoid,
) -> u32 {
    let save_to_queue = |event| {
        let events_queue = context as *const RwLock<VecDeque<Event>>;
        if let Some(mutex) = events_queue.as_ref() {
            if let Ok(mut guard) = mutex.write() {
                guard.push_front(event);
            } else {
                eprintln!("mutex cannot be locked");
            }
        } else {
            eprintln!("mutex cannot be referenced");
        }
    };
    #[allow(clippy::single_match)]
    #[allow(non_upper_case_globals)]
    match event {
        kEdsObjectEvent_DirItemCreated => {
            let item_ref = Event::DirItemCreated(object);
            save_to_queue(item_ref);
        }
        kEdsObjectEvent_VolumeInfoChanged => {}
        _ => println!("unknown event emitted"),
        /*
        kEdsObjectEvent_All => todo!(),
        kEdsObjectEvent_VolumeInfoChanged => todo!(),
        kEdsObjectEvent_VolumeUpdateItems => todo!(),
        kEdsObjectEvent_FolderUpdateItems => todo!(),
        kEdsObjectEvent_DirItemRemoved => todo!(),
        kEdsObjectEvent_DirItemInfoChanged => todo!(),
        kEdsObjectEvent_DirItemContentChanged => todo!(),
        kEdsObjectEvent_DirItemRequestTransfer => todo!(),
        kEdsObjectEvent_DirItemRequestTransferDT => todo!(),
        kEdsObjectEvent_DirItemCancelTransferDT => todo!(),
        kEdsObjectEvent_VolumeAdded => todo!(),
        kEdsObjectEvent_VolumeRemoved => todo!(),
        */
    }
    println!("Okey: {} [{:?}]", event, object);
    0
}

pub(crate) unsafe extern "C" fn property_event_handler(
    event: u32,
    property_id: u32,
    something_else: u32,
    context: *mut EdsVoid,
) -> u32 {
    println!("Okey prop");
    0
}
