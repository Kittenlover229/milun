use core::ffi::c_size_t;
use std::{
    convert::Infallible,
    ffi::{c_char, c_void, CStr},
};

use crate::StandaloneRenderer;

#[no_mangle]
pub extern "C" fn tangerine_new(
    alloc: extern "C" fn(c_size_t) -> *mut c_void,
) -> *mut StandaloneRenderer {
    let ptr = alloc(std::mem::size_of::<StandaloneRenderer>()) as *mut StandaloneRenderer;
    unsafe {
        std::ptr::write(ptr, StandaloneRenderer::new("Hello, world!"));
    }

    ptr
}

#[no_mangle]
pub extern "C" fn tangerine_delete(
    renderer: *mut StandaloneRenderer,
    free: extern "C" fn(*mut c_void),
) {
    unsafe {
        std::ptr::drop_in_place(renderer);
    }
    free(renderer as *mut c_void);
}

#[no_mangle]
pub extern "C" fn tangerine_set_title(renderer: *mut StandaloneRenderer, title: *const c_char) {
    unsafe {
        let c_str = CStr::from_ptr(title).to_str().unwrap();
        (&mut *renderer).window.set_title(c_str);
    }
}

#[no_mangle]
pub extern "C" fn tangerine_run(renderer: *mut StandaloneRenderer) {
    unsafe {
        let _ =
            std::ptr::read(renderer).run::<Infallible>(|renderer, _| Ok(renderer.begin_frame()));
    };
}
