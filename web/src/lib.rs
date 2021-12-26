use std::{
    cell::{self, RefCell},
    io,
    rc::Rc,
};

use phar::read::index;
use wasm_bindgen::prelude::*;

mod app;
mod browser;
mod choose_file;
mod file_list;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    yew::start_app::<app::Comp>();
}

pub type RawReader = phar::Reader<io::Cursor<Vec<u8>>, index::MetadataBTreeMap>;

#[derive(Clone)]
pub struct Reader {
    inner: Rc<RefCell<RawReader>>,
}

impl Reader {
    pub fn new(reader: RawReader) -> Self {
        Self {
            inner: Rc::new(RefCell::new(reader)),
        }
    }

    pub fn borrow(&self) -> cell::RefMut<RawReader> {
        self.inner.borrow_mut()
    }
}

impl PartialEq for Reader {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}
