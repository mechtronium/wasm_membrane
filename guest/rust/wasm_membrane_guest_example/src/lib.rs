mod utils;

use wasm_bindgen::prelude::*;
use wasm_membrane_guest::membrane::log;
use crate::utils::set_panic_hook;
use std::{time, thread};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn test()
{
    log( "Test Works!");
}


#[wasm_bindgen]
pub fn membrane_guest_init()
{
    // if you set_panic_hook() it will fail for some odd reason
    //set_panic_hook();
}


