//! VM-wide RPCs (no isolate needed) plus a few convenience lookups.

use crate::client::VmService;
use crate::error::{Error, Result};
use crate::method::RpcMethod::*;
use crate::params::Params;
use crate::types::*;

impl VmService {
    /// Protocol version of the VM (`getVersion`).
    pub async fn get_version(&self) -> Result<Version> {
        self.request(GetVersion, Params::new()).await
    }

    /// Protocols supported, including middleware such as DDS (`getSupportedProtocols`).
    pub async fn get_supported_protocols(&self) -> Result<ProtocolList> {
        self.request(GetSupportedProtocols, Params::new()).await
    }

    /// VM description with its isolates (`getVM`).
    pub async fn get_vm(&self) -> Result<Vm> {
        self.request(GetVm, Params::new()).await
    }

    /// Renames the VM (`setVMName`).
    pub async fn set_vm_name(&self, name: &str) -> Result<()> {
        self.request_unit(SetVmName, Params::new().with("name", name))
            .await
    }

    /// All VM command line flags and their values (`getFlagList`).
    pub async fn get_flag_list(&self) -> Result<FlagList> {
        self.request(GetFlagList, Params::new()).await
    }

    /// Changes a VM flag at runtime (`setFlag`). Only the flags in [`VmFlag`] are
    /// settable according to the spec.
    ///
    /// ```no_run
    /// # async fn f(vm: dart_vm_service::VmService) -> dart_vm_service::Result<()> {
    /// use dart_vm_service::VmFlag;
    /// vm.set_flag(VmFlag::Profiler, true).await?;
    /// vm.set_flag(VmFlag::ProfilePeriod, 250).await?;
    /// # Ok(()) }
    /// ```
    pub async fn set_flag(&self, flag: VmFlag, value: impl ToString) -> Result<()> {
        self.request_unit(
            SetFlag,
            Params::new()
                .with("name", flag.as_str())
                .with("value", value.to_string()),
        )
        .await
    }

    /// Memory of the whole process (`getProcessMemoryUsage`).
    pub async fn get_process_memory_usage(&self) -> Result<ProcessMemoryUsage> {
        self.request(GetProcessMemoryUsage, Params::new()).await
    }

    /// Advertises a service other clients can call (`registerService`).
    ///
    /// Note: this crate does not host service handlers; incoming calls to your
    /// service are answered with "method not found".
    pub async fn register_service(&self, service: &str, alias: &str) -> Result<()> {
        self.request_unit(
            RegisterService,
            Params::new().with("service", service).with("alias", alias),
        )
        .await
    }

    // ---- timeline --------------------------------------------------------

    /// Current value of the VM's monotonic microsecond clock (`getVMTimelineMicros`).
    pub async fn get_vm_timeline_micros(&self) -> Result<Timestamp> {
        self.request(GetVmTimelineMicros, Params::new()).await
    }

    /// Chrome-format timeline events, optionally within a time window (`getVMTimeline`).
    pub async fn get_vm_timeline(&self, range: Option<TimeRange>) -> Result<Timeline> {
        self.request(GetVmTimeline, time_range(Params::new(), range))
            .await
    }

    /// Perfetto-format timeline (`getPerfettoVMTimeline`).
    pub async fn get_perfetto_vm_timeline(
        &self,
        range: Option<TimeRange>,
    ) -> Result<PerfettoTimeline> {
        self.request(GetPerfettoVmTimeline, time_range(Params::new(), range))
            .await
    }

    /// Which timeline streams exist / are recorded (`getVMTimelineFlags`).
    pub async fn get_vm_timeline_flags(&self) -> Result<TimelineFlags> {
        self.request(GetVmTimelineFlags, Params::new()).await
    }

    /// Selects the timeline streams to record; all others are disabled
    /// (`setVMTimelineFlags`).
    ///
    /// ```no_run
    /// # async fn f(vm: dart_vm_service::VmService) -> dart_vm_service::Result<()> {
    /// use dart_vm_service::TimelineStream;
    /// vm.set_vm_timeline_flags(&[TimelineStream::Dart, TimelineStream::Gc]).await?;
    /// # Ok(()) }
    /// ```
    pub async fn set_vm_timeline_flags(&self, recorded_streams: &[TimelineStream]) -> Result<()> {
        let names: Vec<&str> = recorded_streams
            .iter()
            .map(TimelineStream::as_str)
            .collect();
        self.request_unit(
            SetVmTimelineFlags,
            Params::new().with("recordedStreams", names),
        )
        .await
    }

    /// Drops all recorded timeline events (`clearVMTimeline`).
    pub async fn clear_vm_timeline(&self) -> Result<()> {
        self.request_unit(ClearVmTimeline, Params::new()).await
    }

    // ---- convenience -----------------------------------------------------

    /// The isolate you most likely want: the non-system isolate named `main`,
    /// else the first non-system isolate.
    pub async fn main_isolate(&self) -> Result<IsolateRef> {
        let vm = self.get_vm().await?;
        vm.isolates
            .iter()
            .find(|i| !i.is_system_isolate && i.name == "main")
            .or_else(|| vm.isolates.iter().find(|i| !i.is_system_isolate))
            .or_else(|| vm.isolates.first())
            .cloned()
            .ok_or_else(|| Error::NotFound("the VM has no isolates".into()))
    }
}

pub(crate) fn time_range(p: Params, range: Option<TimeRange>) -> Params {
    match range {
        Some(r) => p
            .with("timeOriginMicros", r.origin_micros)
            .with("timeExtentMicros", r.extent_micros),
        None => p,
    }
}
