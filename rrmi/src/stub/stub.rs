use std::fmt::Debug;

use crate::RemoteRef;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Stub {
    // Generic Stub as an intermediate step before generating ObjectStub
    pub remote: RemoteRef,
}

impl Stub {
    pub fn new(remote: RemoteRef) -> Self {
        Stub { remote }
    }

    pub fn from(remote: RemoteRef) -> Self {
        Stub { remote }
    }
}
