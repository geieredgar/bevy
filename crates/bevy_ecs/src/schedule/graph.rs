use bevy_ecs_macros::all_tuples;

use crate::system::{BoxedSystem, IntoSystem, System};

use super::{
    graph_utils::{Ambiguity, Dependency, DependencyKind, GraphInfo},
    BoxedCondition, BoxedSystemSet, Condition, IntoSystemSet, SystemSet,
};

pub struct SystemGraph {
    pub(crate) graph_info: GraphInfo,
    pub(crate) graph_type: SystemGraphType,
}

pub(crate) enum SystemGraphType {
    System {
        system: BoxedSystem,
        conditions: Vec<BoxedCondition>,
    },
    Set {
        set: BoxedSystemSet,
        conditions: Vec<BoxedCondition>,
    },
    AnonymousSet {
        members: Vec<SystemGraph>,
        chained: bool,
        conditions: Vec<BoxedCondition>,
    },
    Collection {
        members: Vec<SystemGraph>,
        chained: bool,
    },
}

impl SystemGraph {
    fn insert_dependency(&mut self, kind: DependencyKind, graph: SystemGraph) {
        self.graph_info
            .dependencies
            .push(Dependency::new(kind, graph))
    }

    fn contains_system(&self) -> bool {
        match &self.graph_type {
            SystemGraphType::System { .. } => true,
            SystemGraphType::Set { .. } | SystemGraphType::AnonymousSet { .. } => false,
            SystemGraphType::Collection { members, .. } => {
                members.iter().any(Self::contains_system)
            }
        }
    }

    fn contains_system_type_set(&self) -> bool {
        match &self.graph_type {
            SystemGraphType::System { .. } | SystemGraphType::AnonymousSet { .. } => false,
            SystemGraphType::Set { set, .. } => set.is_system_type(),
            SystemGraphType::Collection { members, .. } => {
                members.iter().any(Self::contains_system_type_set)
            }
        }
    }

    fn contains_base_set(&self) -> bool {
        match &self.graph_type {
            SystemGraphType::System { .. } | SystemGraphType::AnonymousSet { .. } => false,
            SystemGraphType::Set { set, .. } => set.is_base(),
            SystemGraphType::Collection { members, .. } => {
                members.iter().any(Self::contains_base_set)
            }
        }
    }

    fn distribute_run_condition<M>(&mut self, condition: impl Condition<M> + Clone) {
        match &mut self.graph_type {
            SystemGraphType::System { conditions, .. }
            | SystemGraphType::Set { conditions, .. }
            | SystemGraphType::AnonymousSet { conditions, .. } => {
                conditions.push(new_condition(condition));
            }
            SystemGraphType::Collection { members, .. } => {
                for member in members {
                    member.distribute_run_condition(condition.clone());
                }
            }
        }
    }
}

pub trait IntoSystemGraph<P>: sealed::IntoSystemGraph<P> + Sized {
    fn into_graph(self) -> SystemGraph;

    fn before<M>(self, graph: impl IntoSystemGraph<M>) -> SystemGraph {
        self.into_graph().before(graph)
    }

    fn after<M>(self, graph: impl IntoSystemGraph<M>) -> SystemGraph {
        self.into_graph().after(graph)
    }

    fn in_set<M>(self, graph: impl IntoSystemGraph<M>) -> SystemGraph {
        self.into_graph().in_set(graph)
    }

    fn in_base_set(self, set: impl SystemSet) -> SystemGraph {
        self.into_graph().in_base_set(set)
    }

    fn in_default_base_set(self) -> SystemGraph {
        self.into_graph().in_default_base_set()
    }

    fn no_default_base_set(self) -> SystemGraph {
        self.into_graph().no_default_base_set()
    }

    fn run_if<M>(self, condition: impl Condition<M>) -> SystemGraph {
        self.into_graph().run_if(condition)
    }

    fn distributive_run_if<M>(self, condition: impl Condition<M> + Clone) -> SystemGraph {
        self.into_graph().distributive_run_if(condition)
    }

    fn into_set(self) -> SystemGraph {
        self.into_graph().into_set()
    }

    fn chain(self) -> SystemGraph {
        self.into_graph().chain()
    }

    fn ambiguous_with<M>(self, set: impl IntoSystemSet<M>) -> SystemGraph {
        self.into_graph().ambiguous_with(set)
    }

    fn ambiguous_with_all(self) -> SystemGraph {
        self.into_graph().ambiguous_with_all()
    }
}

impl IntoSystemGraph<()> for SystemGraph {
    fn into_graph(self) -> SystemGraph {
        self
    }

    fn before<M>(mut self, graph: impl IntoSystemGraph<M>) -> SystemGraph {
        self.insert_dependency(DependencyKind::Before, graph.into_graph());
        self
    }

    fn after<M>(mut self, graph: impl IntoSystemGraph<M>) -> SystemGraph {
        self.insert_dependency(DependencyKind::After, graph.into_graph());
        self
    }

    fn in_set<M>(mut self, graph: impl IntoSystemGraph<M>) -> SystemGraph {
        let graph = graph.into_graph();
        assert!(
            !graph.contains_system_type_set(),
            "adding arbitrary systems to a system type set is not allowed"
        );
        assert!(
            !graph.contains_base_set(),
            "Systems cannot be added to 'base' system sets using 'in_set'. Use 'in_base_set' instead."
        );
        assert!(
            !self.contains_base_set(),
            "Base system sets cannot be added to other sets."
        );
        assert!(!graph.contains_system(), "");
        self.graph_info.sets.push(graph);
        self
    }

    fn in_base_set(mut self, set: impl SystemSet) -> SystemGraph {
        assert!(
            !set.is_system_type(),
            "System type sets cannot be base sets."
        );
        assert!(
            set.is_base(),
            "Systems cannot be added to normal sets using 'in_base_set'. Use 'in_set' instead."
        );
        self.graph_info.set_base_set(Box::new(set));
        self
    }

    fn in_default_base_set(mut self) -> SystemGraph {
        self.graph_info.add_default_base_set = true;
        self
    }

    fn no_default_base_set(mut self) -> SystemGraph {
        self.graph_info.add_default_base_set = false;
        self
    }

    fn run_if<M>(mut self, condition: impl Condition<M>) -> SystemGraph {
        match &mut self.graph_type {
            SystemGraphType::System { conditions, .. }
            | SystemGraphType::Set { conditions, .. }
            | SystemGraphType::AnonymousSet { conditions, .. } => {
                conditions.push(new_condition(condition));
            }
            SystemGraphType::Collection { .. } => panic!(),
        }
        self
    }

    fn distributive_run_if<M>(mut self, condition: impl Condition<M> + Clone) -> SystemGraph {
        match &mut self.graph_type {
            SystemGraphType::System { .. }
            | SystemGraphType::Set { .. }
            | SystemGraphType::AnonymousSet { .. } => {
                panic!()
            }
            SystemGraphType::Collection { .. } => {}
        }
        self.distribute_run_condition(condition);
        self
    }

    fn into_set(self) -> SystemGraph {
        match self.graph_type {
            SystemGraphType::System { .. } => SystemGraph {
                graph_info: GraphInfo::system_set(),
                graph_type: SystemGraphType::AnonymousSet {
                    members: vec![self],
                    chained: false,
                    conditions: Vec::new(),
                },
            },
            SystemGraphType::Set { .. } | SystemGraphType::AnonymousSet { .. } => panic!(),
            SystemGraphType::Collection { members, chained } => SystemGraph {
                graph_info: self.graph_info,
                graph_type: SystemGraphType::AnonymousSet {
                    members,
                    chained,
                    conditions: Vec::new(),
                },
            },
        }
    }

    fn chain(mut self) -> SystemGraph {
        match &mut self.graph_type {
            SystemGraphType::System { .. } | SystemGraphType::Set { .. } => panic!(),
            SystemGraphType::AnonymousSet { chained, .. }
            | SystemGraphType::Collection { chained, .. } => {
                *chained = true;
            }
        }
        self
    }

    fn ambiguous_with<M>(mut self, set: impl IntoSystemSet<M>) -> SystemGraph {
        ambiguous_with(&mut self.graph_info, Box::new(set.into_system_set()));
        self
    }

    fn ambiguous_with_all(mut self) -> SystemGraph {
        self.graph_info.ambiguous_with = Ambiguity::IgnoreAll;
        self
    }
}

mod sealed {
    use std::marker::PhantomData;

    use crate::{
        schedule::{BoxedSystemSet, SystemSet},
        system::{BoxedSystem, IntoSystem},
    };

    use super::SystemGraph;

    pub trait IntoSystemGraph<P> {}

    impl IntoSystemGraph<()> for SystemGraph {}

    pub struct IsSystem<P>(PhantomData<P>);
    pub struct IsSystemSet;

    impl<Params, F: IntoSystem<(), (), Params>> IntoSystemGraph<IsSystem<Params>> for F {}

    impl IntoSystemGraph<()> for BoxedSystem<(), ()> {}

    impl IntoSystemGraph<()> for BoxedSystemSet {}

    impl<S: SystemSet> IntoSystemGraph<IsSystemSet> for S {}
}

impl<Params, F> IntoSystemGraph<sealed::IsSystem<Params>> for F
where
    F: IntoSystem<(), (), Params>,
{
    fn into_graph(self) -> SystemGraph {
        SystemGraph {
            graph_info: GraphInfo::system(),
            graph_type: SystemGraphType::System {
                system: Box::new(IntoSystem::into_system(self)),
                conditions: Vec::new(),
            },
        }
    }
}

impl IntoSystemGraph<()> for BoxedSystem<(), ()> {
    fn into_graph(self) -> SystemGraph {
        SystemGraph {
            graph_info: GraphInfo::system(),
            graph_type: SystemGraphType::System {
                system: self,
                conditions: Vec::new(),
            },
        }
    }
}

impl IntoSystemGraph<()> for BoxedSystemSet {
    fn into_graph(self) -> SystemGraph {
        SystemGraph {
            graph_info: GraphInfo::system_set(),
            graph_type: SystemGraphType::Set {
                set: self,
                conditions: Vec::new(),
            },
        }
    }
}

impl<S> IntoSystemGraph<sealed::IsSystemSet> for S
where
    S: SystemSet,
{
    fn into_graph(self) -> SystemGraph {
        SystemGraph {
            graph_info: GraphInfo::system_set(),
            graph_type: SystemGraphType::Set {
                set: Box::new(self),
                conditions: Vec::new(),
            },
        }
    }
}

fn new_condition<P>(condition: impl Condition<P>) -> BoxedCondition {
    let condition_system = IntoSystem::into_system(condition);
    assert!(
        condition_system.is_send(),
        "Condition `{}` accesses thread-local resources. This is not currently supported.",
        condition_system.name()
    );

    Box::new(condition_system)
}

fn ambiguous_with(graph_info: &mut GraphInfo, set: BoxedSystemSet) {
    match &mut graph_info.ambiguous_with {
        detection @ Ambiguity::Check => {
            *detection = Ambiguity::IgnoreWithSet(vec![set]);
        }
        Ambiguity::IgnoreWithSet(ambiguous_with) => {
            ambiguous_with.push(set);
        }
        Ambiguity::IgnoreAll => (),
    }
}

macro_rules! impl_system_graph_collection {
    ($(($param: ident, $sys: ident)),*) => {
        impl<$($param, $sys),*> sealed::IntoSystemGraph<($($param,)*)> for ($($sys,)*)
        where
            $($sys: IntoSystemGraph<$param>),*
        {
        }

        impl<$($param, $sys),*> IntoSystemGraph<($($param,)*)> for ($($sys,)*)
        where
            $($sys: IntoSystemGraph<$param>),*
        {
            #[allow(non_snake_case)]
            fn into_graph(self) -> SystemGraph {
                let ($($sys,)*) = self;
                SystemGraph {
                    graph_info: GraphInfo::system_set(),
                    graph_type: SystemGraphType::Collection {
                        members: vec![$($sys.into_graph(),)*],
                        chained: false,
                    }
                }
            }
        }
    }
}

all_tuples!(impl_system_graph_collection, 0, 15, P, S);
