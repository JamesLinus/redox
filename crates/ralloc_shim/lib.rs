#![crate_name="ralloc_shim"]
#![crate_type="lib"]
#![feature(core_intrinsics)]
#![feature(lang_items)]
#![no_std]

extern crate system;

pub mod config;
pub mod syscalls;
pub mod thread_destructor;
