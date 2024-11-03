use std::{fmt, sync::OnceLock};

use ethabi::{ethereum_types::U256, ParamType, Token};
use ruint::aliases::U192;

static SCHEMA_1: OnceLock<Vec<ParamType>> = OnceLock::<Vec<ParamType>>::new();
static SCHEMA_REPORT_V3: OnceLock<Vec<ParamType>> = OnceLock::<Vec<ParamType>>::new();

/// Report.
pub struct Report {
    /// The stream ID the report has data for.
    pub feed_id: [u8; 32],
    /// Earliest timestamp for which price is applicable.
    pub valid_from_timestamp: u32,
    /// Latest timestamp for which price is applicable.
    pub observations_timestamp: u32,
    native_fee: U192,
    link_fee: U192,
    expires_at: u32,
    // FIXME: the following types should be I192.
    /// DON consensus median price (8 or 18 decimals).
    pub price: U192,
    /// Simulated price impact of a buy order up to the X% depth of liquidity utilisation (8 or 18 decimals).
    pub bid: U192,
    /// Simulated price impact of a sell order up to the X% depth of liquidity utilisation (8 or 18 decimals).
    pub ask: U192,
}

impl fmt::Debug for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Report")
            .field("feed_id", &self.feed_id)
            .field("valid_from_timestamp", &self.valid_from_timestamp)
            .field("observations_timestamp", &self.observations_timestamp)
            .field("native_fee", self.native_fee.as_limbs())
            .field("link_fee", self.link_fee.as_limbs())
            .field("expires_at", &self.expires_at)
            .field("price", self.price.as_limbs())
            .field("bid", self.bid.as_limbs())
            .field("ask", self.ask.as_limbs())
            .finish()
    }
}

/// Decode Report.
pub fn decode(data: &[u8]) -> ethabi::Result<Report> {
    let schema_1 = SCHEMA_1.get_or_init(|| {
        vec![
            ParamType::FixedArray(Box::new(ParamType::FixedBytes(32)), 3),
            ParamType::Bytes,
        ]
    });

    let schema_report_v3 = SCHEMA_REPORT_V3.get_or_init(|| {
        vec![
            ParamType::FixedBytes(32),
            ParamType::Uint(32),
            ParamType::Uint(32),
            ParamType::Uint(192),
            ParamType::Uint(192),
            ParamType::Uint(32),
            ParamType::Int(192),
            ParamType::Int(192),
            ParamType::Int(192),
        ]
    });

    let [_, Token::Bytes(data)]: [Token; 2] = ethabi::decode(schema_1, data)?
        .try_into()
        .map_err(|_| ethabi::Error::InvalidData)?
    else {
        return Err(ethabi::Error::InvalidData);
    };

    let version = ((data[0] as u16) << 8) | (data[1] as u16);

    if version != 3 {
        return Err(ethabi::Error::InvalidData);
    }

    let [
        Token::FixedBytes(feed_id),
        Token::Uint(valid_from_timestamp),
        Token::Uint(observations_timestamp),
        Token::Uint(native_fee),
        Token::Uint(link_fee),
        Token::Uint(expires_at),
        Token::Int(price),
        Token::Int(bid),
        Token::Int(ask),
    ]: [Token; 9] =
        ethabi::decode_whole(schema_report_v3, &data)?
            .try_into()
            .map_err(|_| ethabi::Error::InvalidData)?
    else {
        return Err(ethabi::Error::InvalidData);
    };

    Ok(Report {
        feed_id: feed_id.try_into().map_err(|_| ethabi::Error::InvalidData)?,
        valid_from_timestamp: valid_from_timestamp
            .try_into()
            .map_err(|_| ethabi::Error::InvalidData)?,
        observations_timestamp: observations_timestamp
            .try_into()
            .map_err(|_| ethabi::Error::InvalidData)?,
        native_fee: u256_to_u192(native_fee),
        link_fee: u256_to_u192(link_fee),
        expires_at: expires_at
            .try_into()
            .map_err(|_| ethabi::Error::InvalidData)?,
        price: u256_to_u192(price),
        bid: u256_to_u192(bid),
        ask: u256_to_u192(ask),
    })
}

fn u256_to_u192(num: U256) -> U192 {
    let inner = num.0;
    U192::from_limbs([inner[0], inner[1], inner[2]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() {
        let data = hex::decode(
            "\
        0006f3dad14cf5df26779bd7b940cd6a9b50ee226256194abbb7643655035d6f\
        0000000000000000000000000000000000000000000000000000000037a8ac19\
        0000000000000000000000000000000000000000000000000000000000000000\
        00000000000000000000000000000000000000000000000000000000000000e0\
        0000000000000000000000000000000000000000000000000000000000000220\
        0000000000000000000000000000000000000000000000000000000000000280\
        0101000000000000000000000000000000000000000000000000000000000000\
        0000000000000000000000000000000000000000000000000000000000000120\
        000305a183fedd7f783d99ac138950cff229149703d2a256d61227ad1e5e66ea\
        000000000000000000000000000000000000000000000000000000006726f480\
        000000000000000000000000000000000000000000000000000000006726f480\
        0000000000000000000000000000000000000000000000000000251afa5b7860\
        000000000000000000000000000000000000000000000000002063f8083c6714\
        0000000000000000000000000000000000000000000000000000000067284600\
        000000000000000000000000000000000000000000000000140f9559e8f303f4\
        000000000000000000000000000000000000000000000000140ede2b99374374\
        0000000000000000000000000000000000000000000000001410c8d592a7f800\
        0000000000000000000000000000000000000000000000000000000000000002\
        abc5fcd50a149ad258673b44c2d1737d175c134a29ab0e1091e1f591af564132\
        737fedd8929a5e6ee155532f116946351e79c1ea3efdb3c88792f48c7cbb02ca\
        0000000000000000000000000000000000000000000000000000000000000002\
        7a478e131ba1474e6b53f2c626ec349f27d64606b1e783d7cb637568ad3b0f7c\
        3ed29f3fd7de70dc2b08e010ab93448e7dd423047e0f224d7145e0489faa9f23",
        )
        .unwrap();
        let report = decode(&data).unwrap();
        println!("{}", u128::try_from(report.ask).unwrap());
        println!("{}", i64::from(report.valid_from_timestamp));
    }
}
