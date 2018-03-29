use errors::*;
use share::{IsShare, IsSignedShare};

// The order of validation that we think makes the most sense is the following:
// 1) Validate shares individually
// 2) Validate duplicate shares share num && data
// 2) Validate group consistency
// 3) Validate other properties, in no specific order

/// TODO: Doc
pub(crate) fn validate_signed_shares<S: IsSignedShare>(
    shares: &Vec<S>,
    verify_signatures: bool,
) -> Result<(u8, usize)> {
    let result = validate_shares(shares)?;

    if verify_signatures {
        S::verify_signatures(&shares)?;
    }

    Ok(result)
}

/// TODO: Doc
pub(crate) fn validate_shares<S: IsShare>(shares: &Vec<S>) -> Result<(u8, usize)> {
    if shares.is_empty() {
        bail!(ErrorKind::EmptyShares);
    }

    let shares_count = shares.len();

    let mut ids = Vec::with_capacity(shares_count);
    let mut threshold = 0;
    let mut slen = 0;

    for share in shares {
        let (id, threshold_, slen_) = (
            share.get_id(),
            share.get_threshold(),
            share.get_data().len(),
        );

        if id < 1 {
            bail!(ErrorKind::ShareParsingInvalidShareId(id))
        } else if threshold_ < 2 {
            bail!(ErrorKind::ShareParsingInvalidShareThreshold(threshold, id))
        } else if slen_ < 1 {
            bail!(ErrorKind::ShareParsingErrorEmptyShare(id))
        }

        if ids.iter().any(|&x| x == id) {
            bail!(ErrorKind::DuplicateShareId(id));
        }

        if threshold == 0 {
            threshold = threshold_;
        } else if threshold_ != threshold {
            bail!(ErrorKind::InconsistentThresholds(
                id,
                threshold_,
                ids,
                threshold
            ))
        }

        if slen == 0 {
            slen = slen_;
        } else if slen_ != slen {
            bail!(ErrorKind::InconsistentSecretLengths(id, slen_, ids, slen))
        }

        ids.push(id);
    }

    if shares_count < threshold as usize {
        bail!(ErrorKind::MissingShares(shares_count, threshold))
    }

    Ok((threshold, slen))
}

pub(crate) fn validate_share_count(threshold: u8, shares_count: u8) -> Result<(u8, u8)> {
    if threshold < MIN_SHARES {
        bail!(ErrorKind::ThresholdTooSmall(threshold));
    }
    if shares_count > MAX_SHARES {
        bail!(ErrorKind::InvalidShareCountMax(shares_count, MAX_SHARES));
    }
    if shares_count < MIN_SHARES {
        bail!(ErrorKind::InvalidShareCountMin(shares_count, MIN_SHARES));
    }
    if threshold > shares_count {
        bail!(ErrorKind::ThresholdTooBig(threshold, shares_count));
    }

    Ok((threshold, shares_count))
}
