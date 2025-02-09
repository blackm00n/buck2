/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! Represents the forward and backward dependencies of the computation graph

use std::any::Any;
use std::collections::hash_map::Entry;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;
use std::sync::Weak;

use allocative::Allocative;
use async_trait::async_trait;
use dupe::Dupe;
use gazebo::cmp::PartialEqAny;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;

use crate::api::error::DiceResult;
use crate::introspection::graph::AnyKey;
use crate::legacy::ctx::ComputationData;
use crate::legacy::incremental::graph::GraphNodeDyn;
use crate::legacy::incremental::graph::ReadOnlyHistory;
use crate::legacy::incremental::transaction_ctx::TransactionCtx;
use crate::legacy::incremental::versions::MinorVersion;
use crate::versions::VersionNumber;
use crate::HashMap;

/// The dependency information stored by the core engine
#[async_trait]
pub(crate) trait Dependency: Allocative + Debug + Display + Send + Sync {
    async fn recompute(
        &self,
        transaction_ctx: &Arc<TransactionCtx>,
        extra: &ComputationData,
    ) -> DiceResult<(Box<dyn ComputedDependency>, Arc<dyn GraphNodeDyn>)>;

    /// looks up the stored node of this dependency. This can return `None` if this entry
    /// was evicted from the storage.
    fn lookup_node(&self, v: VersionNumber, mv: MinorVersion) -> Option<Arc<dyn GraphNodeDyn>>;

    fn dirty(&self, v: VersionNumber);

    fn get_key_equality(&self) -> PartialEqAny;

    fn to_key_any(&self) -> &dyn Any;

    fn hash(&self, state: &mut dyn Hasher);

    /// Provide a type-erased AnyKey representing this Dependency. This is used when traversing
    /// DICE to dump its state.
    fn introspect(&self) -> AnyKey;
}

impl PartialEq for dyn Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.get_key_equality() == other.get_key_equality()
    }
}

impl Eq for dyn Dependency {}

impl Hash for dyn Dependency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

/// The dependency information right after they were requested. This allows us to look up
/// information from the dependency without further computation.
pub(crate) trait ComputedDependency: Allocative + Debug + Send + Sync {
    fn get_history(&self) -> ReadOnlyHistory;

    /// converts itself into the data to be stored in deps and rdeps
    fn into_dependency(self: Box<Self>) -> Box<dyn Dependency>;

    fn get_key_equality(&self) -> (PartialEqAny, VersionNumber);

    fn to_key_any(&self) -> &dyn Any;

    fn hash(&self, state: &mut dyn Hasher);

    fn is_valid(&self) -> bool;
}

impl PartialEq for dyn ComputedDependency {
    fn eq(&self, other: &Self) -> bool {
        self.get_key_equality() == other.get_key_equality()
    }
}

impl Eq for dyn ComputedDependency {}

impl Hash for dyn ComputedDependency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

#[derive(Allocative)]
pub(crate) struct VersionedDependencies {
    /// once the deps at a particular version is written, it is final and never modified
    /// We only store the dependencies relevant to the most recent result
    deps: RwLock<Option<(VersionNumber, Arc<Vec<Box<dyn Dependency>>>)>>,
}

impl VersionedDependencies {
    pub(crate) fn new() -> Self {
        Self {
            deps: RwLock::new(None),
        }
    }

    pub(crate) fn deps(&self) -> Option<Arc<Vec<Box<dyn Dependency>>>> {
        self.deps.read().as_ref().map(|d| d.1.dupe())
    }

    pub(crate) fn add_deps(&self, v: VersionNumber, deps: Arc<Vec<Box<dyn Dependency>>>) {
        let mut this_deps = self.deps.write();
        if this_deps.as_ref().map_or(true, |d| v > d.0) {
            // we only ever write the newest version of the dependencies of this node for simplicity
            // That way, if we are ever dirtied, we just check if the latest version of the deps
            // have changed at the dirtied version which only requires spawning one set of deps.
            // It might cause us to falsely fail to reuse some nodes, but this is less memory
            // and less work per node when in incremental cases.
            *this_deps = Some((v, deps));
        }
    }

    pub(crate) fn debug_deps(
        &self,
    ) -> &RwLock<Option<(VersionNumber, Arc<Vec<Box<dyn Dependency>>>)>> {
        &self.deps
    }
}

/// Eq and Hash for an rdep is related to the address of the node it points to, since in a dice
/// session, the node stored is always kept alive via an `Arc`, node equality is the ptr address
#[derive(Clone, Dupe, Allocative)]
#[repr(transparent)]
pub(crate) struct Rdep(pub(crate) Weak<dyn GraphNodeDyn>);

impl PartialEq for Rdep {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Rdep {}

impl Hash for Rdep {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.upgrade().map(|p| Arc::as_ptr(&p)).hash(state)
    }
}

// the set of reverse dependencies of a node
#[derive(Clone, Dupe, Allocative)]
pub(crate) struct VersionedRevDependencies {
    data: Arc<RwLock<VersionedRevDependenciesData>>,
}

#[derive(Allocative)]
pub(crate) struct VersionedRevDependenciesData {
    // TODO(bobyf) do we need something special for quick lookup per version or is this fine
    pub(crate) rdeps: HashMap<Rdep, VersionNumber>,
}

impl VersionedRevDependencies {
    pub(crate) fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(VersionedRevDependenciesData {
                rdeps: Default::default(),
            })),
        }
    }

    pub(crate) fn add_rdep(
        &self,
        dependent: Weak<dyn GraphNodeDyn>,
        current_version: VersionNumber,
    ) {
        let mut data = self.data.write();

        match data.rdeps.entry(Rdep(dependent)) {
            Entry::Occupied(entry) => {
                if *entry.get() < current_version {
                    entry.replace_entry(current_version);
                }
            }
            Entry::Vacant(v) => {
                v.insert(current_version);
            }
        }
    }

    pub(crate) fn rdeps(&self) -> RwLockReadGuard<VersionedRevDependenciesData> {
        self.data.read()
    }
}
