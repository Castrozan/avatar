use std::io;

use crate::conversation_addressee::Addressee;

pub trait AddresseeStore {
    fn current(&self) -> io::Result<Option<Addressee>>;
    fn hand_to(&self, addressee: &Addressee) -> io::Result<()>;
    fn forget(&self) -> io::Result<()>;
}
