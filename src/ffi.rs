use std::{ffi::CStr, os::raw::c_void};

use crate::{AudioClip, Pcm, ResampleContinuation};

#[repr(C)]
pub struct CResampleResult {
    pub channels: libc::size_t,
    pub frames: libc::size_t,
    pub is_done: bool,
    pub buffer: *mut c_void,
}

#[no_mangle]
pub extern "C" fn pcm_new_from_file(file: *const libc::c_char) -> *mut c_void {
    let file = unsafe {
        let ptr = file;
        CStr::from_ptr(ptr)
    };
    let file = file.to_str().unwrap();

    let pcm = Pcm::new_from_file(file);

    if let Some(pcm) = pcm {
        Box::into_raw(Box::new(pcm)) as *mut c_void
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pcm_new_from_data(data: *const u8, size: libc::size_t) -> *mut c_void {
    let data = unsafe {
        let slice = std::slice::from_raw_parts(data, size);
        slice.to_vec()
    };

    let pcm = Pcm::new_from_data(data);

    if let Some(pcm) = pcm {
        Box::into_raw(Box::new(pcm)) as *mut c_void
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pcm_free(pcm_ptr: *mut c_void) {
    if pcm_ptr.is_null() {
        return;
    }

    unsafe {
        let _ = Box::from_raw(pcm_ptr as *mut Pcm);
    }
}

#[no_mangle]
pub extern "C" fn audio_clip_new(
    pcm_ptr: *mut c_void,
    target_sample_rate: libc::size_t,
    chunk_size: libc::size_t,
) -> *mut c_void {
    let pcm = unsafe {
        assert!(!pcm_ptr.is_null());
        let pcm_ptr = pcm_ptr as *mut Pcm;
        &*pcm_ptr
    };

    let audio_clip = AudioClip::new(pcm, target_sample_rate, chunk_size);

    if let Some(audio_clip) = audio_clip {
        Box::into_raw(Box::new(audio_clip)) as *mut c_void
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn audio_clip_free(audio_clip_ptr: *mut c_void) {
    if audio_clip_ptr.is_null() {
        return;
    }

    unsafe {
        let _ = Box::from_raw(audio_clip_ptr as *mut AudioClip);
    }
}

#[no_mangle]
pub extern "C" fn audio_clip_resample_next(audio_clip_ptr: *mut c_void) -> *mut c_void {
    let audio_clip = unsafe {
        assert!(!audio_clip_ptr.is_null());
        &mut *(audio_clip_ptr as *mut AudioClip)
    };

    let result = audio_clip.resample_next();

    if let Ok((buffer, resample_result)) = result {
        let channels = buffer.len();
        let frames = buffer[0].len();

        let is_done = match resample_result {
            ResampleContinuation::MoreData => false,
            ResampleContinuation::NoMoreData => true,
        };

        let buffer_ptr = buffer.as_ptr() as *mut c_void;

        let resample_result = CResampleResult {
            channels,
            frames,
            is_done,
            buffer: buffer_ptr,
        };

        Box::into_raw(Box::new(resample_result)) as *mut c_void
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn resample_result_free(resample_result_ptr: *mut c_void) {
    if resample_result_ptr.is_null() {
        return;
    }

    unsafe {
        let _ = Box::from_raw(resample_result_ptr as *mut CResampleResult);
    }
}
