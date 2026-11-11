//! Enumerations from the Service Protocol.
//!
//! * Enums the **client sends** (`StepOption`, `ExceptionPauseMode`, …) are closed.
//! * Enums the **VM sends** (`EventKind`, `InstanceKind`, …) carry an `Unknown`
//!   variant: the spec explicitly allows the VM to add values and requires
//!   clients to cope.

use serde::{Deserialize, Serialize};

/// How `resume` should step. See `resume`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StepOption {
    Into,
    Over,
    OverAsyncSuspension,
    Out,
    /// Rewind to a frame (pair with a frame index, see
    /// [`VmService::rewind`](crate::VmService::rewind)).
    Rewind,
}

/// When an isolate pauses on exceptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExceptionPauseMode {
    None,
    Unhandled,
    All,
}

/// Kind of report requested from `getSourceReport`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceReportKind {
    Coverage,
    PossibleBreakpoints,
    BranchCoverage,
}

/// Backing storage of an ID zone (`createIdZone`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdZoneBackingBufferKind {
    Ring,
}

/// How an ID zone assigns temporary ids (`createIdZone`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdAssignmentPolicy {
    AlwaysAllocate,
    ReuseExisting,
}

/// Flags that the spec says may be changed at runtime with `setFlag`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VmFlag {
    PauseIsolatesOnStart,
    PauseIsolatesOnExit,
    PauseIsolatesOnUnhandledExceptions,
    /// Sampling period in microseconds (the VM clamps it to a minimum of 50).
    ProfilePeriod,
    /// Enables / disables the profiler.
    Profiler,
    /// Any other flag name (the VM will reject it if it is not settable).
    Other(String),
}

impl VmFlag {
    pub fn as_str(&self) -> &str {
        match self {
            VmFlag::PauseIsolatesOnStart => "pause_isolates_on_start",
            VmFlag::PauseIsolatesOnExit => "pause_isolates_on_exit",
            VmFlag::PauseIsolatesOnUnhandledExceptions => "pause_isolates_on_unhandled_exceptions",
            VmFlag::ProfilePeriod => "profile_period",
            VmFlag::Profiler => "profiler",
            VmFlag::Other(s) => s,
        }
    }
}

/// Timeline streams accepted by `setVMTimelineFlags`.
///
/// The spec only says "list of timeline stream names"; this set is what a Dart
/// 3.13 VM reports in `TimelineFlags::available_streams`. Use
/// [`TimelineStream::Other`] for names newer VMs may add.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TimelineStream {
    Api,
    Compiler,
    CompilerVerbose,
    Dart,
    Debugger,
    Embedder,
    Gc,
    Isolate,
    Microtask,
    Vm,
    Other(String),
}

impl TimelineStream {
    pub fn as_str(&self) -> &str {
        match self {
            TimelineStream::Api => "API",
            TimelineStream::Compiler => "Compiler",
            TimelineStream::CompilerVerbose => "CompilerVerbose",
            TimelineStream::Dart => "Dart",
            TimelineStream::Debugger => "Debugger",
            TimelineStream::Embedder => "Embedder",
            TimelineStream::Gc => "GC",
            TimelineStream::Isolate => "Isolate",
            TimelineStream::Microtask => "Microtask",
            TimelineStream::Vm => "VM",
            TimelineStream::Other(s) => s,
        }
    }
}

impl From<&str> for TimelineStream {
    fn from(s: &str) -> Self {
        match s {
            "API" => TimelineStream::Api,
            "Compiler" => TimelineStream::Compiler,
            "CompilerVerbose" => TimelineStream::CompilerVerbose,
            "Dart" => TimelineStream::Dart,
            "Debugger" => TimelineStream::Debugger,
            "Embedder" => TimelineStream::Embedder,
            "GC" => TimelineStream::Gc,
            "Isolate" => TimelineStream::Isolate,
            "Microtask" => TimelineStream::Microtask,
            "VM" => TimelineStream::Vm,
            other => TimelineStream::Other(other.to_owned()),
        }
    }
}

impl std::fmt::Display for TimelineStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

macro_rules! open_enum {
    ($(#[$m:meta])* $name:ident { $($variant:ident),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[non_exhaustive]
        pub enum $name {
            $($variant,)+
            /// A value this crate does not know yet.
            #[serde(other)]
            Unknown,
        }
    };
}

open_enum!(
    /// Kind of an [`Event`](super::Event).
    EventKind {
        VMUpdate, VMFlagUpdate,
        IsolateStart, IsolateRunnable, IsolateExit, IsolateUpdate, IsolateReload,
        ServiceExtensionAdded,
        PauseStart, PauseExit, PauseBreakpoint, PauseInterrupted, PauseException,
        PausePostRequest, Resume, None,
        BreakpointAdded, BreakpointResolved, BreakpointRemoved, BreakpointUpdated,
        GC, WriteEvent, Inspect, Extension, Logging, TimerSignificantlyOverdue,
        TimelineEvents, TimelineStreamSubscriptionsUpdate,
        ServiceRegistered, ServiceUnregistered, UserTagChanged, CpuSamples,
    }
);

impl EventKind {
    /// `true` for every `Pause*` kind.
    pub fn is_pause(self) -> bool {
        matches!(
            self,
            EventKind::PauseStart
                | EventKind::PauseExit
                | EventKind::PauseBreakpoint
                | EventKind::PauseInterrupted
                | EventKind::PauseException
                | EventKind::PausePostRequest
        )
    }
}

open_enum!(
    /// Kind of an [`InstanceRef`](super::InstanceRef).
    InstanceKind {
        PlainInstance, Null, Bool, Double, Int, String, List, Map, Set,
        Float32x4, Float64x2, Int32x4,
        Uint8ClampedList, Uint8List, Uint16List, Uint32List, Uint64List,
        Int8List, Int16List, Int32List, Int64List, Float32List, Float64List,
        Int32x4List, Float32x4List, Float64x2List,
        Record, StackTrace, Closure, MirrorReference, RegExp,
        WeakProperty, WeakReference,
        Type, TypeParameter, TypeRef, FunctionType, RecordType, BoundedType,
        ReceivePort, UserTag, Finalizer, NativeFinalizer, FinalizerEntry, Pointer,
    }
);

open_enum!(
    /// Why a [`Sentinel`](super::Sentinel) was returned.
    SentinelKind {
        Collected, Expired, NotInitialized, BeingInitialized, OptimizedOut, Free,
    }
);

open_enum!(ErrorKind {
    UnhandledException,
    LanguageError,
    InternalError,
    TerminationError
});
open_enum!(CodeKind {
    Dart,
    Native,
    Stub,
    Tag,
    Collected
});
open_enum!(FrameKind {
    Regular,
    AsyncCausal,
    AsyncSuspensionMarker,
    AsyncActivation
});
