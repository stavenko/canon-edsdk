use std::{ptr::null_mut, slice, time::Duration};

use edsdk_sys::{
    EdsBaseRef, EdsCameraRef, EdsDownloadEvfImage, EdsEvfImageRef, EdsGetLength, EdsGetPointer,
    EdsStreamRef, EDS_ERR_OBJECT_NOTREADY, EDS_ERR_OK,
};

use crate::blob::Blob;

pub struct LiveViewStream {
    camera_ref: EdsCameraRef,
    evf_image: EdsBaseRef,
    stream: EdsBaseRef,
}

impl LiveViewStream {
    pub(crate) fn new(
        camera_ref: EdsCameraRef,
        evf_image: EdsEvfImageRef,
        stream: EdsStreamRef,
    ) -> Self {
        Self {
            camera_ref,
            evf_image,
            stream,
        }
    }
}

impl Iterator for LiveViewStream {
    type Item = Blob;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let err = unsafe { EdsDownloadEvfImage(self.camera_ref, self.evf_image) };
            if err != EDS_ERR_OK && err == EDS_ERR_OBJECT_NOTREADY {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            } else if err != EDS_ERR_OK {
                eprintln!("Problems in image download: {err}");
                return None;
            }

            let mut ptr = null_mut();
            let err = unsafe { EdsGetPointer(self.stream, &mut ptr) };

            if err != EDS_ERR_OK {
                eprintln!("failed to get stream ptr");
                return None;
            }
            let mut size = 0;

            let err = unsafe { EdsGetLength(self.stream, &mut size) };

            if err != EDS_ERR_OK {
                eprintln!("failed to get stream size",);
                return None;
            }
            let data = unsafe { slice::from_raw_parts(ptr as *const u8, size as usize) }.to_vec();
            return Some(Blob::Jpeg(data));
        }
    }
}
