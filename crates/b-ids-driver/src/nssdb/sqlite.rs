//! Write a complete SQLite database file, for the one case this project needs.
//!
//! ⭐ **It creates a database. It cannot open one, and that is the whole
//! reason it is small enough to be correct.** A certificate database is built
//! from nothing into a profile directory nobody keeps, so there is no existing
//! file to parse, no page to split, no freelist to walk and no journal to
//! replay. What is left is the file header, one b-tree leaf per table, and the
//! record encoding.
//!
//! ⛔ **The reader is NSS's own bundled SQLite**, so the format is theirs
//! rather than a choice made here. Every field below is the one named in
//! `https://sqlite.org/fileformat2.html`, and
//! `references/mozilla__nss/tree/lib/sqlite/` at commit
//! `7db8de42431841b214b49fd2cb7122a07aa631b8` is the copy that will read what
//! this writes.
//!
//! ⚠ **A row that would need an overflow page is REFUSED**, not truncated and
//! not silently split. Overflow is the one part of the format this writer does
//! not implement, so it fails loudly at the boundary rather than producing a
//! file whose first page reads and whose second does not exist.
//!
//! `docs/history/todo/driver.md`, `DRIVER-11`.

/// The page size every database this writer produces uses.
///
/// ⚠ **8192 rather than the 4096 default, and the reason is the refusal
/// above.** A certificate row carries the whole DER, and the inline ceiling is
/// `PAGE_SIZE - 35`. At 4096 that is 4061 bytes, which an RSA-4096 authority
/// with long names approaches; at 8192 it is 8157 and nothing this project
/// mints comes close. The file costs three pages either way.
const PAGE_SIZE: usize = 8192;

/// The largest cell payload that can live inside a table leaf page.
///
/// ⛔ From the format: `usable size - 35`, where the usable size is the page
/// size because this writer reserves no per-page space.
const MAX_INLINE_PAYLOAD: usize = PAGE_SIZE - 35;

/// The version this writer records as having last written the file.
///
/// ⚠ **A record, not a claim about behaviour.** SQLite stores it and does not
/// act on it. It is written because a zero there is a file no tool can
/// attribute.
const SQLITE_VERSION_NUMBER: u32 = 3_045_000;

/// One value in one column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// No value in this column.
    Null,
    /// A signed integer, stored in the narrowest serial type that holds it.
    Int(i64),
    /// Bytes, stored as they are.
    Blob(Vec<u8>),
    /// UTF-8 text.
    Text(String),
}

/// One table: its name, its column list, and its rows.
///
/// ⛔ **The `CREATE TABLE` statement is derived from this, never given
/// alongside it.** A statement and a row width supplied separately are a value
/// in two places with no check between them, and the failure is silent: SQLite
/// reads the row against the declared columns and every value lands one column
/// to the left. That defect was written here and caught by reading the file
/// back with `sqlite3`, which is why the shape changed.
#[derive(Debug, Clone)]
pub struct Table {
    /// The table name.
    pub name: String,
    /// The columns after the row id, in order.
    ///
    /// ⛔ **NSS reads these back.** `sdb_update_column` in
    /// `references/mozilla__nss/tree/lib/softoken/sdb.c:2001` asks SQLite for
    /// the column names of the table it opened and adds every attribute column
    /// it does not find, so this list decides what NSS has to add rather than
    /// what it will accept.
    pub columns: Vec<String>,
    /// The rows: a row id, and one value per entry in [`Table::columns`].
    pub rows: Vec<(i64, Vec<Value>)>,
}

impl Table {
    /// The statement the schema records for this table.
    ///
    /// ⚠ **`id INTEGER PRIMARY KEY` rather than NSS's own
    /// `id PRIMARY KEY UNIQUE ON CONFLICT ABORT`.** The second is not an
    /// integer primary key, so SQLite builds an implicit index for it and a
    /// writer that declared it would have to write that index's b-tree too.
    /// Making the column the row id keeps the uniqueness NSS relies on, since
    /// row ids are unique by construction, and NSS reads column names rather
    /// than this text. Measured 2026-09-04 with sqlite3 3.53.4: the first form
    /// creates no index and the second creates `sqlite_autoindex_nssPublic_1`.
    fn sql(&self) -> String {
        format!(
            "CREATE TABLE {} (id INTEGER PRIMARY KEY, {})",
            self.name,
            self.columns.join(", ")
        )
    }
}

/// Encode `value` as a SQLite record serial type and its bytes.
fn serial(value: &Value) -> (u64, Vec<u8>) {
    match value {
        Value::Null => (0, Vec::new()),
        Value::Int(n) => {
            let n = *n;
            if (-128..=127).contains(&n) {
                (1, vec![n as i8 as u8])
            } else if (-32_768..=32_767).contains(&n) {
                (2, (n as i16).to_be_bytes().to_vec())
            } else if (-8_388_608..=8_388_607).contains(&n) {
                (3, (n as i32).to_be_bytes()[1..].to_vec())
            } else if (-2_147_483_648..=2_147_483_647).contains(&n) {
                (4, (n as i32).to_be_bytes().to_vec())
            } else if (-140_737_488_355_328..=140_737_488_355_327).contains(&n) {
                (5, n.to_be_bytes()[2..].to_vec())
            } else {
                (6, n.to_be_bytes().to_vec())
            }
        }
        Value::Blob(bytes) => (12 + 2 * bytes.len() as u64, bytes.clone()),
        Value::Text(text) => (13 + 2 * text.len() as u64, text.as_bytes().to_vec()),
    }
}

/// Append `value` to `out` as a SQLite variable-length integer.
///
/// ⚠ **Big-endian, seven bits a byte, and the ninth byte carries eight.** The
/// last part is the one a from-memory implementation gets wrong, and it is
/// unreachable for the values this writer produces, which is exactly why it is
/// written rather than asserted away.
fn varint(out: &mut Vec<u8>, value: u64) {
    if value > 0x00ff_ffff_ffff_ffff {
        let mut bytes = [0_u8; 9];
        bytes[8] = (value & 0xff) as u8;
        let mut rest = value >> 8;
        for slot in bytes[..8].iter_mut().rev() {
            *slot = (rest & 0x7f) as u8 | 0x80;
            rest >>= 7;
        }
        out.extend_from_slice(&bytes);
        return;
    }
    let mut stack = [0_u8; 8];
    let mut used = 0;
    let mut rest = value;
    loop {
        stack[used] = (rest & 0x7f) as u8;
        used += 1;
        rest >>= 7;
        if rest == 0 {
            break;
        }
    }
    for i in (0..used).rev() {
        let last = i == 0;
        out.push(if last { stack[i] } else { stack[i] | 0x80 });
    }
}

/// The number of bytes [`varint`] would write for `value`.
fn varint_len(value: u64) -> usize {
    let mut probe = Vec::new();
    varint(&mut probe, value);
    probe.len()
}

/// Encode one row's values as a SQLite record.
fn record(values: &[Value]) -> Vec<u8> {
    let parts: Vec<(u64, Vec<u8>)> = values.iter().map(serial).collect();
    let mut types = Vec::new();
    for (kind, _) in &parts {
        varint(&mut types, *kind);
    }
    // ⛔ THE HEADER SIZE INCLUDES ITS OWN VARINT, so it is a fixed point rather
    // than a sum. Solving it by adding one and stopping is wrong at every
    // boundary where the varint grows, so it iterates until it agrees.
    let mut header = types.len() + 1;
    while varint_len(header as u64) + types.len() != header {
        header += 1;
    }
    let mut out = Vec::with_capacity(header + parts.len() * 8);
    varint(&mut out, header as u64);
    out.extend_from_slice(&types);
    for (_, bytes) in &parts {
        out.extend_from_slice(bytes);
    }
    out
}

/// Build one table b-tree leaf page holding `cells`, keyed in the order given.
///
/// `start` is the offset within the page at which the b-tree header begins,
/// which is 100 for page 1 and 0 for every other page.
///
/// # Errors
///
/// A string naming the row that does not fit, because a page that overflows is
/// the one case this writer refuses rather than handles.
fn leaf(cells: &[(i64, Vec<u8>)], start: usize) -> Result<Vec<u8>, String> {
    let mut page = vec![0_u8; PAGE_SIZE];
    let mut content = PAGE_SIZE;
    let mut pointers: Vec<u16> = Vec::with_capacity(cells.len());
    for (rowid, payload) in cells {
        if payload.len() > MAX_INLINE_PAYLOAD {
            return Err(format!(
                "row {rowid} needs {} bytes and a page holds {MAX_INLINE_PAYLOAD} inline: this \
                 writer does not implement overflow pages",
                payload.len()
            ));
        }
        let mut cell = Vec::with_capacity(payload.len() + 18);
        varint(&mut cell, payload.len() as u64);
        varint(&mut cell, *rowid as u64);
        cell.extend_from_slice(payload);
        content -= cell.len();
        page[content..content + cell.len()].copy_from_slice(&cell);
        pointers.push(content as u16);
    }
    let header_end = start + 8 + cells.len() * 2;
    if header_end > content {
        return Err(format!(
            "{} row(s) do not fit one {PAGE_SIZE}-byte page: this writer does not split b-trees",
            cells.len()
        ));
    }
    page[start] = 0x0d;
    page[start + 1..start + 3].copy_from_slice(&0_u16.to_be_bytes());
    page[start + 3..start + 5].copy_from_slice(&(cells.len() as u16).to_be_bytes());
    // ⚠ A content area starting exactly at 65536 is written as zero by the
    // format. This writer's pages are 8192 bytes, so the case cannot arise and
    // the cast is the plain one.
    page[start + 5..start + 7].copy_from_slice(&(content as u16).to_be_bytes());
    page[start + 7] = 0;
    for (i, offset) in pointers.iter().enumerate() {
        let at = start + 8 + i * 2;
        page[at..at + 2].copy_from_slice(&offset.to_be_bytes());
    }
    Ok(page)
}

/// Serialise a whole database holding `tables`.
///
/// ⛔ **The schema is page 1 and each table gets one page after it**, in the
/// order given, so a table's root page is its index in that list plus two.
///
/// # Errors
///
/// A string naming what did not fit. Nothing else can fail: there is no file
/// handle here and no allocation this can report on.
pub fn database(tables: &[Table]) -> Result<Vec<u8>, String> {
    let mut schema_cells: Vec<(i64, Vec<u8>)> = Vec::with_capacity(tables.len());
    for (i, table) in tables.iter().enumerate() {
        let root = i as i64 + 2;
        // ⚠ The sqlite_schema row is positional and its five columns are the
        // format's, not this project's: type, name, tbl_name, rootpage, sql.
        let row = record(&[
            Value::Text("table".to_owned()),
            Value::Text(table.name.clone()),
            Value::Text(table.name.clone()),
            Value::Int(root),
            Value::Text(table.sql()),
        ]);
        schema_cells.push((i as i64 + 1, row));
    }
    let mut out = leaf(&schema_cells, 100)?;
    for table in tables {
        let mut cells: Vec<(i64, Vec<u8>)> = Vec::with_capacity(table.rows.len());
        for (rowid, values) in &table.rows {
            if values.len() != table.columns.len() {
                return Err(format!(
                    "{}: row {rowid} carries {} value(s) for {} column(s)",
                    table.name,
                    values.len(),
                    table.columns.len()
                ));
            }
            // ⛔ THE ROW ID COLUMN STILL TAKES A SLOT IN THE RECORD, and it is
            // written NULL: SQLite takes the value from the cell's key and
            // ignores what is stored. A record that omits the slot is accepted
            // by every write and read back one column to the left, which is a
            // file that opens, passes an integrity check, and answers wrongly.
            let mut all = Vec::with_capacity(values.len() + 1);
            all.push(Value::Null);
            all.extend_from_slice(values);
            cells.push((*rowid, record(&all)));
        }
        out.extend_from_slice(&leaf(&cells, 0)?);
    }

    let pages = (tables.len() + 1) as u32;
    let header = &mut out[..100];
    header[..16].copy_from_slice(b"SQLite format 3\0");
    header[16..18].copy_from_slice(&(PAGE_SIZE as u16).to_be_bytes());
    header[18] = 1; // write version: rollback journal
    header[19] = 1; // read version: rollback journal
    header[20] = 0; // bytes reserved at the end of every page
    header[21] = 64; // maximum embedded payload fraction, fixed by the format
    header[22] = 32; // minimum embedded payload fraction, fixed by the format
    header[23] = 32; // leaf payload fraction, fixed by the format
    header[24..28].copy_from_slice(&1_u32.to_be_bytes()); // file change counter
    header[28..32].copy_from_slice(&pages.to_be_bytes());
    header[32..36].copy_from_slice(&0_u32.to_be_bytes()); // first freelist trunk page
    header[36..40].copy_from_slice(&0_u32.to_be_bytes()); // freelist page count
    header[40..44].copy_from_slice(&1_u32.to_be_bytes()); // schema cookie
    header[44..48].copy_from_slice(&4_u32.to_be_bytes()); // schema format number
    header[48..52].copy_from_slice(&0_u32.to_be_bytes()); // default page cache size
    header[52..56].copy_from_slice(&0_u32.to_be_bytes()); // largest root page, vacuum only
    header[56..60].copy_from_slice(&1_u32.to_be_bytes()); // text encoding: UTF-8
    header[60..64].copy_from_slice(&0_u32.to_be_bytes()); // user version
    header[64..68].copy_from_slice(&0_u32.to_be_bytes()); // incremental vacuum
    header[68..72].copy_from_slice(&0_u32.to_be_bytes()); // application id
    // ⛔ THE IN-HEADER PAGE COUNT IS ONLY BELIEVED WHEN THESE TWO AGREE. A
    // version-valid-for that does not match the change counter tells a reader
    // to measure the file instead, which is a correct file that every tool
    // reports differently.
    header[92..96].copy_from_slice(&1_u32.to_be_bytes());
    header[96..100].copy_from_slice(&SQLITE_VERSION_NUMBER.to_be_bytes());
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_varint_round_trips_at_every_width_boundary() {
        for value in [
            0_u64,
            1,
            127,
            128,
            16_383,
            16_384,
            0x00ff_ffff_ffff_ffff,
            0x0100_0000_0000_0000,
            u64::MAX,
        ] {
            let mut out = Vec::new();
            varint(&mut out, value);
            assert_eq!(varint_len(value), out.len(), "length disagrees for {value}");
            let mut got = 0_u64;
            if out.len() == 9 {
                for byte in &out[..8] {
                    got = (got << 7) | u64::from(byte & 0x7f);
                }
                got = (got << 8) | u64::from(out[8]);
            } else {
                for byte in &out {
                    got = (got << 7) | u64::from(byte & 0x7f);
                }
            }
            assert_eq!(got, value, "decode disagrees for {value}");
        }
    }

    #[test]
    fn a_record_header_counts_its_own_varint() {
        // 129 NULL columns make the type list 129 bytes, so the header size is
        // 131 and needs two varint bytes to say so. The naive answer, 130, is
        // the one this asserts against.
        let values = vec![Value::Null; 129];
        let encoded = record(&values);
        assert_eq!(encoded[0], 0x81, "the header varint is not two bytes");
        assert_eq!(encoded[1], 0x03, "the header size is not 131");
        assert_eq!(encoded.len(), 131);
    }

    #[test]
    fn a_row_too_large_for_a_page_is_refused() {
        let table = Table {
            name: "t".to_owned(),
            columns: vec!["a".to_owned()],
            rows: vec![(1, vec![Value::Blob(vec![0; PAGE_SIZE])])],
        };
        let err = database(&[table]).expect_err("a row larger than a page was accepted");
        assert!(err.contains("overflow pages"), "unexpected refusal: {err}");
    }

    #[test]
    fn a_database_carries_its_header_and_one_page_per_table() {
        let table = Table {
            name: "t".to_owned(),
            columns: vec!["a".to_owned()],
            rows: vec![(1, vec![Value::Text("x".to_owned())])],
        };
        let bytes = database(&[table]).expect("a one-row database was refused");
        assert_eq!(bytes.len(), PAGE_SIZE * 2);
        assert_eq!(&bytes[..16], b"SQLite format 3\0");
        assert_eq!(u16::from_be_bytes([bytes[16], bytes[17]]), PAGE_SIZE as u16);
        assert_eq!(
            u32::from_be_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]),
            2,
            "the in-header page count disagrees with the file"
        );
        assert_eq!(bytes[100], 0x0d, "page 1 is not a table leaf");
        assert_eq!(bytes[PAGE_SIZE], 0x0d, "page 2 is not a table leaf");
    }
}
