use pythnet_sdk::wire::v1::{AccumulatorUpdateData, Proof};

use super::{hermes::PriceUpdate, EncodingType};

/// Parse [`AccumulatorUpdateData`] from price update.
pub fn parse_accumulator_update_datas(
    update: &PriceUpdate,
) -> crate::Result<Vec<AccumulatorUpdateData>> {
    let update = &update.binary;
    let datas = match update.encoding {
        EncodingType::Base64 => {
            use base64::{engine::general_purpose::STANDARD, Engine};

            update
                .data
                .iter()
                .map(|data| {
                    STANDARD
                        .decode(data)
                        .map_err(crate::Error::from)
                        .and_then(|data| parse_accumulator_update_data(&data))
                })
                .collect::<crate::Result<Vec<_>>>()?
        }
        EncodingType::Hex => {
            unimplemented!()
        }
    };
    Ok(datas)
}

#[inline]
fn parse_accumulator_update_data(data: &[u8]) -> crate::Result<AccumulatorUpdateData> {
    AccumulatorUpdateData::try_from_slice(data).map_err(crate::Error::unknown)
}

/// Get guardian set index from [`Proof`].
pub fn get_guardian_set_index(proof: &Proof) -> crate::Result<i32> {
    let vaa = get_vaa_buffer(proof);
    if vaa.len() < 5 {
        return Err(crate::Error::unknown("invalid vaa"));
    }
    let index: &[u8; 4] = (&vaa[1..5]).try_into().map_err(crate::Error::unknown)?;
    Ok(i32::from_be_bytes(*index))
}

/// Get vaa buffer.
pub fn get_vaa_buffer(proof: &Proof) -> &[u8] {
    match proof {
        Proof::WormholeMerkle { vaa, .. } => vaa.as_ref(),
    }
}
