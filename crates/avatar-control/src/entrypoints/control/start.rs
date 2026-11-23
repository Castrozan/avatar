use std::io;
use std::net::{Ipv4Addr, SocketAddr};

use super::environment::ControlEnvironment;
use super::http::AudioFileServer;
use super::meter::{MeterSettings, MicrophoneLevelMeter};
use crate::server::{RunningServer, ServerProbe, ShutdownHandle, SpeechPort, serve_probed};
use crate::session::{control_origins, page_origins};

#[derive(Debug, Clone)]
pub struct ControlServerSettings {
    pub environment: ControlEnvironment,
    pub websocket_address: SocketAddr,
    pub meter: Option<MeterSettings>,
}

impl ControlServerSettings {
    #[must_use]
    pub fn from_environment(environment: ControlEnvironment) -> Self {
        let websocket_address = SocketAddr::from((Ipv4Addr::LOCALHOST, environment.websocket_port));
        let meter = environment
            .audio_monitor
            .as_deref()
            .map(MeterSettings::for_source);
        Self {
            environment,
            websocket_address,
            meter,
        }
    }
}

pub struct ControlServer {
    websocket_address: SocketAddr,
    http: AudioFileServer,
    shutdown: ShutdownHandle,
    meter: Option<MicrophoneLevelMeter>,
}

impl ControlServer {
    pub async fn start<Port: SpeechPort>(
        settings: ControlServerSettings,
        speech_port: Port,
    ) -> io::Result<Self> {
        let origins = control_origins(&settings.environment.renderer_port);
        let running = serve_probed(settings.websocket_address, speech_port, origins).await?;
        let http = start_http(&settings.environment, running.probe.clone()).await?;
        let meter = start_meter(settings.meter, &running.probe);
        Ok(Self::assemble(running, http, meter))
    }

    fn assemble(
        running: RunningServer,
        http: AudioFileServer,
        meter: Option<MicrophoneLevelMeter>,
    ) -> Self {
        Self {
            websocket_address: running.address,
            http,
            shutdown: running.shutdown,
            meter,
        }
    }

    #[must_use]
    pub fn websocket_address(&self) -> SocketAddr {
        self.websocket_address
    }

    #[must_use]
    pub fn http_address(&self) -> SocketAddr {
        self.http.local_address()
    }

    pub async fn shutdown(self) {
        stop_meter(self.meter).await;
        self.shutdown.shutdown();
        self.http.close();
    }
}

async fn stop_meter(meter: Option<MicrophoneLevelMeter>) {
    if let Some(meter) = meter {
        meter.stop().await;
    }
}

fn start_meter(
    settings: Option<MeterSettings>,
    probe: &ServerProbe,
) -> Option<MicrophoneLevelMeter> {
    settings.map(|meter| MicrophoneLevelMeter::start(meter, probe.clone()))
}

async fn start_http(
    environment: &ControlEnvironment,
    probe: ServerProbe,
) -> io::Result<AudioFileServer> {
    let origins = page_origins(&environment.renderer_port).to_vec();
    AudioFileServer::start(
        environment.http_port,
        environment.audio_directory.clone(),
        origins,
        probe,
    )
    .await
}
