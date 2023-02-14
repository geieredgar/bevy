use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

pub use bevy_ecs_macros::{ScheduleLabel, SystemSet};
use bevy_utils::define_boxed_label;
use bevy_utils::label::DynHash;

use crate::system::{
    ExclusiveSystemParam, ExclusiveSystemParamFunction, IsExclusiveFunctionSystem,
    IsFunctionSystem, SystemParam, SystemParamFunction,
};

define_boxed_label!(ScheduleLabel);

pub type BoxedSystemSet = Box<dyn SystemSet>;
pub type BoxedScheduleLabel = Box<dyn ScheduleLabel>;

/// Types that identify logical groups of systems.
pub trait SystemSet: DynHash + Debug + Send + Sync + 'static {
    /// Returns `true` if this system set is a [`SystemTypeSet`].
    fn is_system_type(&self) -> bool {
        false
    }

    /// Returns `true` if this set is a "base system set". Systems
    /// can only belong to one base set at a time. Systems and Sets
    /// can only be added to base sets using specialized `in_base_set`
    /// APIs. This enables "mutually exclusive" behaviors. It also
    /// enables schedules to have a "default base set", which can be used
    /// to apply default configuration to systems.
    fn is_base(&self) -> bool {
        false
    }

    /// Creates a boxed clone of the label corresponding to this system set.
    fn dyn_clone(&self) -> Box<dyn SystemSet>;
}

impl PartialEq for dyn SystemSet {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other.as_dyn_eq())
    }
}

impl Eq for dyn SystemSet {}

impl Hash for dyn SystemSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

impl Clone for Box<dyn SystemSet> {
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

/// A [`SystemSet`] grouping instances of the same function.
///
/// This kind of set is automatically populated and thus has some special rules:
/// - You cannot manually add members.
/// - You cannot configure them.
/// - You cannot order something relative to one if it has more than one member.
pub struct SystemTypeSet<T: 'static>(PhantomData<fn() -> T>);

impl<T: 'static> SystemTypeSet<T> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Debug for SystemTypeSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SystemTypeSet")
            .field(&std::any::type_name::<T>())
            .finish()
    }
}

impl<T> Hash for SystemTypeSet<T> {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        // all systems of a given type are the same
    }
}
impl<T> Clone for SystemTypeSet<T> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<T> Copy for SystemTypeSet<T> {}

impl<T> PartialEq for SystemTypeSet<T> {
    #[inline]
    fn eq(&self, _other: &Self) -> bool {
        // all systems of a given type are the same
        true
    }
}

impl<T> Eq for SystemTypeSet<T> {}

impl<T> SystemSet for SystemTypeSet<T> {
    fn is_system_type(&self) -> bool {
        true
    }

    fn dyn_clone(&self) -> Box<dyn SystemSet> {
        Box::new(*self)
    }
}

/// A [`SystemSet`] implicitly created when using
/// [`Schedule::add_systems`](super::Schedule::add_systems).
///
/// The names of the members are stored in a reference counted slice.
/// The pointer to the slice is used as hash value and for the equality checks.
pub struct AnonymousSystemSet {
    pub system_names: Arc<[Cow<'static, str>]>,
}

impl AnonymousSystemSet {
    pub(crate) fn new(system_names: impl Iterator<Item = Cow<'static, str>>) -> Self {
        Self {
            system_names: system_names.collect(),
        }
    }
}

impl Debug for AnonymousSystemSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.system_names.join(", "))
    }
}

// Two AnonymousSystemSets should only be equal if they are clones from the same instance.
// Therefore, we only use the pointer to the slice to determine equality.
impl PartialEq for AnonymousSystemSet {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.system_names, &other.system_names)
    }
}

impl Eq for AnonymousSystemSet {}

// Important: This must be kept in sync with the PartialEq/Eq implementation
impl Hash for AnonymousSystemSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Only hash the pointer, because thats what we use in the equality check
        Arc::as_ptr(&self.system_names).hash(state);
    }
}

impl Clone for AnonymousSystemSet {
    fn clone(&self) -> Self {
        Self {
            system_names: Arc::clone(&self.system_names),
        }
    }
}

impl SystemSet for AnonymousSystemSet {
    fn dyn_clone(&self) -> Box<dyn SystemSet> {
        Box::new(self.clone())
    }
}

/// Types that can be converted into a [`SystemSet`].
pub trait IntoSystemSet<Marker>: Sized {
    type Set: SystemSet;

    fn into_system_set(self) -> Self::Set;
}

// systems sets
impl<S: SystemSet> IntoSystemSet<()> for S {
    type Set = Self;

    #[inline]
    fn into_system_set(self) -> Self::Set {
        self
    }
}

// systems
impl<In, Out, Param, Marker, F> IntoSystemSet<(IsFunctionSystem, In, Out, Param, Marker)> for F
where
    Param: SystemParam,
    F: SystemParamFunction<In, Out, Param, Marker>,
{
    type Set = SystemTypeSet<Self>;

    #[inline]
    fn into_system_set(self) -> Self::Set {
        SystemTypeSet::new()
    }
}

// exclusive systems
impl<In, Out, Param, Marker, F> IntoSystemSet<(IsExclusiveFunctionSystem, In, Out, Param, Marker)>
    for F
where
    Param: ExclusiveSystemParam,
    F: ExclusiveSystemParamFunction<In, Out, Param, Marker>,
{
    type Set = SystemTypeSet<Self>;

    #[inline]
    fn into_system_set(self) -> Self::Set {
        SystemTypeSet::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::Cow,
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use super::AnonymousSystemSet;

    #[test]
    fn same_instance() {
        let set_a = AnonymousSystemSet::new([Cow::Borrowed("A"), Cow::Borrowed("B")].into_iter());
        let set_b = set_a.clone();

        assert_eq!(set_a, set_b);

        let mut hasher = DefaultHasher::default();
        set_a.hash(&mut hasher);
        let hash_a = hasher.finish();

        let mut hasher = DefaultHasher::default();
        set_b.hash(&mut hasher);
        let hash_b = hasher.finish();

        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn same_names() {
        let set_a = AnonymousSystemSet::new([Cow::Borrowed("A"), Cow::Borrowed("B")].into_iter());
        let set_b = AnonymousSystemSet::new([Cow::Borrowed("A"), Cow::Borrowed("B")].into_iter());

        assert_ne!(set_a, set_b);
    }

    #[test]
    fn different_names() {
        let set_a = AnonymousSystemSet::new([Cow::Borrowed("A"), Cow::Borrowed("B")].into_iter());
        let set_b = AnonymousSystemSet::new([Cow::Borrowed("C"), Cow::Borrowed("D")].into_iter());

        assert_ne!(set_a, set_b);
    }
}
