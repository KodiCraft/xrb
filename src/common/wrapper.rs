// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{Atom, Colormap, Timestamp, Window};
use bytes::{Buf, BufMut};
use cornflakes::{DataSize, ReadResult, Readable, StaticDataSize, Wrapper, Writable, WriteResult};

macro_rules! impl_wrapper {
	(
		$($Type:ty: $wrapped:ty),*$(,)?
	) => {
		$(
			impl Wrapper for $Type {
				type WrappedType = $wrapped;

				fn wrap(val: $wrapped) -> Self {
					Self(val)
				}

				fn unwrap(&self) -> &$wrapped {
					&self.0
				}
			}
		)*
	}
}

impl_wrapper! {
	Window: u32,
	Colormap: u32,

	Atom: u32,
}

pub enum Inheritable<T> {
	CopyFromParent,
	Uninherited(T),
}

pub enum CurrentableTime {
	CurrentTime,
	Timestamp(Timestamp),
}

impl DataSize for CurrentableTime {
	fn data_size(&self) -> usize {
		Self::static_data_size()
	}
}

impl StaticDataSize for CurrentableTime {
	fn static_data_size() -> usize
	where
		Self: Sized,
	{
		4
	}
}

impl Writable for CurrentableTime {
	fn write_to(&self, buf: &mut impl BufMut) -> WriteResult {
		match self {
			Self::CurrentTime => buf.put_u32(0),
			Self::Timestamp(timestamp) => timestamp.write_to(buf)?,
		}

		Ok(())
	}
}

impl Readable for CurrentableTime {
	fn read_from(buf: &mut impl Buf) -> ReadResult<Self>
	where
		Self: Sized,
	{
		Ok(match buf.get_u32() {
			x if x == 0 => Self::CurrentTime,
			timestamp => Self::Timestamp(Timestamp::new(timestamp)),
		})
	}
}