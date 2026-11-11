//! Isolate-level RPCs: info, lifecycle, hot reload, memory.

use crate::client::VmService;
use crate::error::Result;
use crate::method::RpcMethod::*;
use crate::options::ReloadOptions;
use crate::params::Params;
use crate::types::*;

fn isolate(id: &str) -> Params {
    Params::new().with("isolateId", id)
}

impl VmService {
    /// Details of an isolate (`getIsolate`).
    pub async fn get_isolate(&self, isolate_id: &str) -> Result<Isolate> {
        self.request(GetIsolate, isolate(isolate_id)).await
    }

    /// Details of an isolate group (`getIsolateGroup`).
    pub async fn get_isolate_group(&self, isolate_group_id: &str) -> Result<IsolateGroup> {
        self.request(
            GetIsolateGroup,
            Params::new().with("isolateGroupId", isolate_group_id),
        )
        .await
    }

    /// The event that paused the isolate, or a `None` event (`getIsolatePauseEvent`).
    pub async fn get_isolate_pause_event(&self, isolate_id: &str) -> Result<Event> {
        self.request(GetIsolatePauseEvent, isolate(isolate_id))
            .await
    }

    /// Heap usage of one isolate (`getMemoryUsage`).
    pub async fn get_memory_usage(&self, isolate_id: &str) -> Result<MemoryUsage> {
        self.request(GetMemoryUsage, isolate(isolate_id)).await
    }

    /// Heap usage of an isolate group (`getIsolateGroupMemoryUsage`).
    pub async fn get_isolate_group_memory_usage(
        &self,
        isolate_group_id: &str,
    ) -> Result<MemoryUsage> {
        self.request(
            GetIsolateGroupMemoryUsage,
            Params::new().with("isolateGroupId", isolate_group_id),
        )
        .await
    }

    /// Open ports of the isolate (`getPorts`).
    pub async fn get_ports(&self, isolate_id: &str) -> Result<PortList> {
        self.request(GetPorts, isolate(isolate_id)).await
    }

    /// Microtasks queued in the isolate (`getQueuedMicrotasks`).
    pub async fn get_queued_microtasks(&self, isolate_id: &str) -> Result<QueuedMicrotasks> {
        self.request(GetQueuedMicrotasks, isolate(isolate_id)).await
    }

    /// Renames the isolate (`setName`).
    pub async fn set_name(&self, isolate_id: &str, name: &str) -> Result<()> {
        self.request_unit(SetName, isolate(isolate_id).with("name", name))
            .await
    }

    /// Terminates the isolate (`kill`).
    pub async fn kill(&self, isolate_id: &str) -> Result<()> {
        self.request_unit(Kill, isolate(isolate_id)).await
    }

    /// Hot reload (`reloadSources`).
    ///
    /// ```no_run
    /// # async fn f(vm: dart_vm_service::VmService, id: &str) -> dart_vm_service::Result<()> {
    /// use dart_vm_service::ReloadOptions;
    /// let report = vm.reload_sources(id, ReloadOptions::default().force(true)).await?;
    /// assert!(report.success);
    /// # Ok(()) }
    /// ```
    pub async fn reload_sources(
        &self,
        isolate_id: &str,
        options: ReloadOptions,
    ) -> Result<ReloadReport> {
        self.request(ReloadSources, options.apply(isolate(isolate_id)))
            .await
    }

    /// Asks the isolate to produce a heap snapshot, delivered on the
    /// `HeapSnapshot` stream as binary events (`requestHeapSnapshot`).
    pub async fn request_heap_snapshot(&self, isolate_id: &str) -> Result<()> {
        self.request_unit(RequestHeapSnapshot, isolate(isolate_id))
            .await
    }

    /// Controls pausing on exceptions and at exit (`setIsolatePauseMode`).
    pub async fn set_isolate_pause_mode(
        &self,
        isolate_id: &str,
        exception_pause_mode: Option<ExceptionPauseMode>,
        should_pause_on_exit: Option<bool>,
    ) -> Result<()> {
        self.request_unit(
            SetIsolatePauseMode,
            isolate(isolate_id)
                .opt("exceptionPauseMode", exception_pause_mode)
                .opt("shouldPauseOnExit", should_pause_on_exit),
        )
        .await
    }

    /// Deprecated by the spec; use [`set_isolate_pause_mode`](Self::set_isolate_pause_mode).
    #[deprecated(note = "use set_isolate_pause_mode")]
    pub async fn set_exception_pause_mode(
        &self,
        isolate_id: &str,
        mode: ExceptionPauseMode,
    ) -> Result<()> {
        self.request_unit(
            SetExceptionPauseMode,
            isolate(isolate_id).with("mode", mode),
        )
        .await
    }

    // ---- ID zones --------------------------------------------------------

    /// Creates a zone for temporary object ids (`createIdZone`).
    pub async fn create_id_zone(
        &self,
        isolate_id: &str,
        backing_buffer_kind: IdZoneBackingBufferKind,
        id_assignment_policy: IdAssignmentPolicy,
        capacity: Option<i64>,
    ) -> Result<IdZone> {
        self.request(
            CreateIdZone,
            isolate(isolate_id)
                .with("backingBufferKind", backing_buffer_kind)
                .with("idAssignmentPolicy", id_assignment_policy)
                .opt("capacity", capacity),
        )
        .await
    }

    /// Expires every id allocated in the zone but keeps the zone (`invalidateIdZone`).
    pub async fn invalidate_id_zone(&self, isolate_id: &str, id_zone_id: &str) -> Result<()> {
        self.request_unit(
            InvalidateIdZone,
            isolate(isolate_id).with("idZoneId", id_zone_id),
        )
        .await
    }

    /// Frees the zone for good (`deleteIdZone`).
    pub async fn delete_id_zone(&self, isolate_id: &str, id_zone_id: &str) -> Result<()> {
        self.request_unit(
            DeleteIdZone,
            isolate(isolate_id).with("idZoneId", id_zone_id),
        )
        .await
    }
}
