use std::fmt;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValueError {
    #[error("Invalid value type: {0}")]
    InvalidValueType(u8),
}

type Result<T> = std::result::Result<T, ValueError>;

#[derive(Clone, Copy, Debug, Default)]
#[repr(u8)]
pub enum ValueType {
    #[default]
    Invalid = 0,
    Byte = 1,
    Short = 2,
    Default = 3,
    Long = 4,
    String = 5,
    Date = 6,
    BeShort = 7,
    BeLong = 8,
    BeDate = 9,
    LeShort = 10,
    LeLong = 11,
    LeDate = 12,
    PString = 13,
    LDate = 14,
    BeLDate = 15,
    LeLDate = 16,
    Regex = 17,
    BeString16 = 18,
    LeString16 = 19,
    Search = 20,
    MeDate = 21,
    MeLDate = 22,
    MeLong = 23,
    Quad = 24,
    LeQuad = 25,
    BeQuad = 26,
    QDate = 27,
    LeQDate = 28,
    BeQDate = 29,
    QLDate = 30,
    LeQLDate = 31,
    BeQLDate = 32,
    Float = 33,
    BeFloat = 34,
    LeFloat = 35,
    Double = 36,
    BeDouble = 37,
    LeDouble = 38,
    BeId3 = 39,
    LeId3 = 40,
    Indirect = 41,
    QwDate = 42,
    LeQwDate = 43,
    BeQwDate = 44,
    Name = 45,
    Use = 46,
    Clear = 47,
    Der = 48,
    Guid = 49,
    Offset = 50,
    BeVarInt = 51,
    LeVarInt = 52,
    MSDosDate = 53,
    LeMSDosDate = 54,
    BeMsDosDate = 55,
    MSDosTime = 56,
    LeMSDOSTime = 57,
    BeMSDOSTime = 58,
    Octal = 59,
}

impl ValueType {
    fn is_string(&self) -> bool {
        use ValueType::*;
        matches!(
            self,
            // From file.h definitions
            String
                | PString
                | BeString16
                | LeString16
                | Regex
                | Search
                | Indirect
                | Name
                | Use
                | Octal
        )
    }
}

impl TryFrom<u8> for ValueType {
    type Error = ValueError;

    fn try_from(value: u8) -> Result<Self> {
        use ValueType::*;
        let vtype = match value {
            0 => Invalid,
            1 => Byte,
            2 => Short,
            3 => Default,
            4 => Long,
            5 => String,
            6 => Date,
            7 => BeShort,
            8 => BeLong,
            9 => BeDate,
            10 => LeShort,
            11 => LeLong,
            12 => LeDate,
            13 => PString,
            14 => LDate,
            15 => BeLDate,
            16 => LeLDate,
            17 => Regex,
            18 => BeString16,
            19 => LeString16,
            20 => Search,
            21 => MeDate,
            22 => MeLDate,
            23 => MeLong,
            24 => Quad,
            25 => LeQuad,
            26 => BeQuad,
            27 => QDate,
            28 => LeQDate,
            29 => BeQDate,
            30 => QLDate,
            31 => LeQLDate,
            32 => BeQLDate,
            33 => Float,
            34 => BeFloat,
            35 => LeFloat,
            36 => Double,
            37 => BeDouble,
            38 => LeDouble,
            39 => BeId3,
            40 => LeId3,
            41 => Indirect,
            42 => QwDate,
            43 => LeQwDate,
            44 => BeQwDate,
            45 => Name,
            46 => Use,
            47 => Clear,
            48 => Der,
            49 => Guid,
            50 => Offset,
            51 => BeVarInt,
            52 => LeVarInt,
            53 => MSDosDate,
            54 => LeMSDosDate,
            55 => BeMsDosDate,
            56 => MSDosTime,
            57 => LeMSDOSTime,
            58 => BeMSDOSTime,
            59 => Octal,
            other => {
                return Err(ValueError::InvalidValueType(other));
            }
        };
        Ok(vtype)
    }
}

#[derive(Debug)]
pub enum ValueOption {
    Numeric { mask: u64 },
    String { count: u32, flags: u32 },
}

impl Default for ValueOption {
    fn default() -> Self {
        ValueOption::Numeric { mask: 0 }
    }
}

pub struct Value {
    vtype: ValueType,
    bytes: Box<[u8]>,
}

impl Value {
    pub fn new(vtype: ValueType, len: u8, bytes: &[u8]) -> Result<Self> {
        assert!((len as usize) <= bytes.len());

        // Trust length if it's non-zero
        let len = if len != 0 {
            len as usize
        } else {
            let mut len = bytes.len() - 1;
            // > 1 here because we always want at least a single byte even if
            // its a zero byte.
            while len > 1 {
                if bytes[len - 1] != 0 {
                    break;
                }
                len -= 1;
            }
            len
        };
        assert!(len > 0);
        let bytes = Vec::from(&bytes[0..len]).into_boxed_slice();
        Ok(Value { vtype, bytes })
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.vtype.is_string() {
            // Attempt to show bytes as a printed string, allowing for the
            // possibility of non-utf8 data.
            let res = std::str::from_utf8(&self.bytes);
            if let Ok(sval) = res {
                f.write_fmt(format_args!("{:?}: '{}'", self.vtype, sval))?;
            } else {
                f.write_fmt(format_args!(
                    "{:?}: {:?}",
                    self.vtype, self.bytes
                ))?;
            }
        } else {
            match self.vtype {
                ValueType::Byte => f.write_fmt(format_args!(
                    "{:?}: {:?}",
                    self.vtype, self.bytes[0]
                ))?,
                _ => f.write_fmt(format_args!(
                    "{:?}: {:?}",
                    self.vtype, self.bytes
                ))?,
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_value_type_conversions() {
        for i in 0u8..255 {
            let vt = ValueType::try_from(i);

            if vt.is_err() {
                assert!(i >= 60);
                continue;
            }

            assert!(vt.ok().unwrap() as u8 == i);
        }
    }
}
