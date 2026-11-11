//! Debugger RPCs: breakpoints, execution control, stack, evaluation.

use crate::client::VmService;
use crate::error::Result;
use crate::method::RpcMethod::*;
use crate::options::{EvalOptions, InvokeOptions};
use crate::params::Params;
use crate::types::*;

fn isolate(id: &str) -> Params {
    Params::new().with("isolateId", id)
}

impl VmService {
    // ---- breakpoints -----------------------------------------------------

    /// Breakpoint on `line` (1-based) of a script identified by id (`addBreakpoint`).
    pub async fn add_breakpoint(
        &self,
        isolate_id: &str,
        script_id: &str,
        line: i64,
        column: Option<i64>,
    ) -> Result<Breakpoint> {
        self.request(
            AddBreakpoint,
            isolate(isolate_id)
                .with("scriptId", script_id)
                .with("line", line)
                .opt("column", column),
        )
        .await
    }

    /// Like [`add_breakpoint`](Self::add_breakpoint) but by script URI; works before the
    /// script is loaded (`addBreakpointWithScriptUri`).
    pub async fn add_breakpoint_with_script_uri(
        &self,
        isolate_id: &str,
        script_uri: &str,
        line: i64,
        column: Option<i64>,
    ) -> Result<Breakpoint> {
        self.request(
            AddBreakpointWithScriptUri,
            isolate(isolate_id)
                .with("scriptUri", script_uri)
                .with("line", line)
                .opt("column", column),
        )
        .await
    }

    /// Breakpoint at the entry of a function (`addBreakpointAtEntry`).
    pub async fn add_breakpoint_at_entry(
        &self,
        isolate_id: &str,
        function_id: &str,
    ) -> Result<Breakpoint> {
        self.request(
            AddBreakpointAtEntry,
            isolate(isolate_id).with("functionId", function_id),
        )
        .await
    }

    pub async fn remove_breakpoint(&self, isolate_id: &str, breakpoint_id: &str) -> Result<()> {
        self.request_unit(
            RemoveBreakpoint,
            isolate(isolate_id).with("breakpointId", breakpoint_id),
        )
        .await
    }

    /// Enables or disables a breakpoint (`setBreakpointState`).
    pub async fn set_breakpoint_state(
        &self,
        isolate_id: &str,
        breakpoint_id: &str,
        enable: bool,
    ) -> Result<Breakpoint> {
        self.request(
            SetBreakpointState,
            isolate(isolate_id)
                .with("breakpointId", breakpoint_id)
                .with("enable", enable),
        )
        .await
    }

    /// Marks a library (non-)debuggable (`setLibraryDebuggable`).
    pub async fn set_library_debuggable(
        &self,
        isolate_id: &str,
        library_id: &str,
        is_debuggable: bool,
    ) -> Result<()> {
        self.request_unit(
            SetLibraryDebuggable,
            isolate(isolate_id)
                .with("libraryId", library_id)
                .with("isDebuggable", is_debuggable),
        )
        .await
    }

    // ---- execution control ----------------------------------------------

    /// Interrupts the isolate (`pause`).
    pub async fn pause(&self, isolate_id: &str) -> Result<()> {
        self.request_unit(Pause, isolate(isolate_id)).await
    }

    /// Resumes a paused isolate (`resume`).
    pub async fn resume(&self, isolate_id: &str) -> Result<()> {
        self.request_unit(Resume, isolate(isolate_id)).await
    }

    /// Resumes with a single step (`resume` with a [`StepOption`]).
    pub async fn step(&self, isolate_id: &str, step: StepOption) -> Result<()> {
        self.request_unit(Resume, isolate(isolate_id).with("step", step))
            .await
    }

    /// Rewinds to a frame, `frame_index` 1 is the caller of the top frame
    /// (`resume` with [`StepOption::Rewind`]).
    pub async fn rewind(&self, isolate_id: &str, frame_index: i64) -> Result<()> {
        self.request_unit(
            Resume,
            isolate(isolate_id)
                .with("step", StepOption::Rewind)
                .with("frameIndex", frame_index),
        )
        .await
    }

    /// Current stack, optionally truncated to `limit` frames (`getStack`).
    pub async fn get_stack(
        &self,
        isolate_id: &str,
        limit: Option<i64>,
        id_zone_id: Option<&str>,
    ) -> Result<Stack> {
        self.request(
            GetStack,
            isolate(isolate_id)
                .opt("limit", limit)
                .opt("idZoneId", id_zone_id),
        )
        .await
    }

    // ---- evaluation ------------------------------------------------------

    /// Evaluates `expression` in the context of a library, class or instance
    /// (`evaluate`).
    ///
    /// ```no_run
    /// # async fn f(vm: dart_vm_service::VmService, iso: &str, lib: &str) -> dart_vm_service::Result<()> {
    /// use dart_vm_service::{EvalOptions, EvalResult};
    /// match vm.evaluate(iso, lib, "1 + 2", EvalOptions::default()).await? {
    ///     EvalResult::Instance(i) => println!("{:?}", i.value_as_string),
    ///     EvalResult::Error(e) => eprintln!("runtime error: {}", e.message),
    /// }
    /// # Ok(()) }
    /// ```
    pub async fn evaluate(
        &self,
        isolate_id: &str,
        target_id: &str,
        expression: &str,
        options: EvalOptions,
    ) -> Result<EvalResult> {
        self.request(
            Evaluate,
            options.apply(
                isolate(isolate_id)
                    .with("targetId", target_id)
                    .with("expression", expression),
            ),
        )
        .await
    }

    /// Evaluates in a stack frame of a paused isolate; frame 0 is the top
    /// (`evaluateInFrame`).
    pub async fn evaluate_in_frame(
        &self,
        isolate_id: &str,
        frame_index: i64,
        expression: &str,
        options: EvalOptions,
    ) -> Result<EvalResult> {
        self.request(
            EvaluateInFrame,
            options.apply(
                isolate(isolate_id)
                    .with("frameIndex", frame_index)
                    .with("expression", expression),
            ),
        )
        .await
    }

    /// Calls a method on a library, class or instance (`invoke`). Getters, setters
    /// and constructors are not supported by the protocol.
    pub async fn invoke(
        &self,
        isolate_id: &str,
        target_id: &str,
        selector: &str,
        argument_ids: &[&str],
        options: InvokeOptions,
    ) -> Result<EvalResult> {
        self.request(
            Invoke,
            options.apply(
                isolate(isolate_id)
                    .with("targetId", target_id)
                    .with("selector", selector)
                    .with("argumentIds", argument_ids),
            ),
        )
        .await
    }
}
