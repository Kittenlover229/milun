use std::{
    alloc::Layout,
    convert::Infallible,
    ffi::{c_char, c_uchar, CStr},
};

use crate::{StandaloneInputState, StandaloneRenderer};

#[no_mangle]
unsafe extern "C" fn tangerine_new() -> *mut StandaloneRenderer {
    let ptr = std::alloc::alloc(Layout::new::<StandaloneRenderer>()) as *mut StandaloneRenderer;
    std::ptr::write(ptr, StandaloneRenderer::new("Hello, world!"));
    ptr
}

#[no_mangle]
unsafe extern "C" fn tangerine_delete(renderer: *mut StandaloneRenderer) {
    std::alloc::dealloc(renderer as _, Layout::new::<StandaloneRenderer>());
}

#[no_mangle]
unsafe extern "C" fn tangerine_set_title(renderer: *mut StandaloneRenderer, title: *const c_char) {
    let c_str = CStr::from_ptr(title).to_str().unwrap();
    (&mut *renderer).window.set_title(c_str);
}

#[no_mangle]
unsafe extern "C" fn tangerine_set_background_color(
    renderer: *mut StandaloneRenderer,
    color_rgb: *mut c_uchar,
) {
    (&mut *renderer).clear_color = if !color_rgb.is_null() {
        Some(
            [
                *color_rgb.offset(0),
                *color_rgb.offset(1),
                *color_rgb.offset(2),
            ]
            .into(),
        )
    } else {
        None
    }
}

#[no_mangle]
unsafe extern "C" fn tangerine_run(
    renderer: *mut StandaloneRenderer,
    callback: extern "C" fn(*mut StandaloneRenderer, StandaloneInputState) -> (),
) {
    let _ = std::ptr::read(renderer).run::<Infallible>(move |_, input| {
        callback(renderer, input);
        Ok(())
    });
}
