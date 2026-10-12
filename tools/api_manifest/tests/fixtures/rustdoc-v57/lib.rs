#![allow(dead_code)]
#![feature(associated_type_defaults)]

pub trait Contract<T> {
    type Output = T;
    const LIMIT: usize = 11;

    fn required(&self) -> Self::Output;

    fn provided(&self) -> usize {
        Self::LIMIT
    }
}

pub trait DefaultMembers {
    type Output = u8;
    const LIMIT: usize = 13;
}

mod private_module {
    pub trait HiddenContract {
        fn hidden_method(&self) {}
    }
}

mod hidden_view {
    pub struct View<'a> {
        value: &'a u8,
    }

    impl View<'_> {
        pub fn get(&self) -> &u8 {
            self.value
        }
    }
}

pub use hidden_view::View as Alias;

pub trait Audit {
    fn required(&self) -> u8;
}

pub trait StaticContract {
    fn static_bound(&self);
}

trait PrivateContract {
    const PRIVATE_LIMIT: usize = 5;

    fn hidden(&self) -> usize {
        Self::PRIVATE_LIMIT
    }
}

pub struct Service;

impl DefaultMembers for Service {}

impl private_module::HiddenContract for Service {}

pub struct Generic<T> {
    pub value: T,
}

pub struct Fixed<T> {
    pub value: T,
}

impl Service {
    pub fn required(&self) -> u8 {
        9
    }

    pub fn inherent(&self) -> usize {
        7
    }
}

impl Contract<u8> for Service {
    fn required(&self) -> u8 {
        3
    }
}

impl Contract<u16> for Service {
    fn required(&self) -> u16 {
        3
    }
}

impl<T> Contract<T> for Generic<T>
where
    T: Copy,
{
    fn required(&self) -> T {
        self.value
    }
}

impl<T: 'static> StaticContract for Generic<T> {
    fn static_bound(&self) {}
}

impl Generic<(u8,)> {
    pub fn only_tuple_u8(&self) -> u8 {
        self.value.0
    }
}

impl Generic<(u16,)> {
    pub fn only_tuple_u16(&self) -> u16 {
        self.value.0
    }
}

impl Contract<u8> for Fixed<u8> {
    fn required(&self) -> u8 {
        self.value
    }
}

impl Contract<u16> for Fixed<u16> {
    fn required(&self) -> u16 {
        self.value
    }
}

impl Audit for Service {
    fn required(&self) -> u8 {
        5
    }
}

impl PrivateContract for Service {
    fn hidden(&self) -> usize {
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

pub type ServiceAlias = Service;
pub type ServiceAliasChain = ServiceAlias;
pub type StateAlias = State;
pub type GenericAlias<T> = Generic<T>;
pub type GenericDerivedAlias = Generic<Derived>;
pub type GenericU8TupleAlias = Generic<(u8,)>;
pub type GenericU16TupleAlias = Generic<(u16,)>;
pub type FixedU8Alias = Fixed<u8>;
pub type FixedU16Alias = Fixed<u16>;

impl Fixed<u16> {
    pub fn only_u16(&self) -> u16 {
        self.value
    }
}
