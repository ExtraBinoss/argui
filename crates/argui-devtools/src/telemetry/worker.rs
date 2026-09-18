use super::{DeviceTelemetryProvider, TelemetrySnapshot, process::ProcessSampler};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use web_time::Instant;

#[cfg_attr(not(feature = "all-smi"), derive(Default))]
pub(super) struct Worker {
    provider: Option<Box<dyn DeviceTelemetryProvider>>,
    channels: Option<(SyncSender<bool>, Receiver<TelemetrySnapshot>)>,
    pending: bool,
    failure: Option<String>,
}

#[cfg(feature = "all-smi")]
impl Default for Worker {
    fn default() -> Self {
        Self {
            provider: Some(Box::new(super::AllSmi::default())),
            channels: None,
            pending: false,
            failure: None,
        }
    }
}

impl Worker {
    pub fn new(provider: Box<dyn DeviceTelemetryProvider>) -> Self {
        Self {
            provider: Some(provider),
            ..Self::default()
        }
    }

    pub fn source(&self) -> Option<String> {
        self.provider
            .as_ref()
            .map(|provider| provider.name().to_owned())
    }

    pub fn request(&mut self, devices: bool) -> bool {
        if self.pending || self.failure.is_some() {
            return false;
        }
        if self.channels.is_none() {
            let (send, requests) = mpsc::sync_channel(1);
            let (results, receive) = mpsc::sync_channel(1);
            let mut provider = self.provider.take();
            let spawned = std::thread::Builder::new()
                .name("argui-telemetry".into())
                .stack_size(512 * 1024)
                .spawn(move || {
                    let mut process = ProcessSampler::default();
                    while let Ok(devices) = requests.recv() {
                        let start = Instant::now();
                        let mut snapshot = TelemetrySnapshot::default();
                        match process.sample() {
                            Ok(process) => snapshot.process = Some(process),
                            Err(error) => snapshot.errors.push(error),
                        }
                        if devices && let Some(provider) = &mut provider {
                            match provider.sample() {
                                Ok(devices) => snapshot.devices = devices,
                                Err(error) => snapshot
                                    .errors
                                    .push(format!("{}: {error}", provider.name())),
                            }
                        }
                        snapshot.collection_time = start.elapsed();
                        if results.send(snapshot).is_err() {
                            break;
                        }
                    }
                });
            if let Err(error) = spawned {
                self.failure = Some(format!("Could not start telemetry: {error}"));
                return false;
            }
            self.channels = Some((send, receive));
        }
        let Some((sender, _)) = &self.channels else {
            return false;
        };
        if sender.try_send(devices).is_err() {
            self.failure = Some("Telemetry worker stopped".into());
            return false;
        }
        self.pending = true;
        true
    }

    pub fn poll(&mut self) -> Option<TelemetrySnapshot> {
        if let Some(error) = self.failure.take() {
            self.pending = true;
            return Some(TelemetrySnapshot {
                errors: vec![error],
                ..Default::default()
            });
        }
        let (_, receiver) = self.channels.as_ref()?;
        match receiver.try_recv() {
            Ok(snapshot) => {
                self.pending = false;
                Some(snapshot)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) if self.pending => {
                self.channels = None;
                // A provider panic ends this worker. Preserve its error rather
                // than retrying it continuously from animation frames.
                self.pending = true;
                Some(TelemetrySnapshot {
                    errors: vec!["Telemetry worker stopped".into()],
                    ..Default::default()
                })
            }
            Err(TryRecvError::Disconnected) => None,
        }
    }
}
