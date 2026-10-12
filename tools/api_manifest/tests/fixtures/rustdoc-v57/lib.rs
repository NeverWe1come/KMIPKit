#![allow(dead_code)]

pub trait Contract {
    type Output;
    const LIMIT: usize;

    fn required(&self) -> Self::Output;

    fn provided(&self) -> usize {
        Self::LIMIT
    }
}

pub trait Audit {
    fn required(&self) -> u8;
}

pub struct Service;

impl Service {
    pub fn required(&self) -> u8 {
        9
    }

    pub fn inherent(&self) -> usize {
        7
    }
}

impl Contract for Service {
    type Output = u8;
    const LIMIT: usize = 11;

    fn required(&self) -> Self::Output {
        3
    }
}

impl Audit for Service {
    fn required(&self) -> u8 {
        5
    }
}

#[derive(Debug)]
pub struct Derived;

pub enum State {
    Unit,
    Tuple(u8),
    Named { value: u8 },
}

pub struct Generic<T> {
    pub value: T,
}

pub type ServiceAlias = Service;
pub type ServiceAliasChain = ServiceAlias;
pub type StateAlias = State;
pub type GenericAlias<T> = Generic<T>;
