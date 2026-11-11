//! CPU profiler and allocation RPCs.

use crate::api::vm::time_range;
use crate::client::VmService;
use crate::error::Result;
use crate::method::RpcMethod::*;
use crate::options::AllocationProfileOptions;
use crate::params::Params;
use crate::types::*;

fn isolate(id: &str) -> Params {
    Params::new().with("isolateId", id)
}

impl VmService {
    /// CPU samples inside a time window (`getCpuSamples`). Errors if the profiler is
    /// disabled (see [`VmFlag::Profiler`]).
    pub async fn get_cpu_samples(&self, isolate_id: &str, range: TimeRange) -> Result<CpuSamples> {
        self.request(GetCpuSamples, time_range(isolate(isolate_id), Some(range)))
            .await
    }

    /// CPU samples in Perfetto format (`getPerfettoCpuSamples`).
    pub async fn get_perfetto_cpu_samples(
        &self,
        isolate_id: &str,
        range: Option<TimeRange>,
    ) -> Result<PerfettoCpuSamples> {
        self.request(
            GetPerfettoCpuSamples,
            time_range(isolate(isolate_id), range),
        )
        .await
    }

    /// Discards all collected samples (`clearCpuSamples`).
    pub async fn clear_cpu_samples(&self, isolate_id: &str) -> Result<()> {
        self.request_unit(ClearCpuSamples, isolate(isolate_id))
            .await
    }

    /// Per-class allocation statistics (`getAllocationProfile`).
    pub async fn get_allocation_profile(
        &self,
        isolate_id: &str,
        options: AllocationProfileOptions,
    ) -> Result<AllocationProfile> {
        self.request(GetAllocationProfile, options.apply(isolate(isolate_id)))
            .await
    }

    /// Allocation stack traces for classes enabled with
    /// [`set_trace_class_allocation`](Self::set_trace_class_allocation)
    /// (`getAllocationTraces`).
    pub async fn get_allocation_traces(
        &self,
        isolate_id: &str,
        range: Option<TimeRange>,
        class_id: Option<&str>,
    ) -> Result<CpuSamples> {
        self.request(
            GetAllocationTraces,
            time_range(isolate(isolate_id), range).opt("classId", class_id),
        )
        .await
    }

    /// Starts/stops recording allocation traces for a class (`setTraceClassAllocation`).
    pub async fn set_trace_class_allocation(
        &self,
        isolate_id: &str,
        class_id: &str,
        enable: bool,
    ) -> Result<()> {
        self.request_unit(
            SetTraceClassAllocation,
            isolate(isolate_id)
                .with("classId", class_id)
                .with("enable", enable),
        )
        .await
    }
}
