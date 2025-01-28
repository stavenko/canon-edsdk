use core::slice;
use std::{
    path::PathBuf,
    ptr::{null, null_mut},
};

use anyhow::anyhow;
use edsdk_sys::{
    EdsAccess_kEdsAccess_ReadWrite, EdsAccess_kEdsAccess_Write, EdsCreateFileStream,
    EdsCreateMemoryStream, EdsDirectoryItemInfo, EdsDirectoryItemRef, EdsDownload,
    EdsDownloadComplete, EdsFileCreateDisposition_kEdsFileCreateDisposition_CreateAlways,
    EdsGetDirectoryItemInfo, EdsGetLength, EdsGetPointer, EdsRelease, EDS_ERR_OK,
};

use crate::{blob::Blob, Sdk};

pub struct Image<'a> {
    pub(crate) sdk: &'a Sdk,
    pub(crate) reference: EdsDirectoryItemRef,
}

impl<'a> Image<'a> {
    pub fn get_blob(&self) -> anyhow::Result<Blob> {
        let mut dir_item_info = EdsDirectoryItemInfo {
            size: 0,
            isFolder: 0,
            groupID: 0,
            option: 0,
            szFileName: [0; 256],
            format: 0,
            dateTime: 0,
        };
        let err = unsafe { EdsGetDirectoryItemInfo(self.reference, &mut dir_item_info) };
        if err != EDS_ERR_OK {
            Err(anyhow!("Cannot get directory item info: {err}"))?
        }

        let mut stream = null_mut();

        let err = unsafe { EdsCreateMemoryStream(0, &mut stream) };

        if err != EDS_ERR_OK {
            Err(anyhow!("Failed to create stream: {err}"))?
        }

        let err = unsafe { EdsDownload(self.reference, dir_item_info.size, stream) };

        if err != EDS_ERR_OK {
            Err(anyhow!("Failed to download"))?
        }

        let err = unsafe { EdsDownloadComplete(self.reference) };

        if err != EDS_ERR_OK {
            Err(anyhow!("Failed to set download is completed"))?
        }

        let mut ptr = null_mut();
        let err = unsafe { EdsGetPointer(stream, &mut ptr) };

        if err != EDS_ERR_OK {
            Err(anyhow!("failed to get stream ptr"))?
        }
        let mut size = 0;

        let err = unsafe { EdsGetLength(stream, &mut size) };

        if err != EDS_ERR_OK {
            Err(anyhow!("failed to get stream size"))?
        }
        let data = unsafe { slice::from_raw_parts(ptr as *const u8, size as usize) }.to_vec();
        let err = unsafe { EdsRelease(stream) };

        if err != EDS_ERR_OK {
            Err(anyhow!("Failed to release stream"))?
        }

        Ok(Blob::Jpeg(data))
    }

    pub fn save_to(&self, dir: impl Into<PathBuf>) -> anyhow::Result<()> {
        let mut dir_item_info = EdsDirectoryItemInfo {
            size: 0,
            isFolder: 0,
            groupID: 0,
            option: 0,
            szFileName: [0; 256],
            format: 0,
            dateTime: 0,
        };
        let err = unsafe { EdsGetDirectoryItemInfo(self.reference, &mut dir_item_info) };
        if err != EDS_ERR_OK {
            Err(anyhow!("Cannot get directory item info: {err}"))?
        }

        let mut stream = null_mut();
        let file_name = String::from_utf8(
            dir_item_info
                .szFileName
                .map(|ch| ch as u8)
                .as_slice()
                .to_vec(),
        )?;
        let dir: PathBuf = dir.into();
        if !std::fs::exists(&dir)? {
            eprintln!("Directory not exists - try to create it.");
            if let Err(err) = std::fs::create_dir_all(&dir) {
                Err(anyhow!("Failed to create directory: {err}"))?
            }
        }
        let file_name = dir
            .join(file_name)
            .to_str()
            .expect("Directory has bad symbols")
            .to_string();
        let err = unsafe {
            EdsCreateFileStream(
                file_name.as_ptr() as *const i8,
                EdsFileCreateDisposition_kEdsFileCreateDisposition_CreateAlways,
                EdsAccess_kEdsAccess_ReadWrite,
                &mut stream,
            )
        };

        if err != EDS_ERR_OK {
            Err(anyhow!("Failed to create stream: {err}"))?
        }

        let err = unsafe { EdsDownload(self.reference, dir_item_info.size, stream) };

        if err != EDS_ERR_OK {
            Err(anyhow!("Failed to download"))?
        }

        let err = unsafe { EdsDownloadComplete(self.reference) };

        if err != EDS_ERR_OK {
            Err(anyhow!("Failed to set download is completed"))?
        }

        let err = unsafe { EdsRelease(stream) };

        if err != EDS_ERR_OK {
            Err(anyhow!("Failed to release stream"))?
        }

        Ok(())
    }
}
