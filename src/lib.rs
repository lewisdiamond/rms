pub mod cmd;
pub mod message;
pub mod readmail;
pub mod stores;

extern crate jemallocator;
#[cfg(test)]
extern crate rand;
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
