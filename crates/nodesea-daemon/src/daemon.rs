//! Daemon composition and lifecycle management.

use nodesea_bt::{Engine, SettingsPack};
use nodesea_proto::v1::engine_status_service_server::EngineStatusServiceServer;
use tokio::task::JoinHandle;

use crate::{
    DaemonError, Endpoint,
    engine::EngineStatusServiceImpl,
    transport::{Listener, PlatformTransport, Transport},
};

/// Runs the NodeSea daemon and owns the engine and transport lifecycles.
pub async fn run() -> Result<(), DaemonError> {
    // The native libtorrent session is not `Send`, so keep its runner on a
    // current-thread local task set owned by this composition boundary.
    tokio::task::LocalSet::new().run_until(run_local()).await
}

async fn run_local() -> Result<(), DaemonError> {
    // Build the transport before starting the engine so startup failures do
    // not leave a background engine task running without a listener.
    let endpoint = Endpoint::default_endpoint()?;
    let endpoint_path = endpoint.path().to_owned();
    let daemon = Daemon::bind(endpoint).await?;

    tracing::info!(
        endpoint = %endpoint_path.display(),
        "NodeSea daemon started"
    );

    let engine = build_engine();
    // The handle is cloneable and does not own the native session. The engine
    // itself is moved into the local task and lives until `run` completes.
    let engine_handle = engine.handle();

    let service = EngineStatusServiceImpl::new(engine_handle.clone());

    let mut engine_task: JoinHandle<Result<(), nodesea_bt::EngineError>> =
        tokio::task::spawn_local(engine.run());

    tokio::select! {
        daemon_result = daemon.run(service) => {
            // If the daemon loop exits first, stop the engine explicitly and
            // wait for its task before returning from the composition layer.
            let shutdown_result = engine_handle.shutdown().await;
            let engine_result = engine_task.await?;

            daemon_result?;
            shutdown_result?;
            engine_result?;
            Ok(())
        }
        engine_result = &mut engine_task => {
            // An engine failure or orderly stop ends the daemon as well; the
            // `Daemon` drop implementation then cleans up the socket.
            engine_result??;
            Ok(())
        }
    }
}

fn build_engine() -> Engine {
    let settings = SettingsPack::new();

    Engine::builder().set_settings_pack(settings).build()
}

struct Daemon {
    listener: Option<<PlatformTransport as Transport>::Listener>,
}

impl Daemon {
    async fn bind(endpoint: Endpoint) -> Result<Self, DaemonError> {
        let listener = PlatformTransport::bind(&endpoint).await?;
        Ok(Self {
            listener: Some(listener),
        })
    }

    async fn run(mut self, service: EngineStatusServiceImpl) -> Result<(), DaemonError> {
        let listener = self.listener.take().expect("daemon listener must exist");

        let incoming = listener.into_incoming();

        tonic::transport::Server::builder()
            .add_service(EngineStatusServiceServer::new(service))
            .serve_with_incoming(incoming)
            .await
            .map_err(DaemonError::Grpc)
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        if let Some(listener) = self.listener.as_ref()
            && let Err(error) = listener.cleanup()
        {
            tracing::error!(%error, "Failed to clean up the daemon transport");
        }
    }
}
