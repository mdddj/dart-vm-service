//! Program and heap inspection RPCs.

use crate::client::VmService;
use crate::error::Result;
use crate::method::RpcMethod::*;
use crate::options::{GetObjectOptions, InstancesOptions, SourceReportOptions};
use crate::params::Params;
use crate::types::*;

fn isolate(id: &str) -> Params {
    Params::new().with("isolateId", id)
}

impl VmService {
    /// Fetches any object by id (`getObject`).
    pub async fn get_object(
        &self,
        isolate_id: &str,
        object_id: &str,
        options: GetObjectOptions,
    ) -> Result<Obj> {
        self.request(
            GetObject,
            options.apply(isolate(isolate_id).with("objectId", object_id)),
        )
        .await
    }

    /// Library by id, a typed shortcut over [`get_object`](Self::get_object).
    pub async fn get_library(&self, isolate_id: &str, library_id: &str) -> Result<Library> {
        match self
            .get_object(isolate_id, library_id, GetObjectOptions::default())
            .await?
        {
            Obj::Library(l) => Ok(l),
            other => Err(unexpected("Library", &other)),
        }
    }

    /// Class by id.
    pub async fn get_class(&self, isolate_id: &str, class_id: &str) -> Result<Class> {
        match self
            .get_object(isolate_id, class_id, GetObjectOptions::default())
            .await?
        {
            Obj::Class(c) => Ok(c),
            other => Err(unexpected("Class", &other)),
        }
    }

    /// Script (including its source) by id.
    pub async fn get_script(&self, isolate_id: &str, script_id: &str) -> Result<Script> {
        match self
            .get_object(isolate_id, script_id, GetObjectOptions::default())
            .await?
        {
            Obj::Script(s) => Ok(s),
            other => Err(unexpected("Script", &other)),
        }
    }

    /// Instance by id.
    pub async fn get_instance(
        &self,
        isolate_id: &str,
        instance_id: &str,
        options: GetObjectOptions,
    ) -> Result<Instance> {
        match self.get_object(isolate_id, instance_id, options).await? {
            Obj::Instance(i) => Ok(i),
            other => Err(unexpected("Instance", &other)),
        }
    }

    /// All classes of the isolate (`getClassList`).
    pub async fn get_class_list(&self, isolate_id: &str) -> Result<ClassList> {
        self.request(GetClassList, isolate(isolate_id)).await
    }

    /// All scripts of the isolate (`getScripts`).
    pub async fn get_scripts(&self, isolate_id: &str) -> Result<ScriptList> {
        self.request(GetScripts, isolate(isolate_id)).await
    }

    /// Live instances of a class (`getInstances`).
    pub async fn get_instances(
        &self,
        isolate_id: &str,
        class_id: &str,
        limit: i64,
        options: InstancesOptions,
    ) -> Result<InstanceSet> {
        self.request(
            GetInstances,
            options.apply(
                isolate(isolate_id)
                    .with("objectId", class_id)
                    .with("limit", limit),
            ),
        )
        .await
    }

    /// Live instances of a class packed into one Dart `List` (`getInstancesAsList`).
    pub async fn get_instances_as_list(
        &self,
        isolate_id: &str,
        class_id: &str,
        options: InstancesOptions,
    ) -> Result<InstanceRef> {
        self.request(
            GetInstancesAsList,
            options.apply(isolate(isolate_id).with("objectId", class_id)),
        )
        .await
    }

    /// Who points at the object (`getInboundReferences`).
    pub async fn get_inbound_references(
        &self,
        isolate_id: &str,
        target_id: &str,
        limit: i64,
        id_zone_id: Option<&str>,
    ) -> Result<InboundReferences> {
        self.request(
            GetInboundReferences,
            isolate(isolate_id)
                .with("targetId", target_id)
                .with("limit", limit)
                .opt("idZoneId", id_zone_id),
        )
        .await
    }

    /// Why the object is alive (`getRetainingPath`).
    pub async fn get_retaining_path(
        &self,
        isolate_id: &str,
        target_id: &str,
        limit: i64,
        id_zone_id: Option<&str>,
    ) -> Result<RetainingPath> {
        self.request(
            GetRetainingPath,
            isolate(isolate_id)
                .with("targetId", target_id)
                .with("limit", limit)
                .opt("idZoneId", id_zone_id),
        )
        .await
    }

    /// Coverage / possible-breakpoint report (`getSourceReport`).
    pub async fn get_source_report(
        &self,
        isolate_id: &str,
        reports: &[SourceReportKind],
        options: SourceReportOptions,
    ) -> Result<SourceReport> {
        self.request(
            GetSourceReport,
            options.apply(isolate(isolate_id).with("reports", reports)),
        )
        .await
    }

    /// Maps `package:` URIs (or file URIs) to resolved ones
    /// (`lookupResolvedPackageUris`); `local` asks for local file paths.
    pub async fn lookup_resolved_package_uris(
        &self,
        isolate_id: &str,
        uris: &[&str],
        local: Option<bool>,
    ) -> Result<UriList> {
        self.request(
            LookupResolvedPackageUris,
            isolate(isolate_id).with("uris", uris).opt("local", local),
        )
        .await
    }

    /// The inverse mapping, resolved → `package:` (`lookupPackageUris`).
    pub async fn lookup_package_uris(&self, isolate_id: &str, uris: &[&str]) -> Result<UriList> {
        self.request(LookupPackageUris, isolate(isolate_id).with("uris", uris))
            .await
    }
}

fn unexpected(wanted: &str, got: &Obj) -> crate::Error {
    crate::Error::Protocol(format!("expected a {wanted} object, got {got:?}"))
}
