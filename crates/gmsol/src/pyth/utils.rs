use pythnet_sdk::wire::v1::AccumulatorUpdateData;

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
