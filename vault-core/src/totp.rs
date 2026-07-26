use hmac::{Hmac, Mac};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

/// Generates a 6-digit TOTP token using SHA-1 and 30-second time steps.
///
/// # Arguments
/// * `secret_base32` - The Base32-encoded secret key. Spacing and casing are normalized automatically.
/// * `timestamp` - Unix timestamp in seconds.
pub fn generate_totp(secret_base32: &str, timestamp: u64) -> Result<String, String> {
    let cleaned = secret_base32
        .replace(" ", "")
        .replace("-", "")
        .to_uppercase();

    let key = data_encoding::BASE32
        .decode(cleaned.as_bytes())
        .or_else(|_| data_encoding::BASE32_NOPAD.decode(cleaned.as_bytes()))
        .map_err(|e| format!("Invalid Base32 secret: {}", e))?;

    let counter = timestamp / 30;
    let counter_bytes = counter.to_be_bytes();

    let mut mac =
        HmacSha1::new_from_slice(&key).map_err(|e| format!("HMAC initialization failed: {}", e))?;
    mac.update(&counter_bytes);
    let result = mac.finalize();
    let code_bytes = result.into_bytes();

    let offset = (code_bytes[code_bytes.len() - 1] & 0xf) as usize;
    let code = ((code_bytes[offset] & 0x7f) as u32) << 24
        | (code_bytes[offset + 1] as u32) << 16
        | (code_bytes[offset + 2] as u32) << 8
        | (code_bytes[offset + 3] as u32);

    let totp = code % 1_000_000;
    Ok(format!("{:06}", totp))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfc_6238_test_vectors() {
        // RFC 6238 test vectors with SHA-1.
        // Secret ASCII: "12345678901234567890" -> Base32: "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"
        let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

        // T = 59 (counter 1)
        assert_eq!(generate_totp(secret, 59).unwrap(), "287082");

        // T = 1111111109
        assert_eq!(generate_totp(secret, 1111111109).unwrap(), "081804");

        // T = 1111111111
        assert_eq!(generate_totp(secret, 1111111111).unwrap(), "050471");

        // T = 1234567890
        assert_eq!(generate_totp(secret, 1234567890).unwrap(), "005924");

        // T = 2000000000
        assert_eq!(generate_totp(secret, 2000000000).unwrap(), "279037");
    }

    #[test]
    fn test_lowercase_and_spaces() {
        let secret = "gezd gnbv gy3t qojq gezd gnbv gy3t qojq";
        assert_eq!(generate_totp(secret, 59).unwrap(), "287082");
    }

    #[test]
    fn test_unpadded_base32() {
        // "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ" has length 32, so it's fully padded anyway.
        // Let's use a 10 byte secret (Base32 length 16)
        // ASCII "1234567890" -> Base32 "GEZDGNBVGY3TQOJQ"
        // Let's truncate padding/bytes to see if unpadded works.
        // ASCII "12345" (5 bytes) -> Base32 padded "GEZDGNBV" (8 chars)
        // ASCII "1234" (4 bytes) -> Base32 padded "GEZDGNBS" (8 chars with padding/partial)
        // Wait, standard Base32 encoded "1234" is "GEZDGNBS" with length 8 (no padding '=' needed if it aligns, or maybe it does).
        // Let's test with a secret that is not 8-char aligned to verify.
        // e.g. "MZXW6YTBOI" -> 10 chars, normally padded to 16 with ======
        let secret_padded = "MZXW6YTBOI======";
        let secret_unpadded = "MZXW6YTBOI";

        let token1 = generate_totp(secret_padded, 1000).unwrap();
        let token2 = generate_totp(secret_unpadded, 1000).unwrap();
        assert_eq!(token1, token2);
    }
}
