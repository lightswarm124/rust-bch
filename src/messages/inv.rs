use messages::inv_vect::InvVect;
use messages::message::Payload;
use std::fmt;
use std::io;
use std::io::{Read, Write};
use crate::util::{var_int, Error, Result, Serializable};

/// Maximum number of objects in an inv message
pub const MAX_INV_ENTRIES: usize = 50000;

/// Inventory payload describing objects a node knows about
#[derive(Default, PartialEq, Eq, Hash, Clone)]
pub struct Inv {
    /// List of objects announced
    pub objects: Vec<InvVect>,
}

impl Serializable<Inv> for Inv {
    fn read(reader: &mut dyn Read) -> Result<Inv> {
        let num_objects = var_int::read(reader)? as usize;
        if num_objects > MAX_INV_ENTRIES {
            let msg = format!("Num objects exceeded maximum: {}", num_objects);
            return Err(Error::BadData(msg));
        }
        let mut objects = Vec::with_capacity(num_objects);
        for _ in 0..num_objects {
            objects.push(InvVect::read(reader)?);
        }
        Ok(Inv { objects })
    }

    fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
        var_int::write(self.objects.len() as u64, writer)?;
        for object in self.objects.iter() {
            object.write(writer)?;
        }
        Ok(())
    }
}

impl Payload<Inv> for Inv {
    fn size(&self) -> usize {
        var_int::size(self.objects.len() as u64) + InvVect::SIZE * self.objects.len()
    }
}

impl fmt::Debug for Inv {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.objects.len() <= 3 {
            f.debug_struct("Inv")
                .field("objects", &self.objects)
                .finish()
        } else {
            let s = format!("[<{} inventory vectors>]", self.objects.len());
            f.debug_struct("Inv").field("objects", &s).finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use messages::inv_vect::InvVectType;
    use std::io::Cursor;
    use crate::util::Hash256;

    #[test]
    fn write_read() {
        let iv1 = InvVect {
            obj_type: InvVectType::Tx,
            hash: Hash256([8; 32]),
        };
        let iv2 = InvVect {
            obj_type: InvVectType::Block,
            hash: Hash256([9; 32]),
        };
        let mut inv = Inv {
            objects: Vec::new(),
        };
        inv.objects.push(iv1);
        inv.objects.push(iv2);
        let mut v = Vec::new();
        inv.write(&mut v).unwrap();
        assert!(v.len() == inv.size());
        assert!(Inv::read(&mut Cursor::new(&v)).unwrap() == inv);
    }

    #[test]
    fn too_many_objects() {
        let mut inv = Inv {
            objects: Vec::new(),
        };
        for _i in 0..MAX_INV_ENTRIES + 1 {
            let inv_vect = InvVect {
                obj_type: InvVectType::Tx,
                hash: Hash256([8; 32]),
            };
            inv.objects.push(inv_vect);
        }
        let mut v = Vec::new();
        inv.write(&mut v).unwrap();
        assert!(Inv::read(&mut Cursor::new(&v)).is_err());
    }
}
