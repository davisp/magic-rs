use std::fmt;

use thiserror::Error;

use crate::traits::ReadLittleEndian;
use crate::value::{Value, ValueError, ValueOption, ValueType};

#[derive(Debug, Error)]
pub enum MagicError {
    #[error("Invalid magic record size: {0} expected {1}")]
    InvalidBufferLength(usize, usize),
    #[error("Invalid conditional type: {0} expected <= 3")]
    InvalidConditionalType(u8),
    #[error("Invalid factor operation: {0} expect +, -, *, /, or \\0")]
    InvalidFactorOperation(char),
    #[error("Invalid indirection operation has bit 4 or 5 set.")]
    InvalidIndirectionOperationBitSet,
    #[error("Invalid relation: {0} expected one of =, !, <, >, ^, &, x")]
    InvalidRelation(char),
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    #[error("Error parsing magic record value: {0}")]
    Value(#[from] ValueError),
}

type Result<T> = std::result::Result<T, MagicError>;

// Total size: 434
#[derive(Debug)]
pub struct Magic {
    // 4 bytes
    pub cont_level: u16,
    pub flags: MagicFlags,
    pub factor: u8,

    // 4 bytes
    pub relation: Relation,
    pub value_len: u8,
    pub value_type: ValueType,
    pub indirection_type: ValueType,

    // 4 bytes
    pub indirection_operation: IndirectionOperation,
    pub mask_operation: IndirectionOperation,
    pub conditional_type: ConditionalType,
    pub factor_operation: FactorOperation,

    // 12 bytes
    pub offset: i32,
    pub indirection_offset: i32,
    pub line_number: u32,

    // 136 bytes = 8 bytes option + 128 string max len
    pub value_options: ValueOption,
    pub value: Value,

    // 272 bytes = 64 desc + 80 mimetype + 8 apple + 120 ext
    pub desc: String,
    pub mimetype: String,
    pub apple: String,
    pub ext: String,
}

impl Magic {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != crate::loader::MAGIC_SIZE {
            return Err(MagicError::InvalidBufferLength(
                bytes.len(),
                crate::loader::MAGIC_SIZE,
            ));
        }

        let cont_level = u16::read_le(&bytes[0..2]);
        let flags = MagicFlags::from(bytes[2]);
        let factor = bytes[3];

        let relation = Relation::try_from(bytes[4])?;
        let value_len = bytes[5];
        let value_type = ValueType::try_from(bytes[6])?;
        let indirection_type = ValueType::try_from(bytes[7])?;

        let indirection_operation = IndirectionOperation::try_from(bytes[8])?;
        let mask_operation = IndirectionOperation::try_from(bytes[9])?;
        let conditional_type = ConditionalType::try_from(bytes[10])?;
        let factor_operation = FactorOperation::try_from(bytes[11])?;

        let offset = i32::read_le(&bytes[12..16]);
        let indirection_offset = i32::read_le(&bytes[16..20]);
        let line_number = u32::read_le(&bytes[20..24]);

        let value_options = if value_len > 0 {
            let count = u32::read_le(&bytes[24..28]);
            let flags = u32::read_le(&bytes[28..32]);
            ValueOption::String { count, flags }
        } else {
            let mask = u64::read_le(&bytes[24..32]);
            ValueOption::Numeric { mask }
        };

        let value = Value::new(value_type, value_len, &bytes[32..160])?;
        let desc = bytes_to_string(&bytes[160..224])?;
        let mimetype = bytes_to_string(&bytes[224..304])?;
        let apple = bytes_to_string(&bytes[304..312])?;
        let ext = bytes_to_string(&bytes[312..432])?;

        Ok(Magic {
            cont_level,
            flags,
            factor,
            relation,
            value_len,
            value_type,
            indirection_type,
            indirection_operation,
            mask_operation,
            conditional_type,
            factor_operation,
            offset,
            indirection_offset,
            line_number,
            value_options,
            value,
            desc,
            mimetype,
            apple,
            ext,
        })
    }
}

#[derive(Default)]
pub struct MagicFlags {
    flags: u8,
}

impl MagicFlags {
    const INDIRECT: u8 = 0x01;
    const OFFSET_ADD: u8 = 0x02;
    const INDIRECT_OFFSET_ADD: u8 = 0x04;
    const UNSIGNED: u8 = 0x08;
    const NO_SPACE: u8 = 0x10;
    const BIN_TEST: u8 = 0x20;
    const TEXT_TEST: u8 = 0x40;
    const OFFSET_NEGATIVE: u8 = 0x80;

    pub fn is_indirect(&self) -> bool {
        self.is_set(Self::INDIRECT)
    }

    pub fn is_offset_add(&self) -> bool {
        self.is_set(Self::OFFSET_ADD)
    }

    pub fn is_indirect_offset_add(&self) -> bool {
        self.is_set(Self::INDIRECT_OFFSET_ADD)
    }

    pub fn is_unsigned(&self) -> bool {
        self.is_set(Self::UNSIGNED)
    }

    pub fn is_no_space(&self) -> bool {
        self.is_set(Self::NO_SPACE)
    }

    pub fn is_bin_test(&self) -> bool {
        self.is_set(Self::BIN_TEST)
    }

    pub fn is_text_test(&self) -> bool {
        self.is_set(Self::TEXT_TEST)
    }

    pub fn is_offset_negative(&self) -> bool {
        self.is_set(Self::OFFSET_NEGATIVE)
    }

    pub fn is_set(&self, flag: u8) -> bool {
        self.flags & flag == flag
    }
}

impl From<u8> for MagicFlags {
    fn from(value: u8) -> Self {
        Self { flags: value }
    }
}

impl fmt::Debug for MagicFlags {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut str_flags = Vec::with_capacity(8);

        if self.is_indirect() {
            str_flags.push("INDIRECT");
        }

        if self.is_offset_add() {
            str_flags.push("OFFSET_ADD");
        }

        if self.is_indirect_offset_add() {
            str_flags.push("INDIRECT_OFFSET_ADD");
        }

        if self.is_unsigned() {
            str_flags.push("UNSIGNED");
        }

        if self.is_no_space() {
            str_flags.push("NO_SPACE");
        }

        if self.is_bin_test() {
            str_flags.push("BIN_TEST");
        }

        if self.is_text_test() {
            str_flags.push("TEXT_TEST");
        }

        if self.is_offset_negative() {
            str_flags.push("OFFSET_NEGATIVE");
        }

        if str_flags.is_empty() {
            str_flags.push("N/A");
        }

        fmt.write_fmt(format_args!("{}", str_flags.join(" | ")))
    }
}

#[derive(Debug, Default)]
pub enum Relation {
    #[default]
    Equal,
    NotEqual,
    Lesser,
    Greater,
    BitXor,
    BitAnd,
    Anything,
}

impl TryFrom<u8> for Relation {
    type Error = MagicError;

    fn try_from(value: u8) -> Result<Self> {
        use Relation::*;
        let rel = match value {
            b'=' => Equal,
            b'!' => NotEqual,
            b'<' => Lesser,
            b'>' => Greater,
            b'^' => BitXor,
            b'&' => BitAnd,
            b'x' => Anything,
            other => return Err(MagicError::InvalidRelation(other as char)),
        };
        Ok(rel)
    }
}

#[derive(Debug, Default)]
pub enum FactorOperation {
    #[default]
    None,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

impl TryFrom<u8> for FactorOperation {
    type Error = MagicError;

    fn try_from(value: u8) -> Result<Self> {
        use FactorOperation::*;
        let op = match value {
            b'\0' => None,
            b'+' => Add,
            b'-' => Subtract,
            b'*' => Multiply,
            b'/' => Divide,
            other => {
                return Err(MagicError::InvalidFactorOperation(other as char))
            }
        };
        Ok(op)
    }
}

#[derive(Debug)]
pub enum IndirectionOperator {
    And,
    Or,
    Xor,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug)]
pub struct IndirectionFlags {
    pub signed: bool,
    pub inverse: bool,
    pub indirect: bool,
}

#[derive(Debug)]
pub struct IndirectionOperation {
    pub op: IndirectionOperator,
    pub flags: IndirectionFlags,
}

impl Default for IndirectionOperation {
    fn default() -> Self {
        IndirectionOperation {
            op: IndirectionOperator::And,
            flags: IndirectionFlags {
                signed: false,
                inverse: false,
                indirect: false,
            },
        }
    }
}

impl TryFrom<u8> for IndirectionOperation {
    type Error = MagicError;

    fn try_from(value: u8) -> Result<Self> {
        // libmagic declares these two bits as unused and marks them
        // as FILE_UNUSED_1 and FILE_UNUSED_2. So we assert they're not
        // set for clarity.
        if value & 0x18 != 0 {
            return Err(MagicError::InvalidIndirectionOperationBitSet);
        }

        // Bottom 3 bits are the operator
        let op = match value & 0x07 {
            0 => IndirectionOperator::And,
            1 => IndirectionOperator::Or,
            2 => IndirectionOperator::Xor,
            3 => IndirectionOperator::Add,
            4 => IndirectionOperator::Subtract,
            5 => IndirectionOperator::Multiply,
            6 => IndirectionOperator::Divide,
            7 => IndirectionOperator::Modulo,
            _ => panic!("value & 0x07 failed?"),
        };

        // Top 3 bits are flags that can be set
        let signed = value & 0x20 == 0x20;
        let inverse = value & 0x40 == 0x40;
        let indirect = value & 0x80 == 0x80;

        Ok(IndirectionOperation {
            op,
            flags: IndirectionFlags {
                signed,
                inverse,
                indirect,
            },
        })
    }
}

#[derive(Debug, Default)]
pub enum ConditionalType {
    #[default]
    None,
    If,
    Elif,
    Else,
}

impl TryFrom<u8> for ConditionalType {
    type Error = MagicError;

    fn try_from(value: u8) -> Result<Self> {
        use ConditionalType::*;
        let op = match value {
            0 => None,
            1 => If,
            2 => Elif,
            3 => Else,
            other => return Err(MagicError::InvalidConditionalType(other)),
        };
        Ok(op)
    }
}

fn bytes_to_string(bytes: &[u8]) -> Result<String> {
    let first_null = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    Ok(std::str::from_utf8(&bytes[0..first_null])?.to_string())
}
