//! Event stream subscription RPCs. See also [`VmService::subscribe`].

use crate::client::VmService;
use crate::error::Result;
use crate::method::RpcMethod::*;
use crate::params::Params;
use crate::types::StreamId;

impl VmService {
    /// Starts delivery of a stream's events to this connection (`streamListen`).
    /// Fails with code 103 if already subscribed; prefer
    /// [`subscribe`](Self::subscribe) which tolerates that.
    pub async fn stream_listen(&self, stream: StreamId) -> Result<()> {
        let is_service = stream == StreamId::Service;
        self.request_unit(StreamListen, Params::new().with("streamId", stream))
            .await?;
        if is_service {
            self.set_service_stream_listening();
        }
        Ok(())
    }

    /// Stops delivery of a stream (`streamCancel`).
    pub async fn stream_cancel(&self, stream: StreamId) -> Result<()> {
        self.request_unit(StreamCancel, Params::new().with("streamId", stream))
            .await
    }
}
