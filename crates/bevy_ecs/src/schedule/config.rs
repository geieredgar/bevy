use bevy_ecs_macros::all_tuples;

use crate::{
    schedule::{
        condition::Condition,
        graph::{IntoSystemGraph, SystemGraph},
        set::{BoxedSystemSet, IntoSystemSet, SystemSet},
    },
    system::{BoxedSystem, IntoSystem},
};

/// A [`SystemSet`] with scheduling metadata.
pub struct SystemSetConfig(SystemGraph);

impl SystemSetConfig {
    pub(super) fn into_inner(self) -> SystemGraph {
        self.0
    }
}

/// A [`System`] with scheduling metadata.
pub struct SystemConfig(SystemGraph);

impl SystemConfig {
    pub(super) fn into_inner(self) -> SystemGraph {
        self.0
    }
}

/// Types that can be converted into a [`SystemSetConfig`].
///
/// This has been implemented for all types that implement [`SystemSet`] and boxed trait objects.
pub trait IntoSystemSetConfig: sealed::IntoSystemSetConfig {
    /// Convert into a [`SystemSetConfig`].
    #[doc(hidden)]
    fn into_config(self) -> SystemSetConfig;
    /// Add to the provided `set`.
    #[track_caller]
    fn in_set(self, set: impl SystemSet) -> SystemSetConfig;
    /// Add to the provided "base" `set`. For more information on base sets, see [`SystemSet::is_base`].
    #[track_caller]
    fn in_base_set(self, set: impl SystemSet) -> SystemSetConfig;
    /// Add this set to the schedules's default base set.
    fn in_default_base_set(self) -> SystemSetConfig;
    /// Run before all systems in `set`.
    fn before<M>(self, set: impl IntoSystemSet<M>) -> SystemSetConfig;
    /// Run after all systems in `set`.
    fn after<M>(self, set: impl IntoSystemSet<M>) -> SystemSetConfig;
    /// Run the systems in this set only if the [`Condition`] is `true`.
    ///
    /// The `Condition` will be evaluated at most once (per schedule run),
    /// the first time a system in this set prepares to run.
    fn run_if<P>(self, condition: impl Condition<P>) -> SystemSetConfig;
    /// Suppress warnings and errors that would result from systems in this set having ambiguities
    /// (conflicting access but indeterminate order) with systems in `set`.
    fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> SystemSetConfig;
    /// Suppress warnings and errors that would result from systems in this set having ambiguities
    /// (conflicting access but indeterminate order) with any other system.
    fn ambiguous_with_all(self) -> SystemSetConfig;
}

impl<S> IntoSystemSetConfig for S
where
    S: SystemSet + sealed::IntoSystemSetConfig,
{
    fn into_config(self) -> SystemSetConfig {
        SystemSetConfig(self.into_graph())
    }

    #[track_caller]
    fn in_set(self, set: impl SystemSet) -> SystemSetConfig {
        self.into_config().in_set(set)
    }

    #[track_caller]
    fn in_base_set(self, set: impl SystemSet) -> SystemSetConfig {
        self.into_config().in_base_set(set)
    }

    fn in_default_base_set(self) -> SystemSetConfig {
        self.into_config().in_default_base_set()
    }

    fn before<M>(self, set: impl IntoSystemSet<M>) -> SystemSetConfig {
        self.into_config().before(set)
    }

    fn after<M>(self, set: impl IntoSystemSet<M>) -> SystemSetConfig {
        self.into_config().after(set)
    }

    fn run_if<P>(self, condition: impl Condition<P>) -> SystemSetConfig {
        self.into_config().run_if(condition)
    }

    fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> SystemSetConfig {
        self.into_config().ambiguous_with(set)
    }

    fn ambiguous_with_all(self) -> SystemSetConfig {
        self.into_config().ambiguous_with_all()
    }
}

impl IntoSystemSetConfig for BoxedSystemSet {
    fn into_config(self) -> SystemSetConfig {
        SystemSetConfig(self.into_graph())
    }

    #[track_caller]
    fn in_set(self, set: impl SystemSet) -> SystemSetConfig {
        self.into_config().in_set(set)
    }

    #[track_caller]
    fn in_base_set(self, set: impl SystemSet) -> SystemSetConfig {
        self.into_config().in_base_set(set)
    }

    fn in_default_base_set(self) -> SystemSetConfig {
        self.into_config().in_default_base_set()
    }

    fn before<M>(self, set: impl IntoSystemSet<M>) -> SystemSetConfig {
        self.into_config().before(set)
    }

    fn after<M>(self, set: impl IntoSystemSet<M>) -> SystemSetConfig {
        self.into_config().after(set)
    }

    fn run_if<P>(self, condition: impl Condition<P>) -> SystemSetConfig {
        self.into_config().run_if(condition)
    }

    fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> SystemSetConfig {
        self.into_config().ambiguous_with(set)
    }

    fn ambiguous_with_all(self) -> SystemSetConfig {
        self.into_config().ambiguous_with_all()
    }
}

impl IntoSystemSetConfig for SystemSetConfig {
    fn into_config(self) -> Self {
        self
    }

    #[track_caller]
    fn in_set(self, set: impl SystemSet) -> Self {
        Self(self.0.in_set(set))
    }

    #[track_caller]
    fn in_base_set(self, set: impl SystemSet) -> Self {
        Self(self.0.in_base_set(set))
    }

    fn in_default_base_set(self) -> SystemSetConfig {
        Self(self.0.in_default_base_set())
    }

    fn before<M>(self, set: impl IntoSystemSet<M>) -> Self {
        Self(self.0.before(set.into_system_set().into_graph()))
    }

    fn after<M>(self, set: impl IntoSystemSet<M>) -> Self {
        Self(self.0.after(set.into_system_set().into_graph()))
    }

    fn run_if<P>(self, condition: impl Condition<P>) -> Self {
        Self(self.0.run_if(condition))
    }

    fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> Self {
        Self(self.0.ambiguous_with(set))
    }

    fn ambiguous_with_all(self) -> Self {
        Self(self.0.ambiguous_with_all())
    }
}

/// Types that can be converted into a [`SystemConfig`].
///
/// This has been implemented for boxed [`System<In=(), Out=()>`](crate::system::System)
/// trait objects and all functions that turn into such.
pub trait IntoSystemConfig<Params>: sealed::IntoSystemConfig<Params> {
    /// Convert into a [`SystemConfig`].
    #[doc(hidden)]
    fn into_config(self) -> SystemConfig;
    /// Add to `set` membership.
    #[track_caller]
    fn in_set(self, set: impl SystemSet) -> SystemConfig;
    /// Add to the provided "base" `set`. For more information on base sets, see [`SystemSet::is_base`].
    #[track_caller]
    fn in_base_set(self, set: impl SystemSet) -> SystemConfig;
    /// Don't add this system to the schedules's default set.
    fn no_default_base_set(self) -> SystemConfig;
    /// Run before all systems in `set`.
    fn before<M>(self, set: impl IntoSystemSet<M>) -> SystemConfig;
    /// Run after all systems in `set`.
    fn after<M>(self, set: impl IntoSystemSet<M>) -> SystemConfig;
    /// Run only if the [`Condition`] is `true`.
    ///
    /// The `Condition` will be evaluated at most once (per schedule run),
    /// when the system prepares to run.
    fn run_if<P>(self, condition: impl Condition<P>) -> SystemConfig;
    /// Suppress warnings and errors that would result from this system having ambiguities
    /// (conflicting access but indeterminate order) with systems in `set`.
    fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> SystemConfig;
    /// Suppress warnings and errors that would result from this system having ambiguities
    /// (conflicting access but indeterminate order) with any other system.
    fn ambiguous_with_all(self) -> SystemConfig;
}

impl<Params, F> IntoSystemConfig<Params> for F
where
    F: IntoSystem<(), (), Params> + sealed::IntoSystemConfig<Params>,
{
    fn into_config(self) -> SystemConfig {
        SystemConfig(self.into_graph())
    }

    #[track_caller]
    fn in_set(self, set: impl SystemSet) -> SystemConfig {
        self.into_config().in_set(set)
    }

    #[track_caller]
    fn in_base_set(self, set: impl SystemSet) -> SystemConfig {
        self.into_config().in_base_set(set)
    }

    fn no_default_base_set(self) -> SystemConfig {
        self.into_config().no_default_base_set()
    }

    fn before<M>(self, set: impl IntoSystemSet<M>) -> SystemConfig {
        self.into_config().before(set)
    }

    fn after<M>(self, set: impl IntoSystemSet<M>) -> SystemConfig {
        self.into_config().after(set)
    }

    fn run_if<P>(self, condition: impl Condition<P>) -> SystemConfig {
        self.into_config().run_if(condition)
    }

    fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> SystemConfig {
        self.into_config().ambiguous_with(set)
    }

    fn ambiguous_with_all(self) -> SystemConfig {
        self.into_config().ambiguous_with_all()
    }
}

impl IntoSystemConfig<()> for BoxedSystem<(), ()> {
    fn into_config(self) -> SystemConfig {
        SystemConfig(self.into_graph())
    }

    #[track_caller]
    fn in_set(self, set: impl SystemSet) -> SystemConfig {
        self.into_config().in_set(set)
    }

    #[track_caller]
    fn in_base_set(self, set: impl SystemSet) -> SystemConfig {
        self.into_config().in_base_set(set)
    }

    fn no_default_base_set(self) -> SystemConfig {
        self.into_config().no_default_base_set()
    }

    fn before<M>(self, set: impl IntoSystemSet<M>) -> SystemConfig {
        self.into_config().before(set)
    }

    fn after<M>(self, set: impl IntoSystemSet<M>) -> SystemConfig {
        self.into_config().after(set)
    }

    fn run_if<P>(self, condition: impl Condition<P>) -> SystemConfig {
        self.into_config().run_if(condition)
    }

    fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> SystemConfig {
        self.into_config().ambiguous_with(set)
    }

    fn ambiguous_with_all(self) -> SystemConfig {
        self.into_config().ambiguous_with_all()
    }
}

impl IntoSystemConfig<()> for SystemConfig {
    fn into_config(self) -> Self {
        self
    }

    #[track_caller]
    fn in_set(self, set: impl SystemSet) -> Self {
        Self(self.0.in_set(set))
    }

    #[track_caller]
    fn in_base_set(self, set: impl SystemSet) -> Self {
        Self(self.0.in_base_set(set))
    }

    fn no_default_base_set(self) -> SystemConfig {
        Self(self.0.no_default_base_set())
    }

    fn before<M>(self, set: impl IntoSystemSet<M>) -> Self {
        Self(self.0.before(set.into_system_set().into_graph()))
    }

    fn after<M>(self, set: impl IntoSystemSet<M>) -> Self {
        Self(self.0.after(set.into_system_set().into_graph()))
    }

    fn run_if<P>(self, condition: impl Condition<P>) -> Self {
        Self(self.0.run_if(condition))
    }

    fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> Self {
        Self(self.0.ambiguous_with(set))
    }

    fn ambiguous_with_all(self) -> Self {
        Self(self.0.ambiguous_with_all())
    }
}

// only `System<In=(), Out=()>` system objects can be scheduled
mod sealed {
    use crate::{
        schedule::{BoxedSystemSet, SystemSet},
        system::{BoxedSystem, IntoSystem},
    };

    use super::{SystemConfig, SystemSetConfig};

    pub trait IntoSystemConfig<Params> {}

    impl<Params, F: IntoSystem<(), (), Params>> IntoSystemConfig<Params> for F {}

    impl IntoSystemConfig<()> for BoxedSystem<(), ()> {}

    impl IntoSystemConfig<()> for SystemConfig {}

    pub trait IntoSystemSetConfig {}

    impl<S: SystemSet> IntoSystemSetConfig for S {}

    impl IntoSystemSetConfig for BoxedSystemSet {}

    impl IntoSystemSetConfig for SystemSetConfig {}
}

/// A collection of [`SystemConfig`].
pub struct SystemConfigs(SystemGraph);

impl SystemConfigs {
    pub(super) fn into_inner(self) -> SystemGraph {
        self.0
    }
}

/// Types that can convert into a [`SystemConfigs`].
pub trait IntoSystemConfigs<Params>
where
    Self: Sized,
{
    /// Convert into a [`SystemConfigs`].
    #[doc(hidden)]
    fn into_configs(self) -> SystemConfigs;

    /// Add these systems to the provided `set`.
    #[track_caller]
    fn in_set(self, set: impl SystemSet) -> SystemConfigs {
        self.into_configs().in_set(set)
    }

    /// Add these systems to the provided "base" `set`. For more information on base sets, see [`SystemSet::is_base`].
    #[track_caller]
    fn in_base_set(self, set: impl SystemSet) -> SystemConfigs {
        self.into_configs().in_base_set(set)
    }

    /// Run before all systems in `set`.
    fn before<M>(self, set: impl IntoSystemSet<M>) -> SystemConfigs {
        self.into_configs().before(set)
    }

    /// Run after all systems in `set`.
    fn after<M>(self, set: impl IntoSystemSet<M>) -> SystemConfigs {
        self.into_configs().after(set)
    }

    /// Run these systems only if the [`Condition`] is `true`.
    ///
    /// The `Condition` will be evaluated at most once (per schedule run),
    /// the first time one of these systems prepares to run.
    ///
    /// # Panics
    ///
    /// Panics if [`into_set`](IntoSystemConfigs::into_set) was not called before
    /// [`run_if`](IntoSystemConfigs::run_if).
    /// Use [`distributive_run_if`](IntoSystemConfigs::distributive_run_if) with a cloneable
    /// [`Condition`] if you don't want to turn the system collection into an set.
    fn run_if<P>(self, condition: impl Condition<P>) -> SystemConfigs {
        self.into_configs().run_if(condition)
    }

    /// Run these systems only if the [`Condition`] is `true`
    ///
    /// The `Condition` will be evaluated at most once for each system (per schedule run),
    /// when it prepares to run.
    ///
    /// # Panics
    ///
    /// Panics if [`into_set`](IntoSystemConfigs::into_set) was called before
    /// [`distributive_run_if`](IntoSystemConfigs::distributive_run_if).
    /// Use [`run_if`](IntoSystemConfigs::run_if) after you have turned the system collection
    /// into an set.
    fn distributive_run_if<P>(self, condition: impl Condition<P> + Clone) -> SystemConfigs {
        self.into_configs().distributive_run_if(condition)
    }

    /// Suppress warnings and errors that would result from these systems having ambiguities
    /// (conflicting access but indeterminate order) with systems in `set`.
    fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> SystemConfigs {
        self.into_configs().ambiguous_with(set)
    }

    /// Suppress warnings and errors that would result from these systems having ambiguities
    /// (conflicting access but indeterminate order) with any other system.
    fn ambiguous_with_all(self) -> SystemConfigs {
        self.into_configs().ambiguous_with_all()
    }

    /// Treat this collection as a sequence of systems.
    ///
    /// Ordering constraints will be applied between the successive elements.
    fn chain(self) -> SystemConfigs {
        self.into_configs().chain()
    }

    /// Treat this collection as an anonymous set of systems.
    ///
    /// All following operations will operate on the anonymous set instead of on each
    /// individual system.
    ///
    /// This is required when calling [`run_if`](IntoSystemConfigs::run_if) because the
    /// condition cannot be cloned.
    fn into_set(self) -> SystemConfigs {
        self.into_configs().into_set()
    }
}

impl IntoSystemConfigs<()> for SystemConfigs {
    fn into_configs(self) -> Self {
        self
    }

    #[track_caller]
    fn in_set(self, set: impl SystemSet) -> Self {
        Self(self.0.in_set(set))
    }

    #[track_caller]
    fn in_base_set(self, set: impl SystemSet) -> Self {
        Self(self.0.in_base_set(set))
    }

    fn before<M>(self, set: impl IntoSystemSet<M>) -> Self {
        Self(self.0.before(set.into_system_set().into_graph()))
    }

    fn after<M>(self, set: impl IntoSystemSet<M>) -> Self {
        Self(self.0.after(set.into_system_set().into_graph()))
    }

    fn run_if<P>(self, condition: impl Condition<P>) -> Self {
        Self(self.0.run_if(condition))
    }

    fn distributive_run_if<P>(self, condition: impl Condition<P> + Clone) -> SystemConfigs {
        Self(self.0.run_if(condition))
    }

    fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> Self {
        Self(self.0.ambiguous_with(set))
    }

    fn ambiguous_with_all(self) -> Self {
        Self(self.0.ambiguous_with_all())
    }

    fn chain(self) -> Self {
        Self(self.0.chain())
    }

    fn into_set(self) -> SystemConfigs {
        Self(self.0.into_set())
    }
}

/// A collection of [`SystemSetConfig`].
pub struct SystemSetConfigs(SystemGraph);

impl SystemSetConfigs {
    pub(super) fn into_inner(self) -> SystemGraph {
        self.0
    }
}

/// Types that can convert into a [`SystemSetConfigs`].
pub trait IntoSystemSetConfigs
where
    Self: Sized,
{
    /// Convert into a [`SystemSetConfigs`].
    #[doc(hidden)]
    fn into_configs(self) -> SystemSetConfigs;

    /// Add these system sets to the provided `set`.
    #[track_caller]
    fn in_set(self, set: impl SystemSet) -> SystemSetConfigs {
        self.into_configs().in_set(set)
    }

    /// Add these system sets to the provided "base" `set`. For more information on base sets, see [`SystemSet::is_base`].
    #[track_caller]
    fn in_base_set(self, set: impl SystemSet) -> SystemSetConfigs {
        self.into_configs().in_base_set(set)
    }

    /// Run before all systems in `set`.
    fn before<M>(self, set: impl IntoSystemSet<M>) -> SystemSetConfigs {
        self.into_configs().before(set)
    }

    /// Run after all systems in `set`.
    fn after<M>(self, set: impl IntoSystemSet<M>) -> SystemSetConfigs {
        self.into_configs().after(set)
    }

    /// Suppress warnings and errors that would result from systems in these sets having ambiguities
    /// (conflicting access but indeterminate order) with systems in `set`.
    fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> SystemSetConfigs {
        self.into_configs().ambiguous_with(set)
    }

    /// Suppress warnings and errors that would result from systems in these sets having ambiguities
    /// (conflicting access but indeterminate order) with any other system.
    fn ambiguous_with_all(self) -> SystemSetConfigs {
        self.into_configs().ambiguous_with_all()
    }

    /// Treat this collection as a sequence of system sets.
    ///
    /// Ordering constraints will be applied between the successive elements.
    fn chain(self) -> SystemSetConfigs {
        self.into_configs().chain()
    }
}

impl IntoSystemSetConfigs for SystemSetConfigs {
    fn into_configs(self) -> Self {
        self
    }

    #[track_caller]
    fn in_set(self, set: impl SystemSet) -> Self {
        Self(self.0.in_set(set))
    }

    #[track_caller]
    fn in_base_set(self, set: impl SystemSet) -> Self {
        Self(self.0.in_base_set(set))
    }

    fn before<M>(self, set: impl IntoSystemSet<M>) -> Self {
        Self(self.0.before(set.into_system_set().into_graph()))
    }

    fn after<M>(self, set: impl IntoSystemSet<M>) -> Self {
        Self(self.0.after(set.into_system_set().into_graph()))
    }

    fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> Self {
        Self(self.0.ambiguous_with(set))
    }

    fn ambiguous_with_all(self) -> Self {
        Self(self.0.ambiguous_with_all())
    }

    fn chain(self) -> Self {
        Self(self.0.chain())
    }
}

macro_rules! impl_system_collection {
    ($(($param: ident, $sys: ident)),*) => {
        impl<$($param, $sys),*> IntoSystemConfigs<($($param,)*)> for ($($sys,)*)
        where
            $($sys: IntoSystemConfig<$param>),*
        {
            #[allow(non_snake_case)]
            fn into_configs(self) -> SystemConfigs {
                let ($($sys,)*) = self;
                SystemConfigs(($($sys.into_config().0,)*).into_graph())
            }
        }
    }
}

macro_rules! impl_system_set_collection {
    ($($set: ident),*) => {
        impl<$($set: IntoSystemSetConfig),*> IntoSystemSetConfigs for ($($set,)*)
        {
            #[allow(non_snake_case)]
            fn into_configs(self) -> SystemSetConfigs {
                let ($($set,)*) = self;
                SystemSetConfigs(($($set.into_config().0,)*).into_graph())
            }
        }
    }
}

all_tuples!(impl_system_collection, 0, 15, P, S);
all_tuples!(impl_system_set_collection, 0, 15, S);
