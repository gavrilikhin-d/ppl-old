use std::cell::RefCell;

use super::ty::Struct;

pub struct Package {
    pub types: Vec<Struct>,
}

impl Package {
    pub fn new() -> Self {
        Self { types: vec![] }
    }
}

thread_local! {
    pub static CURRENT_PACKAGE: RefCell<Package> = RefCell::new(Package::new());
}
