use std::{collections::VecDeque, sync::RwLock};

use edsdk_sys::{
    kEdsObjectEvent_DirItemCreated, kEdsObjectEvent_VolumeInfoChanged, kEdsPropID_ProductName,
    EdsBaseRef, EdsDirectoryItemRef, EdsObjectEvent, EdsVoid,
};

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
    println!("Prop: {property_id} is {event}");
    match property_id {
        edsdk_sys::kEdsPropID_ProductName => (),
        edsdk_sys::kEdsPropID_OwnerName => (),
        edsdk_sys::kEdsPropID_MakerName => (),
        edsdk_sys::kEdsPropID_DateTime => (),
        edsdk_sys::kEdsPropID_FirmwareVersion => (),
        edsdk_sys::kEdsPropID_BatteryLevel => (),
        edsdk_sys::kEdsPropID_SaveTo => (),
        edsdk_sys::kEdsPropID_CurrentStorage => (),
        edsdk_sys::kEdsPropID_CurrentFolder => (),
        edsdk_sys::kEdsPropID_BatteryQuality => (),
        edsdk_sys::kEdsPropID_BodyIDEx => (),
        edsdk_sys::kEdsPropID_HDDirectoryStructure => (),
        edsdk_sys::kEdsPropID_ImageQuality => (),
        edsdk_sys::kEdsPropID_Orientation => (),
        edsdk_sys::kEdsPropID_ICCProfile => (),
        edsdk_sys::kEdsPropID_FocusInfo => (),
        edsdk_sys::kEdsPropID_WhiteBalance => (),
        edsdk_sys::kEdsPropID_ColorTemperature => (),
        edsdk_sys::kEdsPropID_WhiteBalanceShift => (),
        edsdk_sys::kEdsPropID_ColorSpace => (),
        edsdk_sys::kEdsPropID_PictureStyle => (),
        edsdk_sys::kEdsPropID_PictureStyleDesc => (),
        edsdk_sys::kEdsPropID_PictureStyleCaption => (),
        edsdk_sys::kEdsPropID_GPSVersionID => (),
        edsdk_sys::kEdsPropID_GPSLatitudeRef => (),
        edsdk_sys::kEdsPropID_GPSLatitude => (),
        edsdk_sys::kEdsPropID_GPSLongitudeRef => (),
        edsdk_sys::kEdsPropID_GPSLongitude => (),
        edsdk_sys::kEdsPropID_GPSAltitudeRef => (),
        edsdk_sys::kEdsPropID_GPSAltitude => (),
        edsdk_sys::kEdsPropID_GPSTimeStamp => (),
        edsdk_sys::kEdsPropID_GPSSatellites => (),
        edsdk_sys::kEdsPropID_GPSStatus => (),
        edsdk_sys::kEdsPropID_GPSMapDatum => (),
        edsdk_sys::kEdsPropID_GPSDateStamp => (),
        edsdk_sys::kEdsPropID_AEMode => (),
        edsdk_sys::kEdsPropID_DriveMode => (),
        edsdk_sys::kEdsPropID_ISOSpeed => (),
        edsdk_sys::kEdsPropID_MeteringMode => (),
        edsdk_sys::kEdsPropID_AFMode => (),
        edsdk_sys::kEdsPropID_Av => (),
        edsdk_sys::kEdsPropID_Tv => (),
        edsdk_sys::kEdsPropID_ExposureCompensation => (),
        edsdk_sys::kEdsPropID_FocalLength => (),
        edsdk_sys::kEdsPropID_AvailableShots => (),
        edsdk_sys::kEdsPropID_Bracket => (),
        edsdk_sys::kEdsPropID_WhiteBalanceBracket => (),
        edsdk_sys::kEdsPropID_LensName => (),
        edsdk_sys::kEdsPropID_AEBracket => (),
        edsdk_sys::kEdsPropID_FEBracket => (),
        edsdk_sys::kEdsPropID_ISOBracket => (),
        edsdk_sys::kEdsPropID_NoiseReduction => (),
        edsdk_sys::kEdsPropID_FlashOn => (),
        edsdk_sys::kEdsPropID_RedEye => (),
        edsdk_sys::kEdsPropID_FlashMode => (),
        edsdk_sys::kEdsPropID_LensStatus => (),
        edsdk_sys::kEdsPropID_Artist => (),
        edsdk_sys::kEdsPropID_Copyright => (),
        edsdk_sys::kEdsPropID_AEModeSelect => (),
        edsdk_sys::kEdsPropID_PowerZoom_Speed => (),
        edsdk_sys::kEdsPropID_ColorFilter => (),
        edsdk_sys::kEdsPropID_DigitalZoomSetting => (),
        edsdk_sys::kEdsPropID_AfLockState => (),
        edsdk_sys::kEdsPropID_BrightnessSetting => (),
        edsdk_sys::kEdsPropID_Evf_OutputDevice => {
            println!("EVF Output device set: {something_else}")
        }
        edsdk_sys::kEdsPropID_Evf_Mode => (),
        edsdk_sys::kEdsPropID_Evf_WhiteBalance => (),
        edsdk_sys::kEdsPropID_Evf_ColorTemperature => (),
        edsdk_sys::kEdsPropID_Evf_DepthOfFieldPreview => (),
        edsdk_sys::kEdsPropID_Evf_Zoom => (),
        edsdk_sys::kEdsPropID_Evf_ZoomPosition => (),
        edsdk_sys::kEdsPropID_Evf_Histogram => (),
        edsdk_sys::kEdsPropID_Evf_ImagePosition => (),
        edsdk_sys::kEdsPropID_Evf_HistogramStatus => (),
        edsdk_sys::kEdsPropID_Evf_AFMode => (),
        edsdk_sys::kEdsPropID_Record => (),
        edsdk_sys::kEdsPropID_Evf_HistogramY => (),
        edsdk_sys::kEdsPropID_Evf_HistogramR => (),
        edsdk_sys::kEdsPropID_Evf_HistogramG => (),
        edsdk_sys::kEdsPropID_Evf_HistogramB => (),
        edsdk_sys::kEdsPropID_Evf_CoordinateSystem => (),
        edsdk_sys::kEdsPropID_Evf_ZoomRect => (),
        edsdk_sys::kEdsPropID_Evf_ImageClipRect => (),
        edsdk_sys::kEdsPropID_Evf_PowerZoom_CurPosition => (),
        edsdk_sys::kEdsPropID_Evf_PowerZoom_MaxPosition => (),
        edsdk_sys::kEdsPropID_Evf_PowerZoom_MinPosition => (),
        edsdk_sys::kEdsPropID_UTCTime => (),
        edsdk_sys::kEdsPropID_TimeZone => (),
        edsdk_sys::kEdsPropID_SummerTimeSetting => (),
        edsdk_sys::kEdsPropID_ManualWhiteBalanceData => (),
        edsdk_sys::kEdsPropID_TempStatus => (),
        edsdk_sys::kEdsPropID_MirrorLockUpState => (),
        edsdk_sys::kEdsPropID_FixedMovie => (),
        edsdk_sys::kEdsPropID_MovieParam => (),
        edsdk_sys::kEdsPropID_Aspect => (),
        edsdk_sys::kEdsPropID_ContinuousAfMode => (),
        edsdk_sys::kEdsPropID_MirrorUpSetting => (),
        edsdk_sys::kEdsPropID_MovieServoAf => (),
        edsdk_sys::kEdsPropID_AutoPowerOffSetting => (),
        edsdk_sys::kEdsPropID_AFEyeDetect => (),
        edsdk_sys::kEdsPropID_FocusShiftSetting => (),
        edsdk_sys::kEdsPropID_MovieHFRSetting => (),
        edsdk_sys::kEdsPropID_AFTrackingObject => (),
        edsdk_sys::kEdsPropID_RegisterFocusEdge => (),
        edsdk_sys::kEdsPropID_DriveFocusToEdge => (),
        edsdk_sys::kEdsPropID_FocusPosition => (),
        edsdk_sys::kEdsPropID_StillMovieDivideSetting => (),
        edsdk_sys::kEdsPropID_CardExtension => (),
        edsdk_sys::kEdsPropID_MovieCardExtension => (),
        edsdk_sys::kEdsPropID_StillCurrentMedia => (),
        edsdk_sys::kEdsPropID_MovieCurrentMedia => (),
        edsdk_sys::kEdsPropID_ApertureLockSetting => (),
        edsdk_sys::kEdsPropID_LensIsSetting => (),
        edsdk_sys::kEdsPropID_ScreenDimmerTime => (),
        edsdk_sys::kEdsPropID_ScreenOffTime => (),
        edsdk_sys::kEdsPropID_ViewfinderOffTime => (),
        edsdk_sys::kEdsPropID_Evf_ClickWBCoeffs => (),
        edsdk_sys::kEdsPropID_EVF_RollingPitching => (),
        edsdk_sys::kEdsPropID_Evf_VisibleRect => (),
        edsdk_sys::kEdsPropID_DC_Zoom => (),
        edsdk_sys::kEdsPropID_DC_Strobe => (),
        edsdk_sys::kEdsPropID_LensBarrelStatus => (),
        _ => (),
    }

    println!("Okey prop");
    0
}

pub(crate) unsafe extern "C" fn camera_state_event_handler(
    event: u32,
    property_id: u32,
    context: *mut EdsVoid,
) -> u32 {
    println!("!!!!Okey camera state");
    0
}
