use nodesea_bt::{EngineHandle, EngineStatus};
use nodesea_proto::v1::{
    GetStatusRequest, GetStatusResponse, engine_status_service_server::EngineStatusService,
};

pub(crate) struct EngineStatusServiceImpl {
    engine: EngineHandle,
}

impl EngineStatusServiceImpl {
    pub(crate) fn new(engine: EngineHandle) -> Self {
        Self { engine }
    }
}

fn to_proto_status(status: EngineStatus) -> nodesea_proto::v1::EngineStatus {
    match status {
        EngineStatus::Idle => nodesea_proto::v1::EngineStatus::Idle,
        EngineStatus::Starting => nodesea_proto::v1::EngineStatus::Starting,
        EngineStatus::Running => nodesea_proto::v1::EngineStatus::Running,
        EngineStatus::Stopping => nodesea_proto::v1::EngineStatus::Stopping,
        EngineStatus::Stopped => nodesea_proto::v1::EngineStatus::Stopped,
        EngineStatus::Failed => nodesea_proto::v1::EngineStatus::Failed,
    }
}

#[tonic::async_trait]
impl EngineStatusService for EngineStatusServiceImpl {
    async fn get_status(
        &self,
        _request: tonic::Request<GetStatusRequest>,
    ) -> Result<tonic::Response<GetStatusResponse>, tonic::Status> {
        let status = to_proto_status(self.engine.status());

        Ok(tonic::Response::new(GetStatusResponse {
            status: status as i32,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_every_engine_status_to_the_proto_status() {
        let statuses = [
            (EngineStatus::Idle, nodesea_proto::v1::EngineStatus::Idle),
            (
                EngineStatus::Starting,
                nodesea_proto::v1::EngineStatus::Starting,
            ),
            (
                EngineStatus::Running,
                nodesea_proto::v1::EngineStatus::Running,
            ),
            (
                EngineStatus::Stopping,
                nodesea_proto::v1::EngineStatus::Stopping,
            ),
            (
                EngineStatus::Stopped,
                nodesea_proto::v1::EngineStatus::Stopped,
            ),
            (
                EngineStatus::Failed,
                nodesea_proto::v1::EngineStatus::Failed,
            ),
        ];

        for (engine_status, proto_status) in statuses {
            assert_eq!(to_proto_status(engine_status), proto_status);
        }
    }
}
