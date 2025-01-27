use std::{
    collections::VecDeque,
    ffi::c_void,
    ptr::null_mut,
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::anyhow;
use edsdk_sys::{
    kEdsCameraCommand_PressShutterButton, kEdsObjectEvent_All, kEdsPropID_SaveTo, tagEdsDeviceInfo,
    EdsCameraRef, EdsCloseSession, EdsGetChildAtIndex, EdsGetDeviceInfo, EdsOpenSession,
    EdsRelease, EdsSendCommand, EdsSetObjectEventHandler, EdsSetPropertyData,
    EdsSetPropertyEventHandler, EdsShutterButton_kEdsCameraCommand_ShutterButton_Completely_NonAF,
    EdsShutterButton_kEdsCameraCommand_ShutterButton_OFF, EDS_ERR_OK,
};

use crate::{
    events::{object_event_handler, property_event_handler, Event},
    image::Image,
    sdk::Sdk,
};

pub struct Camera<'a> {
    id: u32,
    sdk: &'a Sdk,
    camera_ref: EdsCameraRef,
    events: Arc<RwLock<VecDeque<Event>>>,
}

impl Drop for Camera<'_> {
    fn drop(&mut self) {
        let err = unsafe { EdsCloseSession(self.camera_ref) };
        if err != EDS_ERR_OK {
            eprintln!("failed to close camera session: {err:?}")
        }
        let err = unsafe { EdsRelease(self.camera_ref) };
        if err != EDS_ERR_OK {
            eprintln!("failed to release camera: {err:?}")
        }
        println!("camera also dropped");
    }
}

impl<'a> Camera<'a> {
    pub(crate) fn init(sdk: &'a Sdk, camera_id: u32) -> anyhow::Result<Self> {
        let mut camera = null_mut();
        let err = unsafe { EdsGetChildAtIndex(sdk.camera_list, camera_id as i32, &mut camera) };
        if err != EDS_ERR_OK {
            Err(anyhow!(
                "failed to get camera id={}: result: {err:?}",
                camera_id
            ))?
        }
        let err = unsafe { EdsOpenSession(camera) };
        if err != EDS_ERR_OK {
            Err(anyhow!(
                "failed to open camera session id={}: result: {err:?}",
                camera_id
            ))?
        }
        let events = Arc::new(RwLock::new(VecDeque::<Event>::new()));
        let ctx = Arc::as_ptr(&events);
        let ctx = ctx as *mut c_void;
        let err = unsafe {
            EdsSetObjectEventHandler(camera, kEdsObjectEvent_All, Some(object_event_handler), ctx)
        };
        if err != EDS_ERR_OK {
            Err(anyhow!("failed to set up object event handler",))?
        }
        let err = unsafe {
            EdsSetPropertyEventHandler(
                camera,
                kEdsObjectEvent_All,
                Some(property_event_handler),
                ctx,
            )
        };
        if err != EDS_ERR_OK {
            Err(anyhow!("failed to set up object event handler",))?
        }
        Ok(Self {
            id: camera_id,
            sdk,
            camera_ref: camera,
            events,
        })
    }
    pub fn take_shot(&self) -> anyhow::Result<Image> {
        let err = unsafe {
            EdsSendCommand(
                self.camera_ref,
                kEdsCameraCommand_PressShutterButton,
                EdsShutterButton_kEdsCameraCommand_ShutterButton_Completely_NonAF as i32,
            )
        };
        if err != EDS_ERR_OK {
            Err(anyhow!("Failed to press shutter "))?
        }

        let err = unsafe {
            EdsSendCommand(
                self.camera_ref,
                kEdsCameraCommand_PressShutterButton,
                EdsShutterButton_kEdsCameraCommand_ShutterButton_OFF as i32,
            )
        };
        if err != EDS_ERR_OK {
            Err(anyhow!("Failed to unpress shutter "))?
        }

        let mut attempts = 100;

        while attempts != 0 {
            attempts -= 1;
            if let Ok(mut guard) = self.events.write() {
                if let Some(ix) = guard
                    .iter()
                    .position(|event| matches!(event, Event::DirItemCreated(..)))
                {
                    println!("wait for image....");
                    if let Some(Event::DirItemCreated(reference)) = guard.remove(ix) {
                        return Ok(Image {
                            sdk: self.sdk,
                            reference,
                        });
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        Err(anyhow!("Directory item creation timeout"))
    }

    pub fn set_flash(&self) -> &Self {
        unsafe {
            let mut value = 1;
            let ptr = &mut value as *mut _ as *mut c_void;
            let err = EdsSetPropertyData(self.camera_ref, kEdsPropID_SaveTo, 0, 4, ptr);
            if err != EDS_ERR_OK {
                eprintln!("failed to set write on flash")
            }
        };
        self
    }
    pub fn description(&self) -> anyhow::Result<String> {
        let mut device_info = tagEdsDeviceInfo {
            szPortName: [0; 256],
            szDeviceDescription: [0; 256],
            deviceSubType: 0,
            reserved: 0,
        };
        let err = unsafe { EdsGetDeviceInfo(self.camera_ref, &mut device_info) };
        if err != EDS_ERR_OK {
            Err(anyhow!("failed to get device info: result: {err:?}"))?
        }
        let desc = String::from_utf8(device_info.szDeviceDescription.map(|s| s as u8).to_vec())
            .expect("String parse error");
        Ok(desc)
    }
}
