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

mod alpha_view {
    pub struct AlphaView<'a> {
        value: &'a u8,
    }

    impl<'b> AlphaView<'b> {
        pub fn get(&self) -> &u8 {
            self.value
        }
    }
}

pub use alpha_view::AlphaView as AlphaAlias;

pub struct LifetimeBound<T> {
    value: T,
}

pub type LifetimeBoundAlias<'a> = LifetimeBound<&'a u8>;
pub type StaticLifetimeBoundAlias = LifetimeBound<&'static u8>;

impl<'a> LifetimeBound<&'a u8>
where
    'a: 'static,
{
    pub fn only_when_static(&self) -> &u8 {
        self.value
    }
}

pub struct InlineLifetimeBound<T> {
    value: T,
}

pub type InlineLifetimeBoundAlias<'a> = InlineLifetimeBound<&'a u8>;
pub type StaticInlineLifetimeBoundAlias = InlineLifetimeBound<&'static u8>;

impl<'a: 'static> InlineLifetimeBound<&'a u8> {
    pub fn only_inline_static(&self) -> &u8 {
        self.value
    }
}

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

#[derive(Clone, Debug)]
pub struct CloneDebug;

impl core::fmt::Display for CloneDebug {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("clone-debug")
    }
}

pub struct NoTraits;

#[derive(Clone)]
pub struct Conditional<T>(pub T);

pub trait RequiresClone {
    fn clone_bound(&self);
}

impl<T: Clone> RequiresClone for T {
    fn clone_bound(&self) {}
}

pub trait RequiresDebug {
    fn debug_bound(&self);
}

impl<T: core::fmt::Debug> RequiresDebug for T {
    fn debug_bound(&self) {}
}

pub trait RequiresDisplay {
    fn display_bound(&self);
}

impl<T: core::fmt::Display> RequiresDisplay for T {
    fn display_bound(&self) {}
}

pub trait Printable {
    fn printable(&self);
}

impl<T: Copy + core::fmt::Debug> Printable for T {
    fn printable(&self) {}
}

#[derive(Clone, Copy, Debug)]
pub struct PrintableValue;

#[derive(Clone, Copy)]
pub struct CopyOnly;

#[derive(Debug)]
pub struct DebugOnly;

#[derive(Clone)]
pub struct CloneOnly;

pub struct CoreTraitCycle<T>(pub core::marker::PhantomData<T>);

impl<T: core::fmt::Debug> Clone for CoreTraitCycle<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T: Clone> core::fmt::Debug for CoreTraitCycle<T> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("CoreTraitCycle")
    }
}

pub struct A<T>(pub core::marker::PhantomData<T>);

pub struct B<T>(pub core::marker::PhantomData<T>);

impl<T> Clone for A<T>
where
    B<T>: core::fmt::Debug,
{
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> core::fmt::Debug for B<T>
where
    A<T>: Clone,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("B")
    }
}

pub struct Recursive<T>(pub core::marker::PhantomData<T>);

pub trait RecursiveBound {
    fn recurse(&self);
}

impl<T> RecursiveBound for T
where
    Recursive<T>: Clone,
{
    fn recurse(&self) {}
}

impl<T: RecursiveBound> Clone for Recursive<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

pub type CloneDebugAlias = CloneDebug;
pub type NoTraitsAlias = NoTraits;
pub type ConditionalCloneDebugAlias = Conditional<CloneDebug>;
pub type ConditionalNoTraitsAlias = Conditional<NoTraits>;
pub type ConditionalOpenAlias<T> = Conditional<T>;
pub type RecursiveU8Alias = Recursive<u8>;
pub type PrintableValueAlias = PrintableValue;
pub type CopyOnlyAlias = CopyOnly;
pub type DebugOnlyAlias = DebugOnly;
pub type CloneOnlyAlias = CloneOnly;
pub type AliasA = A<u8>;
pub type AliasB = B<u8>;

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

pub type StaticAlias = hidden_view::View<'static>;
