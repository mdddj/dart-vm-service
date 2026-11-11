//! The set of public RPC methods defined by the Dart VM Service Protocol.

macro_rules! rpc_methods {
    ($( $(#[$m:meta])* $variant:ident => $wire:literal ),+ $(,)?) => {
        /// Every public RPC of the VM Service Protocol.
        ///
        /// Using the enum with [`VmService::call_method`](crate::VmService::call_method)
        /// avoids typos in method names; the typed wrappers on
        /// [`VmService`](crate::VmService) use it internally.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum RpcMethod {
            $( $(#[$m])* $variant, )+
        }

        impl RpcMethod {
            /// All methods, handy for tooling and tests.
            pub const ALL: &'static [RpcMethod] = &[ $( RpcMethod::$variant, )+ ];

            /// The method name as sent on the wire.
            pub const fn as_str(self) -> &'static str {
                match self { $( RpcMethod::$variant => $wire, )+ }
            }
        }
    };
}

rpc_methods! {
    AddBreakpoint => "addBreakpoint",
    AddBreakpointWithScriptUri => "addBreakpointWithScriptUri",
    AddBreakpointAtEntry => "addBreakpointAtEntry",
    ClearCpuSamples => "clearCpuSamples",
    ClearVmTimeline => "clearVMTimeline",
    CreateIdZone => "createIdZone",
    DeleteIdZone => "deleteIdZone",
    InvalidateIdZone => "invalidateIdZone",
    Invoke => "invoke",
    Evaluate => "evaluate",
    EvaluateInFrame => "evaluateInFrame",
    GetAllocationProfile => "getAllocationProfile",
    GetAllocationTraces => "getAllocationTraces",
    GetClassList => "getClassList",
    GetCpuSamples => "getCpuSamples",
    GetFlagList => "getFlagList",
    GetInboundReferences => "getInboundReferences",
    GetInstances => "getInstances",
    GetInstancesAsList => "getInstancesAsList",
    GetIsolate => "getIsolate",
    GetIsolateGroup => "getIsolateGroup",
    GetIsolatePauseEvent => "getIsolatePauseEvent",
    GetMemoryUsage => "getMemoryUsage",
    GetIsolateGroupMemoryUsage => "getIsolateGroupMemoryUsage",
    GetScripts => "getScripts",
    GetObject => "getObject",
    GetPerfettoCpuSamples => "getPerfettoCpuSamples",
    GetPerfettoVmTimeline => "getPerfettoVMTimeline",
    GetPorts => "getPorts",
    GetRetainingPath => "getRetainingPath",
    GetProcessMemoryUsage => "getProcessMemoryUsage",
    GetQueuedMicrotasks => "getQueuedMicrotasks",
    GetStack => "getStack",
    GetSupportedProtocols => "getSupportedProtocols",
    GetSourceReport => "getSourceReport",
    GetVersion => "getVersion",
    GetVm => "getVM",
    GetVmTimeline => "getVMTimeline",
    GetVmTimelineFlags => "getVMTimelineFlags",
    GetVmTimelineMicros => "getVMTimelineMicros",
    Pause => "pause",
    Kill => "kill",
    LookupResolvedPackageUris => "lookupResolvedPackageUris",
    LookupPackageUris => "lookupPackageUris",
    RegisterService => "registerService",
    ReloadSources => "reloadSources",
    RemoveBreakpoint => "removeBreakpoint",
    RequestHeapSnapshot => "requestHeapSnapshot",
    Resume => "resume",
    SetBreakpointState => "setBreakpointState",
    /// Deprecated by the spec in favour of [`RpcMethod::SetIsolatePauseMode`].
    SetExceptionPauseMode => "setExceptionPauseMode",
    SetIsolatePauseMode => "setIsolatePauseMode",
    SetFlag => "setFlag",
    SetLibraryDebuggable => "setLibraryDebuggable",
    SetName => "setName",
    SetTraceClassAllocation => "setTraceClassAllocation",
    SetVmName => "setVMName",
    SetVmTimelineFlags => "setVMTimelineFlags",
    StreamCancel => "streamCancel",
    /// Deprecated by the spec; calling it has no effect.
    StreamCpuSamplesWithUserTag => "streamCpuSamplesWithUserTag",
    StreamListen => "streamListen",
}

impl std::fmt::Display for RpcMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
