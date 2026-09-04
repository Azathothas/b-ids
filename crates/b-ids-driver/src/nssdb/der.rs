//! The four fields of a certificate that NSS indexes, read off the DER.
//!
//! ⛔ **Read, never re-encoded.** NSS matches a trust object to a certificate
//! by comparing `CKA_ISSUER` and `CKA_SERIAL_NUMBER` byte for byte against
//! what it decoded from the certificate itself, so a field re-serialised from
//! a parsed structure is a field that matches nothing. Every value this module
//! returns is a slice of the input, copied and not rebuilt.
//!
//! ⚠ **`CKA_SERIAL_NUMBER` is the whole INTEGER, tag and length included.**
//! It looks like an off-by-two and it is the format: NSS stores the DER
//! element rather than the number. A value stored without its header is
//! accepted by every write and found by no lookup.
//!
//! ⛔ **It parses permissively and refuses loudly.** Anything it cannot walk is
//! an error naming the field, because a certificate this cannot read is a
//! capture that must not be attempted rather than one taken with a profile
//! that trusts nothing.
//!
//! `TODO/driver.md`, `DRIVER-11`.

/// The `rsaEncryption` algorithm identifier, `1.2.840.113549.1.1.1`.
const OID_RSA_ENCRYPTION: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01];

/// What one certificate carries that a certificate database indexes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fields {
    /// The issuer `Name`, as the complete DER element.
    pub issuer: Vec<u8>,
    /// The subject `Name`, as the complete DER element.
    pub subject: Vec<u8>,
    /// The serial number, as the complete DER `INTEGER` element.
    pub serial: Vec<u8>,
    /// The bytes NSS hashes to make `CKA_ID`.
    ///
    /// ⚠ **The rule is the algorithm's, and this follows it for the two
    /// families that reach it.** `PK11_GetPubIndexKeyID` in
    /// `references/mozilla__nss/tree/lib/pk11wrap/pk11cert.c:1121` takes the
    /// public value: for an elliptic-curve key that is the point, and for RSA
    /// it is the modulus alone rather than the encoded key. Anything else
    /// falls back to the whole public key bit string, ⛔ **which is not what
    /// NSS would compute.** Nothing in this project reads `CKA_ID`: a trust
    /// object is found by issuer and serial and a certificate by subject, so
    /// the difference is recorded rather than guessed at.
    pub public_value: Vec<u8>,
}

/// One DER element: its tag, and where its content sits in the buffer.
struct Element<'a> {
    tag: u8,
    /// The whole element, header included.
    whole: &'a [u8],
    /// The content, header excluded.
    content: &'a [u8],
}

/// Read the element at the start of `input`.
fn element<'a>(input: &'a [u8], what: &str) -> Result<Element<'a>, String> {
    if input.len() < 2 {
        return Err(format!("{what}: {} byte(s) is not an element", input.len()));
    }
    let tag = input[0];
    let first = input[1];
    let (header, length) = if first < 0x80 {
        (2_usize, first as usize)
    } else if first == 0x80 {
        // ⛔ Indefinite length is BER and not DER, and a certificate carrying
        // it is one this project must not guess at.
        return Err(format!("{what}: indefinite length is not DER"));
    } else {
        let count = (first & 0x7f) as usize;
        if count > 4 {
            return Err(format!(
                "{what}: a {count}-byte length is beyond this reader"
            ));
        }
        if input.len() < 2 + count {
            return Err(format!("{what}: the length runs past the end"));
        }
        let mut value = 0_usize;
        for byte in &input[2..2 + count] {
            value = (value << 8) | *byte as usize;
        }
        (2 + count, value)
    };
    let end = header
        .checked_add(length)
        .ok_or_else(|| format!("{what}: the length overflows"))?;
    if end > input.len() {
        return Err(format!(
            "{what}: an element of {end} byte(s) in a buffer of {}",
            input.len()
        ));
    }
    Ok(Element {
        tag,
        whole: &input[..end],
        content: &input[header..end],
    })
}

/// Read the element at the start of `input`, and return the rest with it.
fn next<'a>(input: &'a [u8], what: &str) -> Result<(Element<'a>, &'a [u8]), String> {
    let element = element(input, what)?;
    let rest = &input[element.whole.len()..];
    Ok((element, rest))
}

/// The fields a certificate database indexes, from one DER certificate.
///
/// # Errors
///
/// A string naming the field that could not be read.
pub fn fields(certificate: &[u8]) -> Result<Fields, String> {
    let outer = element(certificate, "certificate")?;
    if outer.tag != 0x30 {
        return Err(format!(
            "certificate: tag {:#04x} is not a SEQUENCE",
            outer.tag
        ));
    }
    let (tbs, _) = next(outer.content, "tbsCertificate")?;
    if tbs.tag != 0x30 {
        return Err(format!(
            "tbsCertificate: tag {:#04x} is not a SEQUENCE",
            tbs.tag
        ));
    }
    let mut rest = tbs.content;
    // ⚠ The version is [0] EXPLICIT and DEFAULT v1, so it is absent on a v1
    // certificate and every field after it shifts by one. Reading by position
    // without this check reads the serial number out of the version.
    if rest.first() == Some(&0xa0) {
        let (_, after) = next(rest, "version")?;
        rest = after;
    }
    let (serial, rest) = next(rest, "serialNumber")?;
    if serial.tag != 0x02 {
        return Err(format!(
            "serialNumber: tag {:#04x} is not an INTEGER",
            serial.tag
        ));
    }
    let (_, rest) = next(rest, "signature")?;
    let (issuer, rest) = next(rest, "issuer")?;
    let (_, rest) = next(rest, "validity")?;
    let (subject, rest) = next(rest, "subject")?;
    let (spki, _) = next(rest, "subjectPublicKeyInfo")?;

    let (algorithm, after_algorithm) = next(spki.content, "algorithm")?;
    let (oid, _) = next(algorithm.content, "algorithm.oid")?;
    let (key, _) = next(after_algorithm, "subjectPublicKey")?;
    if key.tag != 0x03 {
        return Err(format!(
            "subjectPublicKey: tag {:#04x} is not a BIT STRING",
            key.tag
        ));
    }
    let bits = key
        .content
        .split_first()
        .map(|(_unused, rest)| rest)
        .ok_or("subjectPublicKey: an empty BIT STRING")?;
    // ⚠ RSA IS THE ONLY FAMILY WHOSE PUBLIC VALUE IS NOT THE BIT STRING. NSS
    // takes the modulus alone for `rsaEncryption` and the encoded public value
    // for everything else, so `id-ecPublicKey`, 1.2.840.10045.2.1, which is
    // what this project's own authority carries, falls to the branch below and
    // is correct there. ⛔ An algorithm that is neither is ALSO handled there
    // and the value is then not what NSS would compute. It is left permissive
    // rather than refused because nothing here reads `CKA_ID`, and a refusal
    // would be a wall in front of the first authority this project mints with
    // some other key.
    let public_value = if oid.content == OID_RSA_ENCRYPTION {
        let (rsa, _) = next(bits, "RSAPublicKey")?;
        let (modulus, _) = next(rsa.content, "RSAPublicKey.modulus")?;
        modulus.content.to_vec()
    } else {
        bits.to_vec()
    };

    Ok(Fields {
        issuer: issuer.whole.to_vec(),
        subject: subject.whole.to_vec(),
        serial: serial.whole.to_vec(),
        public_value,
    })
}

/// The first certificate in a PEM document, as DER.
///
/// # Errors
///
/// A string naming what was missing or would not decode.
pub fn from_pem(pem: &str) -> Result<Vec<u8>, String> {
    const BEGIN: &str = "-----BEGIN CERTIFICATE-----";
    const END: &str = "-----END CERTIFICATE-----";
    let start = pem
        .find(BEGIN)
        .ok_or("no BEGIN CERTIFICATE line in the authority")?
        + BEGIN.len();
    let end = pem[start..]
        .find(END)
        .ok_or("no END CERTIFICATE line in the authority")?
        + start;
    unbase64(&pem[start..end])
}

/// Decode base64, ignoring the line breaks a PEM document carries.
fn unbase64(text: &str) -> Result<Vec<u8>, String> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let mut accumulator: u32 = 0;
    let mut bits = 0_u32;
    for byte in text.bytes() {
        if byte.is_ascii_whitespace() || byte == b'=' {
            continue;
        }
        let value = ALPHABET
            .iter()
            .position(|c| *c == byte)
            .ok_or_else(|| format!("base64: {:#04x} is not in the alphabet", byte))?;
        accumulator = (accumulator << 6) | value as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((accumulator >> bits) & 0xff) as u8);
        }
    }
    if out.is_empty() {
        return Err("base64: the body decoded to nothing".to_owned());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A v3 certificate: SEQUENCE { tbs, alg, sig } where tbs carries [0],
    /// serial 0x0102, an algorithm, an issuer, a validity, a subject and an
    /// EC public key. Small, hand-built, and every field distinguishable.
    fn fixture() -> Vec<u8> {
        fn tlv(tag: u8, content: &[u8]) -> Vec<u8> {
            let mut out = vec![tag, content.len() as u8];
            out.extend_from_slice(content);
            out
        }
        let version = tlv(0xa0, &tlv(0x02, &[0x02]));
        let serial = tlv(0x02, &[0x01, 0x02]);
        let sigalg = tlv(0x30, &tlv(0x06, &[0x2a, 0x86, 0x48]));
        let issuer = tlv(0x30, &tlv(0x31, b"issuer"));
        let validity = tlv(0x30, &tlv(0x17, b"whenever"));
        let subject = tlv(0x30, &tlv(0x31, b"subject"));
        // id-ecPublicKey, 1.2.840.10045.2.1.
        let ec = [0x2a_u8, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01];
        let mut spki_body = tlv(0x30, &tlv(0x06, &ec));
        spki_body.extend_from_slice(&tlv(0x03, &[0x00, 0x04, 0xaa, 0xbb]));
        let spki = tlv(0x30, &spki_body);
        let mut tbs = Vec::new();
        for part in [
            &version, &serial, &sigalg, &issuer, &validity, &subject, &spki,
        ] {
            tbs.extend_from_slice(part);
        }
        let mut body = tlv(0x30, &tbs);
        body.extend_from_slice(&tlv(0x30, &tlv(0x06, &[0x2a])));
        body.extend_from_slice(&tlv(0x03, &[0x00, 0xff]));
        tlv(0x30, &body)
    }

    #[test]
    fn the_serial_number_keeps_its_der_header() {
        let got = fields(&fixture()).expect("the fixture did not parse");
        assert_eq!(got.serial, vec![0x02, 0x02, 0x01, 0x02]);
    }

    #[test]
    fn the_issuer_and_subject_are_whole_elements_and_are_not_swapped() {
        let got = fields(&fixture()).expect("the fixture did not parse");
        assert_eq!(
            got.issuer,
            vec![0x30, 0x08, 0x31, 0x06, b'i', b's', b's', b'u', b'e', b'r']
        );
        assert_eq!(
            got.subject,
            vec![
                0x30, 0x09, 0x31, 0x07, b's', b'u', b'b', b'j', b'e', b'c', b't'
            ]
        );
    }

    #[test]
    fn an_ec_public_value_is_the_point_without_the_unused_bit_count() {
        let got = fields(&fixture()).expect("the fixture did not parse");
        assert_eq!(got.public_value, vec![0x04, 0xaa, 0xbb]);
    }

    #[test]
    fn a_version_1_certificate_does_not_lose_a_field_to_the_absent_version() {
        // The same fixture with the [0] element removed, which is what a v1
        // certificate is. Every field after it shifts, so a reader that skips
        // unconditionally reads the algorithm as the serial.
        let whole = fixture();
        let outer = element(&whole, "certificate").unwrap();
        let tbs = element(outer.content, "tbs").unwrap();
        let without = &tbs.content[5..];
        let mut rebuilt = vec![0x30, without.len() as u8];
        rebuilt.extend_from_slice(without);
        let got = fields(&{
            let mut body = rebuilt.clone();
            body.extend_from_slice(&[0x30, 0x03, 0x06, 0x01, 0x2a, 0x03, 0x02, 0x00, 0xff]);
            let mut out = vec![0x30, body.len() as u8];
            out.extend_from_slice(&body);
            out
        })
        .expect("the v1 fixture did not parse");
        assert_eq!(got.serial, vec![0x02, 0x02, 0x01, 0x02]);
    }

    #[test]
    fn a_truncated_certificate_is_refused_by_name() {
        let whole = fixture();
        let err = fields(&whole[..whole.len() / 2]).expect_err("a truncated certificate parsed");
        assert!(err.contains("certificate"), "unexpected refusal: {err}");
    }

    #[test]
    fn a_pem_body_decodes_and_a_missing_header_is_refused() {
        let der = fixture();
        let mut body = String::new();
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut bits = 0_u32;
        let mut accumulator = 0_u32;
        for byte in &der {
            accumulator = (accumulator << 8) | u32::from(*byte);
            bits += 8;
            while bits >= 6 {
                bits -= 6;
                body.push(ALPHABET[((accumulator >> bits) & 0x3f) as usize] as char);
            }
        }
        if bits > 0 {
            body.push(ALPHABET[((accumulator << (6 - bits)) & 0x3f) as usize] as char);
        }
        let pem = format!("-----BEGIN CERTIFICATE-----\n{body}\n-----END CERTIFICATE-----\n");
        assert_eq!(from_pem(&pem).expect("the PEM did not decode"), der);
        let err = from_pem("nothing here").expect_err("a PEM with no header decoded");
        assert!(err.contains("BEGIN"), "unexpected refusal: {err}");
    }
}
