use crate::frame::*;

// ⭐ PATCHED HERE, for StreamDependency::encode. TODO/emitters.md, EMIT-03.
use bytes::BufMut;

#[derive(Debug, Eq, PartialEq)]
pub struct Priority {
    stream_id: StreamId,
    dependency: StreamDependency,
}

#[derive(Debug, Eq, PartialEq)]
pub struct StreamDependency {
    /// The ID of the stream dependency target
    dependency_id: StreamId,

    /// The weight for the stream. The value exposed (and set) here is always in
    /// the range [0, 255], instead of [1, 256] (as defined in section 5.3.2.)
    /// so that the value fits into a `u8`.
    weight: u8,

    /// True if the stream dependency is exclusive.
    is_exclusive: bool,
}

impl Priority {
    pub fn load(head: Head, payload: &[u8]) -> Result<Self, Error> {
        let dependency = StreamDependency::load(payload)?;

        if dependency.dependency_id() == head.stream_id() {
            return Err(Error::InvalidDependencyId);
        }

        Ok(Priority {
            stream_id: head.stream_id(),
            dependency,
        })
    }
}

impl<B> From<Priority> for Frame<B> {
    fn from(src: Priority) -> Self {
        Frame::Priority(src)
    }
}

// ===== impl StreamDependency =====

impl StreamDependency {
    pub fn new(dependency_id: StreamId, weight: u8, is_exclusive: bool) -> Self {
        StreamDependency {
            dependency_id,
            weight,
            is_exclusive,
        }
    }

    pub fn load(src: &[u8]) -> Result<Self, Error> {
        if src.len() != 5 {
            return Err(Error::InvalidPayloadLength);
        }

        // Parse the stream ID and exclusive flag
        let (dependency_id, is_exclusive) = StreamId::parse(&src[..4]);

        // Read the weight
        let weight = src[4];

        Ok(StreamDependency::new(dependency_id, weight, is_exclusive))
    }

    pub fn dependency_id(&self) -> StreamId {
        self.dependency_id
    }

    /// Write the five bytes of a stream dependency.
    ///
    /// ⭐ **PATCHED HERE. `load` had no counterpart**, which is the whole of
    /// why a client built on this library cannot send the PRIORITY block a
    /// browser sends. The layout is RFC 7540 section 6.2: a 31-bit stream
    /// identifier with the exclusive flag in the top bit, then one byte of
    /// weight. ⚠ The weight on the wire is the value here, in [0, 255], which
    /// is one less than the [1, 256] the specification defines; that offset is
    /// upstream's own convention and this does not re-apply it.
    ///
    /// `patches/README.md` and `TODO/emitters.md`, `EMIT-03`.
    pub fn encode<B: BufMut>(&self, dst: &mut B) {
        const EXCLUSIVE: u32 = 1 << 31;
        let mut id: u32 = self.dependency_id.into();
        if self.is_exclusive {
            id |= EXCLUSIVE;
        }
        dst.put_u32(id);
        dst.put_u8(self.weight);
    }
}
