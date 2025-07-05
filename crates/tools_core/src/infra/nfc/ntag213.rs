use log::warn;
use nfc1::{target_info::TargetInfo, Device, Modulation};

use super::NfcError;

const NTAG213_MODULATION: Modulation = Modulation {
    modulation_type: nfc1::ModulationType::Iso14443a,
    baud_rate: nfc1::BaudRate::Baud106,
};
const UID_LEN: usize = 7;
const RX_LEN: usize = 256;
const WRITE_PAGE_ADDR_MAX: usize = 0x2C;
const WRITE_PAGE_ADDR_MIN: usize = 0x02;
const READ_PAGE_ADDR_MAX: usize = 0x2C;
const READ_PAGE_ADDR_MIN: usize = 0x00;

const CFG0_PAGE_ADDR: usize = 0x29;

mod cmd_code {
    pub const GET_VERSION: u8 = 0x60;
    pub const READ: u8 = 0x30;
    pub const WRITE: u8 = 0xA2;
}

/// Scan for an NTAG213 card and return its UID.
///
/// # Arguments
/// * `reader` - The NFC reader device to scan with.
///
/// # Returns
/// * `Ok(String)` containing the UID of the scanned NTAG213 card.
///
/// # Errors
/// * `NfcError::NfcError` if there is an error during the NFC communication.
/// * `NfcError::InvalidTarget` if the target is not a valid NTAG213 card.
/// * `NfcError::InvalidArgument` if the UID length is not as expected.
pub fn scan(reader: &mut Device) -> Result<String, NfcError> {
    reader.set_property_bool(nfc1::Property::EasyFraming, false)?;
    let uid = match reader.initiator_select_passive_target(&NTAG213_MODULATION)?.target_info {
        TargetInfo::Iso14443a(x) => {
            let mut uid = [0u8; UID_LEN];
            uid.copy_from_slice(&x.uid[..UID_LEN]);
            hex::encode(uid)
        }
        _ => unreachable!(),
    };
    validate(reader)?;
    Ok(uid)
}

fn transceive(tx: &[u8],  reader: &mut Device) -> Result<Vec<u8>, NfcError> {
    let recv = reader.initiator_transceive_bytes(tx, RX_LEN, nfc1::Timeout::Default)?;
    Ok(recv)
}

fn transceive_write(
    tx: &[u8],
    addr: usize,
    reader: &mut Device,
) -> Result<(), NfcError> {
    match transceive(&tx, reader) {
        Ok(_)=> Ok(()),
        Err(NfcError::NfcError(ref e)) if (*e == nfc1::Error::RfTransmissionError || *e == nfc1::Error::Timeout) => Ok(()),
        Err(e) => {
            warn!("Failed to write data to page {}: {}", addr, e);
            return Err(e);
        }
    }
}

fn get_version(reader: &mut Device) -> Result<Vec<u8>, NfcError> {
    let tx = [cmd_code::GET_VERSION];
    transceive(&tx, reader)
}

fn validate(reader: &mut Device) -> Result<(), NfcError> {
    const GET_VERSION_RESPONSE: [u8; 8] = [0x00, 0x04, 0x04, 0x02, 0x01, 0x00, 0x0F, 0x03];
    let version = get_version(reader)?;
    if &version == &GET_VERSION_RESPONSE {
        Ok(())
    } else {
        warn!("GET_VERSION response mismatch: expected {:?}, got {:?}", GET_VERSION_RESPONSE, version);
        Err(NfcError::InvalidTarget)
    }
}

fn write(data: &[u8], page_addr: usize, reader: &mut Device) -> Result<(), NfcError> {
    if data.is_empty() {
        return Ok(());
    }

    let page_num = data.len().div_ceil(4);
    let page_end = page_addr + page_num - 1;
    if page_addr < WRITE_PAGE_ADDR_MIN || page_end > WRITE_PAGE_ADDR_MAX {
        warn!("Write address out of bounds: {} to {}", page_addr, page_end);
        return Err(NfcError::InvalidArgument(format!(
            "Write address out of bounds: {} to {}",
            page_addr, page_end
        )));
    }
    let mut addr = page_addr;
    for chunk in data.chunks_exact(4) {
        let mut tx = vec![cmd_code::WRITE, addr as u8];
        tx.extend_from_slice(chunk);
        transceive_write(&tx, addr, reader)?;
        addr += 1;
    }
    let r = data.chunks_exact(4).remainder();
    if !r.is_empty() {
        let mut tx = vec![cmd_code::WRITE, addr as u8];
        let mut last_page_data = [0u8; 4];
        last_page_data[..r.len()].copy_from_slice(r);
        tx.extend_from_slice(&last_page_data);
        transceive_write(&tx, addr, reader)?;
    }
    Ok(())
}

/// Read data from the NTAG213 card.
///
/// # Arguments
/// * `page_addr` - The starting page address to read from.
/// * `byte_len` - The number of bytes to read.
/// * `reader` - The NFC reader device to read from.
///
/// # Returns
/// * `Ok(Vec<u8>)` containing the read data if successful.
///
/// # Errors
/// * `NfcError::InvalidArgument` if the page address or byte length is invalid.
/// * `NfcError::UnexpectedResponse` if the response length is not as expected.
/// * `NfcError::NfcError` if there is an error during the NFC communication.
pub fn read(page_addr: usize, byte_len: usize, reader: &mut Device) -> Result<Vec<u8>, NfcError> {
    if byte_len == 0 {
        return Ok(Vec::new());
    }
    let page_num = byte_len.div_ceil(4);
    let end_page = page_addr + page_num - 1;
    if page_addr < READ_PAGE_ADDR_MIN || end_page > READ_PAGE_ADDR_MAX {
        warn!("Read address out of bounds: {} to {}", page_addr, end_page);
        return Err(NfcError::InvalidArgument(format!(
            "Read address out of bounds: {} to {}",
            page_addr, end_page
        )));
    }

    let mut rs = Vec::with_capacity(byte_len);

    let mut cur_page = page_addr;
    while cur_page <= end_page {
        let tx = [cmd_code::READ, cur_page as u8];
        let recv = transceive(&tx, reader)?;
        if recv.len() != 16 {
            warn!("Unexpected response length: expected 16, got {}", recv.len());
            return Err(NfcError::UnexpectedResponse(format!(
                "Unexpected response length: expected 16, got {}",
                recv.len()
            )));
        }
        let byte_needed = (end_page - cur_page + 1).min(4) * 4;
        rs.extend_from_slice(&recv[..byte_needed]);
        cur_page += 4;
    }
    rs.truncate(byte_len);
    Ok(rs)
}

fn strip_uri_prefix(url: &str) -> (u8, &str) {
    if url.starts_with("https://www.") {
        (0x02, &url[12..])
    } else if url.starts_with("http://www.") {
        (0x01, &url[11..])
    } else if url.starts_with("https://") {
        (0x04, &url[8..])
    } else if url.starts_with("http://") {
        (0x03, &url[7..])
    } else if url.starts_with("tel:") {
        (0x05, &url[4..])
    } else if url.starts_with("mailto:") {
        (0x06, &url[7..])
    } else {
        (0x00, url)
    }
}

fn url_to_msgtlv_payload(url: &str) -> Vec<u8> {
    let (prefix, stripped_url) = strip_uri_prefix(url);
    let mut rs = vec![prefix];
    rs.extend_from_slice(stripped_url.as_bytes());
    rs
}

fn url_to_write_bytes(url: &str) -> Vec<u8> {
    use ndef_const::*;

    let payload = url_to_msgtlv_payload(url);
    let mut rs = Vec::with_capacity(TLV_HEADER_LEN + 2 + payload.len() + 1);

    rs.extend_from_slice(&TLV_HEADER);

    rs.extend_from_slice(&[
        TLV_NDEF_MESSAGE,
        payload.len() as u8 +4,
        TLV_NDEF_RECORD,
        TLV_NDEF_MESSAGE_TYPE_LEN, payload.len() as u8,
        TLV_NDEF_RECORD_TYPE,
    ]);
    rs.extend_from_slice(&payload);

    rs.push(TLV_TERMINATOR);
    rs
}

mod ndef_const {
    pub const TLV_HEADER: [u8; 5] = [0x01, 0x03, 0xA0, 0x0C, 0x34];
    pub const TLV_HEADER_LEN: usize = 5;

    pub const TLV_NDEF_MESSAGE: u8 = 0x03;
    pub const TLV_NDEF_MESSAGE_TYPE_LEN: u8 = 1;
    pub const TLV_NDEF_RECORD: u8 = 0xD1;
    pub const TLV_NDEF_RECORD_TYPE: u8 = b'U';

    pub const TLV_TERMINATOR: u8 = 0xFE;
}

/// Write a URL to the NTAG213 card.
///
/// # Arguments
/// * `url` - The URL to write, which can be prefixed with `https://`, `http://`, `tel:`, or `mailto:`.
/// * `reader` - The NFC reader device to write to.
///
/// # Returns
/// * `Ok(())` if the write operation was successful.
///
/// # Errors
/// * `NfcError::InvalidArgument` if the URL is invalid or the write address is out of bounds.
/// * `NfcError::NfcError` if there is an error during the NFC communication.
/// * `NfcError::UnexpectedResponse` if the response length is not as expected.
pub fn write_url(url: &str, reader: &mut Device) -> Result<(), NfcError> {
    const PAGE_ADDR: usize = 4;

    let data = url_to_write_bytes(url);
    write(&data, PAGE_ADDR, reader)
}

pub fn set_uid_mirror(page_addr: usize, byte_offset: usize, reader: &mut Device) -> Result<(), NfcError> {
    const MIRROR_PAGE_MIN: usize = 0x04;
    const MIRROR_PAGE_MAX: usize = 0x27 - 3;
    const MIRROR_BYTE_BITS_MAX: usize = 0x01;

    if page_addr < MIRROR_PAGE_MIN || page_addr > MIRROR_PAGE_MAX {
        warn!("Mirror page address out of bounds: {}", page_addr);
        return Err(NfcError::InvalidArgument(format!(
            "Mirror page address out of bounds: {}",
            page_addr
        )));
    }
    if byte_offset > 0b11 || (page_addr == MIRROR_PAGE_MAX && byte_offset > MIRROR_BYTE_BITS_MAX) {
        warn!("Mirror byte out of bounds: {}", byte_offset);
        return Err(NfcError::InvalidArgument(format!(
            "Mirror byte out of bounds: {}",
            byte_offset
        )));
    }

    let mut tx = read(CFG0_PAGE_ADDR, 4, reader)?;
    tx[0] = (tx[0] & 0b1111) | (0b01 << 6) | (byte_offset << 4) as u8;
    tx[2] = page_addr as u8;
    write(&tx, CFG0_PAGE_ADDR, reader)?;
    Ok(())
}

pub fn with_card<F, R>(reader: &mut Device, f: F) -> Result<R, NfcError>
where
    F: FnOnce(&mut Device) -> Result<R, NfcError>,
{
    scan(reader)?;
    let rs = f(reader);
    reader.initiator_deselect_target()?;
    rs
}

#[test]
fn test() {
    let mut ctx = nfc1::Context::new().unwrap();
    let mut reader = super::open_reader(None, &mut ctx).unwrap();

    let uid = scan(&mut reader).unwrap();
    println!("Scanned NTAG213 card with UID: {:?}", uid);

    write_url("https://example.com?uid=11223344556677", &mut reader).unwrap();
    let mirror_page = 0x04_usize + 7;
    let byte_offset = 0_usize;
    set_uid_mirror(mirror_page, byte_offset, &mut reader).unwrap();
    let read_rs = read(4, 64, &mut reader).unwrap();
    println!("Read data: {:?}", read_rs);
    reader.initiator_deselect_target().unwrap();
}
