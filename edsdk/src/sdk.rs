use std::{
    ptr::null_mut,
    sync::{atomic::AtomicBool, Arc},
    thread::JoinHandle,
    time::Duration,
};

use edsdk_sys::{
    EdsCameraListRef, EdsGetCameraList, EdsGetChildCount, EdsRelease, EdsTerminateSDK, EDS_ERR_OK,
};

use crate::camera::Camera;

pub struct Sdk {
    pub(crate) camera_list: EdsCameraListRef,
    thread: Option<JoinHandle<()>>,
    continue_updates: Arc<AtomicBool>,
}

impl Drop for Sdk {
    fn drop(&mut self) {
        let err = unsafe { EdsRelease(self.camera_list) };
        if err != EDS_ERR_OK {
            eprintln!("Some fails(release camera list):{err}");
        }
        let err = unsafe { EdsTerminateSDK() };
        if err != EDS_ERR_OK {
            eprintln!("Some fails (terminate):{err}");
        }
        self.continue_updates
            .store(false, std::sync::atomic::Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("Thread not joined");
        }

        println!("Dropped");
    }
}

impl Sdk {
    pub fn get_camera_list(&self) -> anyhow::Result<Vec<Camera>> {
        let mut count = 0;
        let err = unsafe { EdsGetChildCount(self.camera_list, &mut count) };

        if err != EDS_ERR_OK {
            Err(anyhow::anyhow!(
                "failed to get amount of cameras: result: {err:?}"
            ))?
        }
        let mut cameras = Vec::new();
        for c in 0..count {
            cameras.push(Camera::init(self, c)?)
        }
        Ok(cameras)
    }

    pub fn init() -> anyhow::Result<Self> {
        let err = unsafe { edsdk_sys::EdsInitializeSDK() };

        if err != EDS_ERR_OK {
            Err(anyhow::anyhow!("Cannot initialize SDK"))?
        }

        let camera_list = unsafe {
            let mut camera_list = null_mut();
            let err = EdsGetCameraList(&mut camera_list);
            if err != EDS_ERR_OK {
                Err(anyhow::anyhow!(
                    "failed to get camera list: result: {err:?}"
                ))?
            }
            let mut count = 0;
            let err = EdsGetChildCount(camera_list, &mut count);
            if err != EDS_ERR_OK {
                Err(anyhow::anyhow!(
                    "failed to get camera list: result: {err:?}"
                ))?
            }
            camera_list
        };
        let continue_updates = Arc::new(AtomicBool::new(true));
        let thread = std::thread::spawn({
            let continue_updates = continue_updates.clone();
            move || {
                while continue_updates.load(std::sync::atomic::Ordering::Relaxed) {
                    unsafe { edsdk_sys::EdsGetEvent() };
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        });

        Ok(Self {
            camera_list,
            thread: Some(thread),
            continue_updates,
        })
    }
}
