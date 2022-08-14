// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bitmask;

bitmask! {
	/// A bitmask of X core protocol events.
	///
	/// Bitmask value `0xfe000000` must be zero.
	pub enum Event -> u32 {
		KeyPress => 0x00000001,
		KeyRelease => 0x00000002,
		ButtonPress => 0x00000004,
		ButtonRelease => 0x00000008,
		EnterWindow => 0x00000010,
		LeaveWindow => 0x00000020,
		PointerMotion => 0x00000040,
		PointerMotionHint => 0x00000080,
		Button1Motion => 0x00000100,
		Button2Motion => 0x00000200,
		Button3Motion => 0x00000400,
		Button4Motion => 0x00000800,
		Button5Motion => 0x00001000,
		ButtonMotion => 0x00002000,
		KeymapState => 0x00004000,
		Exposure => 0x00008000,
		VisibilityChange => 0x00010000,
		StructureNotify => 0x00020000,
		SubstructureNotify => 0x00040000,
		SubstructureRedirect => 0x00080000,
		FocusChange => 0x00100000,
		PropertyChange => 0x00400000,
		ColormapChange => 0x00800000,
		OwnerGrabButton => 0x01000000,
		// unused but must be zero => 0xfe000000
	}

	/// A bitmask of X core protocol events, specifically used in pointer events.
	///
	/// Bitmask value `0xffff8003` must be zero.
	pub enum PointerEvent -> u32 {
		KeyPress => 0x00000001,
		KeyRelease => 0x00000002,
		ButtonPress => 0x00000004,
		ButtonRelease => 0x00000008,
		EnterWindow => 0x00000010,
		LeaveWindow => 0x00000020,
		PointerMotion => 0x00000040,
		PointerMotionHint => 0x00000080,
		Button1Motion => 0x00000100,
		Button2Motion => 0x00000200,
		Button3Motion => 0x00000400,
		Button4Motion => 0x00000800,
		Button5Motion => 0x00001000,
		ButtonMotion => 0x00002000,
		KeymapState => 0x00004000,
		Exposure => 0x00008000,
		VisibilityChange => 0x00010000,
		StructureNotify => 0x00020000,
		SubstructureNotify => 0x00040000,
		SubstructureRedirect => 0x00080000,
		FocusChange => 0x00100000,
		PropertyChange => 0x00400000,
		ColormapChange => 0x00800000,
		OwnerGrabButton => 0x01000000,
		// unused but must be zero => 0xffff8003
	}

	/// A bitmask of X core protocol events, specifically used in device events.
	///
	/// Bitmask value `0xffffc0b0` must be zero.
	pub enum DeviceEvent -> u32 {
		KeyPress => 0x00000001,
		KeyRelease => 0x00000002,
		ButtonPress => 0x00000004,
		ButtonRelease => 0x00000008,
		EnterWindow => 0x00000010,
		LeaveWindow => 0x00000020,
		PointerMotion => 0x00000040,
		PointerMotionHint => 0x00000080,
		Button1Motion => 0x00000100,
		Button2Motion => 0x00000200,
		Button3Motion => 0x00000400,
		Button4Motion => 0x00000800,
		Button5Motion => 0x00001000,
		ButtonMotion => 0x00002000,
		KeymapState => 0x00004000,
		Exposure => 0x00008000,
		VisibilityChange => 0x00010000,
		StructureNotify => 0x00020000,
		SubstructureNotify => 0x00040000,
		SubstructureRedirect => 0x00080000,
		FocusChange => 0x00100000,
		PropertyChange => 0x00400000,
		ColormapChange => 0x00800000,
		OwnerGrabButton => 0x01000000,
		// unused but must be zero => 0xffc0b0
	}

	/// A bitmask of modifier keys and mouse buttons.
	///
	/// Bitmask value `0xe000` must be zero.
	pub enum KeyButtonMask -> u16 {
		Shift => 0x0001,
		Lock => 0x0002,
		Control => 0x0004,
		Mod1 => 0x0008,
		Mod2 => 0x0010,
		Mod3 => 0x0020,
		Mod4 => 0x0040,
		Mod5 => 0x0080,
		Button1 => 0x0100,
		Button2 => 0x0200,
		Button3 => 0x0400,
		Button4 => 0x0800,
		Button5 => 0x1000,
		// unused but must be zero => 0xe000
	}

	/// A bitmask of modifier keys and mouse buttons, specifically used in key events.
	///
	/// Bitmask value `0xff00` must be zero.
	pub enum KeyMask -> u16 {
		Shift => 0x0001,
		Lock => 0x0002,
		Control => 0x0004,
		Mod1 => 0x0008,
		Mod2 => 0x0010,
		Mod3 => 0x0020,
		Mod4 => 0x0040,
		Mod5 => 0x0080,
		Button1 => 0x0100,
		Button2 => 0x0200,
		Button3 => 0x0400,
		Button4 => 0x0800,
		Button5 => 0x1000,
		// unused but must be zero => 0xff00
	}
}