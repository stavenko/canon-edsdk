use std::{
    path::PathBuf,
    ptr::{null, null_mut},
};

use anyhow::anyhow;
use edsdk_sys::{
    EdsAccess_kEdsAccess_ReadWrite, EdsAccess_kEdsAccess_Write, EdsCreateFileStream,
    EdsDirectoryItemInfo, EdsDirectoryItemRef, EdsDownload, EdsDownloadComplete,
    EdsFileCreateDisposition_kEdsFileCreateDisposition_CreateAlways, EdsGetDirectoryItemInfo,
    EdsRelease, EDS_ERR_OK,
};

use crate::Sdk;

pub struct Image<'a> {
    pub(crate) sdk: &'a Sdk,
    pub(crate) reference: EdsDirectoryItemRef,
}

impl<'a> Image<'a> {
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
