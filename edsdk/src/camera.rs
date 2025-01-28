use std::{
    collections::VecDeque,
    ffi::c_void,
    ptr::{null, null_mut},
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

use anyhow::anyhow;
use edsdk_sys::{
    kEdsCameraCommand_ExtendShutDownTimer, kEdsCameraCommand_PressShutterButton,
    kEdsObjectEvent_All, kEdsPropID_Evf_OutputDevice, kEdsPropID_Orientation, kEdsPropID_SaveTo,
    kEdsPropertyEvent_All, kEdsStateEvent_All, kEdsStateEvent_ShutDownTimerUpdate,
    tagEdsDeviceInfo, EdsCameraRef, EdsCloseSession, EdsCreateEvfImageRef, EdsCreateMemoryStream,
    EdsDownloadEvfImage, EdsEvfOutputDevice_kEdsEvfOutputDevice_PC,
    EdsEvfOutputDevice_kEdsEvfOutputDevice_PC_Small, EdsGetChildAtIndex, EdsGetDeviceInfo,
    EdsGetLength, EdsGetPointer, EdsGetPropertyData, EdsOpenSession, EdsRead, EdsRelease,
    EdsSendCommand, EdsSetCameraStateEventHandler, EdsSetObjectEventHandler, EdsSetPropertyData,
    EdsSetPropertyEventHandler, EdsShutterButton_kEdsCameraCommand_ShutterButton_Completely_NonAF,
    EdsShutterButton_kEdsCameraCommand_ShutterButton_OFF, EDS_ERR_OK,
};

use crate::{
    events::{camera_state_event_handler, object_event_handler, property_event_handler, Event},
    image::Image,
    live_view_stream::LiveViewStream,
    sdk::Sdk,
};

#[derive(Debug)]
pub enum Orientation {
    Zero,
    CW90,
    CW180,
    CCW90,
}

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
                kEdsPropertyEvent_All,
                Some(property_event_handler),
                ctx,
            )
        };
        if err != EDS_ERR_OK {
            Err(anyhow!("failed to set up props event handler",))?
        }

        let err = unsafe {
            EdsSetCameraStateEventHandler(
                camera,
                kEdsStateEvent_All,
                Some(camera_state_event_handler),
                ctx,
            )
        };
        if err != EDS_ERR_OK {
            Err(anyhow!("failed to set up state event handler",))?
        }

        let err = unsafe { EdsSendCommand(camera, kEdsCameraCommand_ExtendShutDownTimer, 1) };

        if err != EDS_ERR_OK {
            Err(anyhow!("failed to extend camera shut down interval",))?
        }

        Ok(Self {
            id: camera_id,
            sdk,
            camera_ref: camera,
            events,
        })
    }

    pub fn live_view_stream(&self) -> anyhow::Result<LiveViewStream> {
        let mut device = 0;
        let param = &mut device as *mut _ as *mut c_void;
        let err = unsafe {
            EdsGetPropertyData(
                self.camera_ref,
                kEdsPropID_Evf_OutputDevice,
                0,
                size_of::<u32>() as u32,
                param,
            )
        };

        if err != EDS_ERR_OK {
            Err(anyhow!("failed to get camera live-view device property",))?
        }

        if device != EdsEvfOutputDevice_kEdsEvfOutputDevice_PC {
            Err(anyhow!(
                "Live view device not set to PC, live view inactive"
            ))?
        }
        println!("Live view device: {device}");

        let mut stream = null_mut();
        let err = unsafe { EdsCreateMemoryStream(0, &mut stream) };

        if err != EDS_ERR_OK {
            Err(anyhow!("failed to create camera memory stream",))?
        }

        let mut evf_image = null_mut();
        let err = unsafe { EdsCreateEvfImageRef(stream, &mut evf_image) };

        if err != EDS_ERR_OK {
            Err(anyhow!("failed to create efv image",))?
        }
        let lv_stream = LiveViewStream::new(self.camera_ref, evf_image, stream);

        Ok(lv_stream)
    }

    pub fn live_view(&self) -> anyhow::Result<&Self> {
        let mut device = 0;
        let param = &mut device as *mut _ as *mut c_void;
        let err = unsafe {
            EdsGetPropertyData(
                self.camera_ref,
                kEdsPropID_Evf_OutputDevice,
                0,
                size_of::<u32>() as u32,
                param,
            )
        };

        if err != EDS_ERR_OK {
            Err(anyhow!("failed to get camera live-view device property",))?
        }
        let mut device = EdsEvfOutputDevice_kEdsEvfOutputDevice_PC;
        let param = &mut device as *mut _ as *mut c_void;
        let err = unsafe {
            EdsSetPropertyData(
                self.camera_ref,
                kEdsPropID_Evf_OutputDevice,
                0,
                size_of::<u32>() as u32,
                param,
            )
        };
        if err != EDS_ERR_OK {
            Err(anyhow!("failed to SET camera live-view device property",))?
        }
        // println!("LV prop: {device}");

        Ok(self)
    }

    pub fn orientation(&self) -> anyhow::Result<Orientation> {
        let mut orientation = 0;
        let ptr = &mut orientation as *mut _ as *mut c_void;
        let err = unsafe {
            EdsGetPropertyData(
                self.camera_ref,
                kEdsPropID_Orientation,
                0,
                size_of::<u32>() as u32,
                ptr,
            )
        };

        if err != EDS_ERR_OK {
            Err(anyhow!("failed to GET camera Orientation: {err}",))?
        }

        let orientation = match orientation {
            1 => Orientation::Zero,
            3 => Orientation::CW180,
            6 => Orientation::CW90,
            8 => Orientation::CCW90,
            unexpected => Err(anyhow!("Unexpected value for orientation: {unexpected}"))?,
        };
        Ok(orientation)
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
