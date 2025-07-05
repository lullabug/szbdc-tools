use log::info;
use nfc1::{Context, Device};

pub mod ntag213;

#[derive(Debug, thiserror::Error)]
pub enum NfcError {
    #[error(transparent)]
    NfcError(#[from] nfc1::Error),
    #[error("invalid target")]
    InvalidTarget,
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
}

pub fn list_reader() -> Result<Vec<String>, NfcError> {
    const MAX_DEVICES: usize = 8;
    info!("finding NFC readers");
    let mut ctx = nfc1::Context::new()?;
    let devices = ctx.list_devices(MAX_DEVICES)?;
    info!("found {} NFC readers", devices.len());
    devices.iter()
        .for_each(|x| info!("NFC reader: {}", x));
    Ok(devices)
}

pub fn open_reader(name: Option<&str>, ctx: &mut Context) -> Result<Device, NfcError> {
    let device = match name {
        Some(name) => {
            info!("opening NFC reader: {}", name);
            ctx.open_with_connstring(name)?
        }
        None => {
            info!("opening first available NFC reader");
            ctx.open()?
        }
    };
    Ok(device)
}