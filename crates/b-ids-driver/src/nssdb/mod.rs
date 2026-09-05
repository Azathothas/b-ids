//! Seed a Gecko profile's certificate database with one trusted authority.
//!
//! ⭐ **This is what stands between the corpus and its first non-Chromium
//! profile.** Chromium takes `--ignore-certificate-errors-spki-list` on the
//! command line and Firefox takes no equivalent, so a capture against this
//! project's own terminator cannot be arranged by adding arguments. The trust
//! has to be arranged where Firefox looks for it, which is the NSS
//! certificate database inside the profile.
//!
//! ⛔ **Two objects, and one of them is the whole point.** A certificate
//! object alone is a certificate the browser knows and does not trust. The
//! trust is a second object, of class `CKO_NSS_TRUST`, found by issuer and
//! serial number:
//! [`nssToken_FindTrustForCertificate`](https://github.com/mozilla/nss/blob/7db8de42431841b214b49fd2cb7122a07aa631b8/lib/dev/devtoken.c#L1124) builds that lookup
//! from `CKA_TOKEN`, `CKA_CLASS`, `CKA_ISSUER` and `CKA_SERIAL_NUMBER` and
//! nothing else.
//!
//! ⚠ **A trust record without the certificate's SHA-1 is discarded in
//! silence.** [`nssTrust_Create`](https://github.com/mozilla/nss/blob/7db8de42431841b214b49fd2cb7122a07aa631b8/lib/pki/certificate.c#L1022) accepts a record
//! with no hash only when `nssTrust_IsSafeToIgnoreCertHash` says every purpose
//! on it is unknown or distrusted, so a delegator record must carry
//! `CKA_NSS_CERT_SHA1_HASH` and it must match. Nothing reports the mismatch:
//! the browser simply does not trust the authority.
//!
//! All NSS citations use commit `7db8de42431841b214b49fd2cb7122a07aa631b8`.
//!
//! `docs/history/todo/driver.md`, `DRIVER-11`.

pub mod der;
pub mod sha1;
pub mod sqlite;

use std::path::{Path, PathBuf};

use sqlite::{Table, Value};

/// `CKO_CERTIFICATE`, from `pkcs11t.h`.
const CKO_CERTIFICATE: u32 = 0x0000_0001;

/// `CKO_NSS_TRUST`, which is `CKO_VENDOR_DEFINED | NSSCK_VENDOR_NSS` plus 3.
///
/// [`pkcs11n.h`](https://github.com/mozilla/nss/blob/7db8de42431841b214b49fd2cb7122a07aa631b8/lib/util/pkcs11n.h#L124).
const CKO_NSS_TRUST: u32 = 0xce53_4353;

/// `CKC_X_509`, the only certificate type this writes.
const CKC_X_509: u32 = 0x0000_0000;

/// `CKT_NSS_TRUSTED_DELEGATOR`: an authority that may issue for this purpose.
///
/// [`pkcs11n.h`](https://github.com/mozilla/nss/blob/7db8de42431841b214b49fd2cb7122a07aa631b8/lib/util/pkcs11n.h#L608), and
/// [`get_nss_trust`](https://github.com/mozilla/nss/blob/7db8de42431841b214b49fd2cb7122a07aa631b8/lib/dev/ckhelper.c#L374) turns it into
/// the trust level a chain builder reads.
const CKT_NSS_TRUSTED_DELEGATOR: u32 = 0xce53_4352;

/// `CKT_NSS_MUST_VERIFY_TRUST`: no opinion, defer to a chain.
///
/// [`pkcs11n.h`](https://github.com/mozilla/nss/blob/7db8de42431841b214b49fd2cb7122a07aa631b8/lib/util/pkcs11n.h#L609).
const CKT_NSS_MUST_VERIFY_TRUST: u32 = 0xce53_4353;

/// The table a certificate database keeps its public objects in.
const TABLE: &str = "nssPublic";

/// The columns this writer populates, in the order NSS declares them.
///
/// ⚠ **A subset, deliberately.** [`sdb_update_column`](https://github.com/mozilla/nss/blob/7db8de42431841b214b49fd2cb7122a07aa631b8/lib/softoken/sdb.c#L2001) asks SQLite for the
/// column names of the table it opened and adds every known attribute it does
/// not find, so a narrow table is completed by NSS on first write rather than
/// rejected. Writing a column NSS does not know would not be.
const COLUMNS: &[&str] = &[
    "a0",        // CKA_CLASS
    "a1",        // CKA_TOKEN
    "a2",        // CKA_PRIVATE
    "a3",        // CKA_LABEL
    "a11",       // CKA_VALUE
    "a80",       // CKA_CERTIFICATE_TYPE
    "a81",       // CKA_ISSUER
    "a82",       // CKA_SERIAL_NUMBER
    "a101",      // CKA_SUBJECT
    "a102",      // CKA_ID
    "a170",      // CKA_MODIFIABLE
    "ace536358", // CKA_NSS_TRUST_SERVER_AUTH
    "ace536359", // CKA_NSS_TRUST_CLIENT_AUTH
    "ace53635a", // CKA_NSS_TRUST_CODE_SIGNING
    "ace53635b", // CKA_NSS_TRUST_EMAIL_PROTECTION
    "ace536360", // CKA_NSS_TRUST_STEP_UP_APPROVED
    "ace5363b4", // CKA_NSS_CERT_SHA1_HASH
];

/// Where a named column sits in [`COLUMNS`].
fn column(name: &str) -> usize {
    COLUMNS
        .iter()
        .position(|c| *c == name)
        .expect("every column written is declared in COLUMNS")
}

/// A `CK_ULONG` as the certificate database stores one.
///
/// ⛔ **Four bytes, big-endian, whatever the machine is.**
/// [`sftk_ULong2SDBULong`](https://github.com/mozilla/nss/blob/7db8de42431841b214b49fd2cb7122a07aa631b8/lib/softoken/sftkdb.c#L149) writes exactly
/// that, and [`SDB_ULONG_SIZE`](https://github.com/mozilla/nss/blob/7db8de42431841b214b49fd2cb7122a07aa631b8/lib/softoken/sftkdbt.h#L10) is 4. A native
/// `CK_ULONG` written instead is eight little-endian bytes, which NSS reads as
/// a blob of the wrong length and hands back unconverted.
fn ck_ulong(value: u32) -> Value {
    Value::Blob(value.to_be_bytes().to_vec())
}

/// A `CK_BBOOL`.
fn ck_bool(value: bool) -> Value {
    Value::Blob(vec![u8::from(value)])
}

/// What one seeding wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seeded {
    /// The certificate database that was created.
    pub cert9: PathBuf,
    /// The SHA-1 of the authority certificate, which is what ties the trust
    /// record to it.
    pub sha1: [u8; sha1::LEN],
    /// The nickname the certificate carries in the database.
    pub nickname: String,
}

/// The bytes of a `cert9.db` trusting `certificate` to issue for servers.
///
/// ⚠ **Server authority only.** Client authentication, code signing and mail
/// are written as `CKT_NSS_MUST_VERIFY_TRUST`, which is no opinion rather than
/// trust: a capture needs the browser to complete one server handshake, and a
/// profile that trusted more than the measurement needs would be a profile
/// measuring something else.
///
/// # Errors
///
/// A string naming the field that could not be read or the row that would not
/// fit a page.
pub fn cert9(certificate: &[u8], nickname: &str) -> Result<Vec<u8>, String> {
    let fields = der::fields(certificate)?;
    let digest = sha1::sha1(certificate);

    let mut cert_row = vec![Value::Null; COLUMNS.len()];
    cert_row[column("a0")] = ck_ulong(CKO_CERTIFICATE);
    cert_row[column("a1")] = ck_bool(true);
    cert_row[column("a2")] = ck_bool(false);
    cert_row[column("a3")] = Value::Blob(nickname.as_bytes().to_vec());
    cert_row[column("a11")] = Value::Blob(certificate.to_vec());
    cert_row[column("a80")] = ck_ulong(CKC_X_509);
    cert_row[column("a81")] = Value::Blob(fields.issuer.clone());
    cert_row[column("a82")] = Value::Blob(fields.serial.clone());
    cert_row[column("a101")] = Value::Blob(fields.subject);
    cert_row[column("a102")] = Value::Blob(sha1::sha1(&fields.public_value).to_vec());
    cert_row[column("a170")] = ck_bool(true);

    let mut trust_row = vec![Value::Null; COLUMNS.len()];
    trust_row[column("a0")] = ck_ulong(CKO_NSS_TRUST);
    trust_row[column("a1")] = ck_bool(true);
    trust_row[column("a2")] = ck_bool(false);
    trust_row[column("a3")] = Value::Blob(nickname.as_bytes().to_vec());
    trust_row[column("a81")] = Value::Blob(fields.issuer);
    trust_row[column("a82")] = Value::Blob(fields.serial);
    trust_row[column("a170")] = ck_bool(true);
    trust_row[column("ace536358")] = ck_ulong(CKT_NSS_TRUSTED_DELEGATOR);
    trust_row[column("ace536359")] = ck_ulong(CKT_NSS_MUST_VERIFY_TRUST);
    trust_row[column("ace53635a")] = ck_ulong(CKT_NSS_MUST_VERIFY_TRUST);
    trust_row[column("ace53635b")] = ck_ulong(CKT_NSS_MUST_VERIFY_TRUST);
    // ⚠ A CK_BBOOL rather than a CK_ULONG, because that is the width
    // [`nssCryptokiTrust_GetAttributes`](https://github.com/mozilla/nss/blob/7db8de42431841b214b49fd2cb7122a07aa631b8/lib/dev/ckhelper.c#L445)
    // reads it into.
    trust_row[column("ace536360")] = ck_bool(false);
    trust_row[column("ace5363b4")] = Value::Blob(digest.to_vec());

    // ⛔ IDS 1 AND 2, NOT RANDOM ONES. NSS draws a free handle in 1..2^30 and
    // checks the table first in
    // [`sdb_getObjectId`](https://github.com/mozilla/nss/blob/7db8de42431841b214b49fd2cb7122a07aa631b8/lib/softoken/sdb.c#L1292), so a table this
    // writer creates from nothing has nothing to collide with and a generator
    // here would be a second source of a value that has to be unique.
    let table = Table {
        name: TABLE.to_owned(),
        columns: COLUMNS.iter().map(|c| (*c).to_owned()).collect(),
        rows: vec![(1, cert_row), (2, trust_row)],
    };
    sqlite::database(&[table])
}

/// Write a `cert9.db` into `profile` trusting the authority in `ca_pem`.
///
/// ⛔ **It refuses a profile that already has one.** A certificate database is
/// where a browser keeps what its user chose to trust, and overwriting one is
/// not recoverable. This is only ever pointed at a directory the driver
/// created for one launch and removes afterwards.
///
/// # Errors
///
/// A string naming what was refused, what would not parse, or which write
/// failed.
pub fn seed(profile: &Path, ca_pem: &str, nickname: &str) -> Result<Seeded, String> {
    let path = profile.join("cert9.db");
    if path.exists() {
        return Err(format!(
            "{} already exists: this seeds a profile it created, never one that \
             carries somebody's own certificates",
            path.display()
        ));
    }
    let der = der::from_pem(ca_pem)?;
    let bytes = cert9(&der, nickname)?;
    std::fs::create_dir_all(profile).map_err(|e| format!("{}: {e}", profile.display()))?;
    std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    // ⚠ key4.db IS NOT WRITTEN, and that is deliberate. Firefox creates it
    // with a metaData table carrying the empty-password check, and a profile
    // that differs from an ordinary one by more than the added authority is a
    // profile measuring something else. Measured 2026-09-04 on Firefox
    // 148.0.2: a profile started with no files at all is left holding
    // cert9.db with only `nssPublic`, and key4.db with `nssPrivate` and
    // `metaData`.
    Ok(Seeded {
        cert9: path,
        sha1: sha1::sha1(&der),
        nickname: nickname.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal certificate, enough for the fields the database indexes.
    fn certificate() -> Vec<u8> {
        fn tlv(tag: u8, content: &[u8]) -> Vec<u8> {
            let mut out = vec![tag, content.len() as u8];
            out.extend_from_slice(content);
            out
        }
        let mut tbs = tlv(0xa0, &tlv(0x02, &[0x02]));
        tbs.extend_from_slice(&tlv(0x02, &[0x2a]));
        tbs.extend_from_slice(&tlv(0x30, &tlv(0x06, &[0x2a, 0x86, 0x48])));
        tbs.extend_from_slice(&tlv(0x30, &tlv(0x31, b"an issuer")));
        tbs.extend_from_slice(&tlv(0x30, &tlv(0x17, b"whenever")));
        tbs.extend_from_slice(&tlv(0x30, &tlv(0x31, b"a subject")));
        let mut spki = tlv(
            0x30,
            &tlv(0x06, &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01]),
        );
        spki.extend_from_slice(&tlv(0x03, &[0x00, 0x04, 0x11, 0x22]));
        tbs.extend_from_slice(&tlv(0x30, &spki));
        let mut body = tlv(0x30, &tbs);
        body.extend_from_slice(&tlv(0x30, &tlv(0x06, &[0x2a])));
        body.extend_from_slice(&tlv(0x03, &[0x00, 0xff]));
        tlv(0x30, &body)
    }

    #[test]
    fn the_database_carries_a_certificate_and_a_trust_object() {
        let bytes = cert9(&certificate(), "an authority").expect("the fixture was refused");
        let text: Vec<u8> = bytes.clone();
        // The class values are four big-endian bytes, so both objects are
        // findable in the file by the class they declare.
        assert!(
            text.windows(4).any(|w| w == CKO_CERTIFICATE.to_be_bytes()),
            "no CKO_CERTIFICATE object in the file"
        );
        assert!(
            text.windows(4).any(|w| w == CKO_NSS_TRUST.to_be_bytes()),
            "no CKO_NSS_TRUST object in the file"
        );
        assert!(
            text.windows(4)
                .any(|w| w == CKT_NSS_TRUSTED_DELEGATOR.to_be_bytes()),
            "no delegator trust value in the file"
        );
    }

    #[test]
    fn the_trust_object_carries_the_certificate_sha1() {
        let der = certificate();
        let bytes = cert9(&der, "an authority").expect("the fixture was refused");
        let digest = sha1::sha1(&der);
        assert!(
            bytes.windows(sha1::LEN).any(|w| w == digest),
            "the trust record does not carry the certificate hash, so NSS would discard it"
        );
    }

    #[test]
    fn a_profile_that_already_has_a_certificate_database_is_refused() {
        let dir = std::env::temp_dir().join(format!("b-ids-nssdb-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("the scratch directory was not created");
        let path = dir.join("cert9.db");
        std::fs::write(&path, b"not ours").expect("the fixture was not written");
        let err = seed(&dir, "irrelevant", "x").expect_err("an existing database was overwritten");
        assert!(err.contains("already exists"), "unexpected refusal: {err}");
        assert_eq!(
            std::fs::read(&path).expect("the fixture disappeared"),
            b"not ours",
            "the refused seeding still wrote"
        );
        std::fs::remove_dir_all(&dir).expect("the scratch directory was not removed");
    }
}
