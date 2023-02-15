use downcast_rs::{impl_downcast, Downcast};

use crate::App;
use std::any::Any;

/// A collection of Bevy app logic and configuration.
///
/// Plugins configure an [`App`]. When an [`App`] registers a plugin,
/// the plugin's [`Plugin::build`] function is run. By default, a plugin
/// can only be added once to an [`App`].
///
/// If the plugin may need to be added twice or more, the function [`is_unique()`](Self::is_unique)
/// should be overridden to return `false`. Plugins are considered duplicate if they have the same
/// [`name()`](Self::name). The default `name()` implementation returns the type name, which means
/// generic plugins with different type parameters will not be considered duplicates.
pub trait Plugin: Downcast + Any + Send + Sync {
    /// Configures the [`App`] to which this plugin is added.
    fn build(&self, app: &mut App);

    /// Runs after all plugins are built, but before the app runner is called.
    /// This can be useful if you have some resource that other plugins need during their build step,
    /// but after build you want to remove it and send it to another thread.
    fn setup(&self, _app: &mut App) {
        // do nothing
    }

    /// Configures a name for the [`Plugin`] which is primarily used for checking plugin
    /// uniqueness and debugging.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// If the plugin can be meaningfully instantiated several times in an [`App`](crate::App),
    /// override this method to return `false`.
    fn is_unique(&self) -> bool {
        true
    }
}

impl_downcast!(Plugin);

/// A type representing an unsafe function that returns a mutable pointer to a [`Plugin`].
/// It is used for dynamically loading plugins.
///
/// See `bevy_dynamic_plugin/src/loader.rs#dynamically_load_plugin`.
pub type CreatePlugin = unsafe fn() -> *mut dyn Plugin;

/// Types that can be converted into a [`Plugin`].
///
/// This is implemented for all types which implement [`Plugin`] or
/// [`FnOnce(&mut App) -> impl Plugin`](FnOnce).
pub trait IntoPlugin<Params>: sealed::IntoPlugin<Params> {}

impl<Params, T> IntoPlugin<Params> for T where T: sealed::IntoPlugin<Params> {}

mod sealed {

    use crate::{App, Plugin};

    pub trait IntoPlugin<Params> {
        type Plugin: Plugin;
        fn into_plugin(self, app: &mut App) -> Self::Plugin;
    }

    pub struct IsPlugin;
    pub struct IsFunction;

    impl<P: Plugin> IntoPlugin<IsPlugin> for P {
        type Plugin = Self;
        fn into_plugin(self, _: &mut App) -> Self {
            self
        }
    }

    impl<F: FnOnce(&mut App) -> P, P: Plugin> IntoPlugin<IsFunction> for F {
        type Plugin = P;

        fn into_plugin(self, app: &mut App) -> Self::Plugin {
            self(app)
        }
    }
}
