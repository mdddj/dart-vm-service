//! Option structs for RPCs with several optional parameters.
//!
//! All of them implement `Default`, so you can write
//! `EvalOptions::default().disable_breakpoints(true)` or use struct-update syntax.

use std::collections::BTreeMap;

use crate::params::Params;

macro_rules! setters {
    ($ty:ident { $($field:ident : $t:ty),+ $(,)? }) => {
        impl $ty {
            $(
                #[doc = concat!("Sets `", stringify!($field), "`.")]
                pub fn $field(mut self, value: impl Into<$t>) -> Self {
                    self.$field = Some(value.into());
                    self
                }
            )+
        }
    };
}

/// Options for [`VmService::invoke`](crate::VmService::invoke).
#[derive(Debug, Clone, Default)]
pub struct InvokeOptions {
    /// Ignore breakpoints hit while invoking.
    pub disable_breakpoints: Option<bool>,
    /// ID zone in which temporary ids of the response are allocated.
    pub id_zone_id: Option<String>,
}
setters!(InvokeOptions {
    disable_breakpoints: bool,
    id_zone_id: String
});

impl InvokeOptions {
    pub(crate) fn apply(&self, p: Params) -> Params {
        p.opt("disableBreakpoints", self.disable_breakpoints)
            .opt("idZoneId", self.id_zone_id.as_ref())
    }
}

/// Options for `evaluate` / `evaluateInFrame`.
#[derive(Debug, Clone, Default)]
pub struct EvalOptions {
    /// Extra bindings, identifier → object id.
    pub scope: Option<BTreeMap<String, String>>,
    pub disable_breakpoints: Option<bool>,
    pub id_zone_id: Option<String>,
}
setters!(EvalOptions {
    scope: BTreeMap<String, String>,
    disable_breakpoints: bool,
    id_zone_id: String,
});

impl EvalOptions {
    /// Adds one `identifier → object id` binding to the scope.
    pub fn bind(mut self, name: impl Into<String>, object_id: impl Into<String>) -> Self {
        self.scope
            .get_or_insert_with(Default::default)
            .insert(name.into(), object_id.into());
        self
    }

    pub(crate) fn apply(&self, p: Params) -> Params {
        p.opt("scope", self.scope.as_ref())
            .opt("disableBreakpoints", self.disable_breakpoints)
            .opt("idZoneId", self.id_zone_id.as_ref())
    }
}

/// Options for `getAllocationProfile`.
#[derive(Debug, Clone, Default)]
pub struct AllocationProfileOptions {
    /// Reset the allocation accumulators first.
    pub reset: Option<bool>,
    /// Attempt a GC first (not guaranteed to run).
    pub gc: Option<bool>,
}
setters!(AllocationProfileOptions {
    reset: bool,
    gc: bool
});

impl AllocationProfileOptions {
    pub(crate) fn apply(&self, p: Params) -> Params {
        p.opt("reset", self.reset).opt("gc", self.gc)
    }
}

/// Options for `getInstances` / `getInstancesAsList`.
#[derive(Debug, Clone, Default)]
pub struct InstancesOptions {
    pub include_subclasses: Option<bool>,
    pub include_implementers: Option<bool>,
    pub id_zone_id: Option<String>,
}
setters!(InstancesOptions {
    include_subclasses: bool,
    include_implementers: bool,
    id_zone_id: String,
});

impl InstancesOptions {
    pub(crate) fn apply(&self, p: Params) -> Params {
        p.opt("includeSubclasses", self.include_subclasses)
            .opt("includeImplementers", self.include_implementers)
            .opt("idZoneId", self.id_zone_id.as_ref())
    }
}

/// Options for `getObject` (paging of lists/maps/strings).
#[derive(Debug, Clone, Default)]
pub struct GetObjectOptions {
    pub offset: Option<i64>,
    pub count: Option<i64>,
    pub id_zone_id: Option<String>,
}
setters!(GetObjectOptions {
    offset: i64,
    count: i64,
    id_zone_id: String
});

impl GetObjectOptions {
    pub(crate) fn apply(&self, p: Params) -> Params {
        p.opt("offset", self.offset)
            .opt("count", self.count)
            .opt("idZoneId", self.id_zone_id.as_ref())
    }
}

/// Options for `getSourceReport` (besides the report kinds).
#[derive(Debug, Clone, Default)]
pub struct SourceReportOptions {
    pub script_id: Option<String>,
    pub token_pos: Option<i64>,
    pub end_token_pos: Option<i64>,
    pub force_compile: Option<bool>,
    pub report_lines: Option<bool>,
    pub library_filters: Option<Vec<String>>,
    pub libraries_already_compiled: Option<Vec<String>>,
}
setters!(SourceReportOptions {
    script_id: String,
    token_pos: i64,
    end_token_pos: i64,
    force_compile: bool,
    report_lines: bool,
    library_filters: Vec<String>,
    libraries_already_compiled: Vec<String>,
});

impl SourceReportOptions {
    pub(crate) fn apply(&self, p: Params) -> Params {
        p.opt("scriptId", self.script_id.as_ref())
            .opt("tokenPos", self.token_pos)
            .opt("endTokenPos", self.end_token_pos)
            .opt("forceCompile", self.force_compile)
            .opt("reportLines", self.report_lines)
            .opt("libraryFilters", self.library_filters.as_ref())
            .opt(
                "librariesAlreadyCompiled",
                self.libraries_already_compiled.as_ref(),
            )
    }
}

/// Options for `reloadSources` (hot reload).
#[derive(Debug, Clone, Default)]
pub struct ReloadOptions {
    /// Reload even unchanged sources.
    pub force: Option<bool>,
    /// Pause the isolate after the reload.
    pub pause: Option<bool>,
    pub root_lib_uri: Option<String>,
    pub packages_uri: Option<String>,
}
setters!(ReloadOptions {
    force: bool,
    pause: bool,
    root_lib_uri: String,
    packages_uri: String
});

impl ReloadOptions {
    pub(crate) fn apply(&self, p: Params) -> Params {
        p.opt("force", self.force)
            .opt("pause", self.pause)
            .opt("rootLibUri", self.root_lib_uri.as_ref())
            .opt("packagesUri", self.packages_uri.as_ref())
    }
}
