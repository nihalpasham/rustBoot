use core::mem::size_of;
use core::slice::from_raw_parts_mut;
use core::str::from_utf8;

use super::common::*;
use super::internal::*;

/// DTB structure item.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum StructItem<'a> {
    BeginNode {
        name: &'a str,
    },
    Property {
        name: &'a str,
        value: &'a [u8],
    },
    EndNode,
    /// This variant represents a non-existent item. Used to
    /// initialize a [`StructItem`] with a default value.
    None,
}

impl<'a> StructItem<'a> {
    /// Returns true if the structure item is BeginNode.
    pub fn is_begin_node(&self) -> bool {
        match self {
            StructItem::BeginNode { .. } => true,
            _ => false,
        }
    }

    /// Returns true if the structure item is Property.
    pub fn is_property(&self) -> bool {
        match self {
            StructItem::Property { .. } => true,
            _ => false,
        }
    }
    /// Returns true if the structure item is Property.
    pub fn is_end_node(&self) -> bool {
        match self {
            StructItem::EndNode => true,
            _ => false,
        }
    }

    /// Returns name for BeginNode or Property structure items.
    pub fn name(&self) -> Result<&'a str> {
        match self {
            StructItem::BeginNode { name } => Ok(name),
            StructItem::Property { name, .. } => Ok(name),
            _ => Err(Error::BadStructItemType),
        }
    }

    /// Returns node name for BeginNode structure items.
    pub fn node_name(&self) -> Result<&'a str> {
        match self {
            StructItem::BeginNode { name } => Ok(name.split('@').next().unwrap()),
            _ => Err(Error::BadStructItemType),
        }
    }

    /// Returns unit address for BeginNode structure items.
    pub fn unit_address(&self) -> Result<&'a str> {
        match self {
            StructItem::BeginNode { name } => {
                let mut iter = name.split('@');
                iter.next();
                Ok(match iter.next() {
                    Option::Some(addr) => addr,
                    Option::None => "",
                })
            }
            _ => Err(Error::BadStructItemType),
        }
    }

    /// Returns value for Property structure items.
    pub fn value(&self) -> Result<&'a [u8]> {
        match self {
            StructItem::Property { value, .. } => Ok(value),
            _ => Err(Error::BadStructItemType),
        }
    }

    /// Returns string value for Property structure items.
    pub fn value_str(&self) -> Result<&'a str> {
        let value = self.value()?;
        let len = value.len();
        if len == 0 || value[len - 1] != 0 {
            return Err(Error::BadValueStr);
        }
        match from_utf8(&value[..len - 1]) {
            Ok(value_str) => Ok(value_str),
            Err(err) => Err(Error::BadStrEncoding(err)),
        }
    }

    pub unsafe fn transmute_buf<T>(buf: &mut [u8]) -> Result<&mut [T]> {
        let buf = align_buf::<T>(buf)?;
        Ok(from_raw_parts_mut(
            buf.as_ptr() as *mut T,
            buf.len() / size_of::<T>(),
        ))
    }

    /// Returns string list value for Property structure items.
    pub fn value_str_list<'b>(&self, buf: &'b mut [u8]) -> Result<&'b [&'a str]> {
        let mut i = 0;
        let buf = unsafe { StructItem::transmute_buf(buf)? };
        for part in self.value_str()?.split('\0') {
            if i >= buf.len() {
                return Err(Error::BufferTooSmall);
            }
            buf[i] = part;
            i += 1;
        }
        Ok(&buf[..i])
    }

    /// Returns integer list value for Property structure items.
    #[allow(clippy::cast_ptr_alignment)]
    pub fn value_u32_list<'b>(&self, buf: &'b mut [u8]) -> Result<&'b [u32]> {
        let value = self.value()?;

        if value.len() % 4 != 0 {
            return Err(Error::BadU32List);
        }

        let len = value.len() / 4;
        let buf = unsafe { StructItem::transmute_buf(buf)? };
        if buf.len() < len {
            return Err(Error::BufferTooSmall);
        }

        for (i, val) in buf.iter_mut().enumerate().take(len) {
            *val = u32::from_be(unsafe { *(value.as_ptr().add(4 * i as usize) as *const u32) });
        }

        Ok(&buf[..len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_begin_node() {
        assert_eq!(StructItem::BeginNode { name: "" }.is_begin_node(), true);
        assert_eq!(
            StructItem::Property {
                name: "",
                value: &[],
            }
            .is_begin_node(),
            false
        );
        assert_eq!(StructItem::EndNode.is_begin_node(), false);
    }

    #[test]
    fn test_is_property() {
        assert_eq!(StructItem::BeginNode { name: "" }.is_property(), false);
        assert_eq!(
            StructItem::Property {
                name: "",
                value: &[],
            }
            .is_property(),
            true
        );
        assert_eq!(StructItem::EndNode.is_property(), false);
    }

    #[test]
    fn test_name() {
        assert_eq!(
            StructItem::BeginNode { name: "node" }.name().unwrap(),
            "node"
        );
        assert_eq!(
            StructItem::Property {
                name: "property",
                value: &[],
            }
            .name()
            .unwrap(),
            "property"
        );
        assert_eq!(
            StructItem::EndNode.name().unwrap_err(),
            Error::BadStructItemType
        );
    }

    macro_rules! assert_value {
        ($method:ident $(, $buf:ident)*) => {
            assert_eq!(
                StructItem::BeginNode {name: "node" }
                .$method($( &mut $buf )*).unwrap_err(),
                Error::BadStructItemType
            );
            assert_eq!(
                StructItem::EndNode.$method($( &mut $buf )*).unwrap_err(),
                Error::BadStructItemType
            );
        };
    }

    #[test]
    fn test_value() {
        assert_value!(value);
        assert_eq!(
            StructItem::Property {
                name: "property",
                value: &[1, 2, 3],
            }
            .value()
            .unwrap(),
            &[1, 2, 3]
        );
    }

    macro_rules! assert_value_str {
        ($method:ident $(, $buf:ident)*) => {
            assert_value!($method $(, $buf )*);
            assert_eq!(
                StructItem::Property {
                    name: "property",
                    value: "".as_bytes(),
                }
                .$method($( &mut $buf )*)
                .unwrap_err(),
                Error::BadValueStr
            );
            assert_eq!(
                StructItem::Property {
                    name: "property",
                    value: "value".as_bytes(),
                }
                .$method($( &mut $buf )*)
                .unwrap_err(),
                Error::BadValueStr
            );
        };
    }

    #[test]
    fn test_value_str() {
        assert_value_str!(value_str);
        assert_eq!(
            StructItem::Property {
                name: "property",
                value: "value\0".as_bytes(),
            }
            .value_str()
            .unwrap(),
            "value"
        );
    }

    #[test]
    fn test_value_str_list() {
        aligned_buf!(buf, [""; 2]);
        assert_value_str!(value_str_list, buf);

        let prop = StructItem::Property {
            name: "property",
            value: "part1\0part2\0".as_bytes(),
        };

        aligned_buf!(tmp, [""; 3]);
        let len = tmp.len();
        let mut unaligned_buf = &mut tmp[size_of::<usize>() - 1..len - size_of::<usize>() - 1];
        assert_eq!(
            prop.value_str_list(&mut unaligned_buf).unwrap_err(),
            Error::BufferTooSmall
        );

        aligned_buf!(small_buf, [""; 1]);
        assert_eq!(
            prop.value_str_list(&mut small_buf).unwrap_err(),
            Error::BufferTooSmall
        );

        assert_eq!(prop.value_str_list(&mut buf).unwrap(), &["part1", "part2"]);
    }

    #[test]
    fn test_value_u32_list() {
        aligned_buf!(buf, [0u32; 3]);
        assert_value!(value_u32_list, buf);

        assert_eq!(
            StructItem::Property {
                name: "property",
                value: &[1, 2, 3],
            }
            .value_u32_list(&mut buf)
            .unwrap_err(),
            Error::BadU32List
        );

        assert_eq!(
            StructItem::Property {
                name: "property",
                value: &[],
            }
            .value_u32_list(&mut buf)
            .unwrap(),
            &[]
        );

        let prop = StructItem::Property {
            name: "property",
            value: &[0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3],
        };

        aligned_buf!(tmp, [0u32; 4]);
        let len = tmp.len();
        let mut unaligned_buf = &mut tmp[1..len - 1];
        assert_eq!(
            prop.value_u32_list(&mut unaligned_buf).unwrap_err(),
            Error::BufferTooSmall
        );

        aligned_buf!(small_buf, [0u32; 2]);
        assert_eq!(
            prop.value_u32_list(&mut small_buf).unwrap_err(),
            Error::BufferTooSmall
        );

        assert_eq!(prop.value_u32_list(&mut buf).unwrap(), &[1, 2, 3]);
    }

    fn assert_begin_node_accessor<'a>(
        accessor: fn(item: StructItem<'a>) -> Result<&'a str>,
        name: &'static str,
        expected: &'static str,
    ) {
        assert_eq!(
            accessor(StructItem::BeginNode { name: name }).unwrap(),
            expected
        );
        assert_eq!(
            accessor(StructItem::Property {
                name: name,
                value: &[],
            })
            .unwrap_err(),
            Error::BadStructItemType
        );
        assert_eq!(
            accessor(StructItem::EndNode).unwrap_err(),
            Error::BadStructItemType
        );
    }

    #[test]
    fn test_node_name() {
        assert_begin_node_accessor(
            |item| item.node_name(),
            "node_name@unit_address",
            "node_name",
        );
        assert_begin_node_accessor(|item| item.node_name(), "node_name", "node_name");
    }

    #[test]
    fn test_unit_address() {
        assert_begin_node_accessor(
            |item| item.unit_address(),
            "node_name@unit_address",
            "unit_address",
        );
        assert_begin_node_accessor(|item| item.unit_address(), "node_name", "");
    }
}
