use log::warn;
use rxing::RXingResult;

#[derive(Debug, thiserror::Error)]
pub enum BarcodeError {
    #[error(transparent)]
    RxingException(#[from] rxing::Exceptions),
    #[error(transparent)]
    TaskJoinError(#[from] tokio::task::JoinError),
    #[error("barcode not found")]
    NotFound,
}

async fn decode_from_luma(luma: Vec<u8>, width: u32, height: u32) -> Result<Vec<RXingResult>, BarcodeError> {
    let rs = tokio::task::spawn_blocking(move || {
        rxing::helpers::detect_multiple_in_luma(luma, width, height)
    }).await
        .inspect_err(|e| warn!("luma decode task join error: {e}"))?;

    if let Err(rxing::Exceptions::NotFoundException(_)) = rs {
        return Ok(vec![]);
    }
    Ok(rs?)
}

/// Decode a barcode from a luma image. Only the first result is returned.
///
/// # Arguments
/// * `luma` - A vector of u8 representing the luma image.
/// * `width` - The width of the image.
/// * `height` - The height of the image.
///
/// # Returns
/// * `Ok(Some(String))` if a barcode is found and decoded successfully.
/// * `Ok(None)` if no barcode is found.
/// * `Err(BarcodeError)` if an error occurs during decoding.
pub async fn decode_text_from_luma(luma: Vec<u8>, width: u32, height: u32) -> Result<Option<String>, BarcodeError> {
    let rs = decode_from_luma(luma, width, height).await?;
    if rs.is_empty() {
        Ok(None)
    } else {
        let text = rs[0].getText().to_string();
        Ok(Some(text))
    }
}
