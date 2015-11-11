#[cfg(target_arch = "x86")]
#[repr(packed)]
pub struct TSS {
    pub prev_tss: u32,
    pub sp0: usize,
    pub ss0: usize,
    pub sp1: usize,
    pub ss1: usize,
    pub sp2: usize,
    pub ss2: usize,
    pub cr3: usize,
    pub ip: usize,
    pub flags: usize,
    pub ax: usize,
    pub cx: usize,
    pub dx: usize,
    pub bx: usize,
    pub sp: usize,
    pub bp: usize,
    pub si: usize,
    pub di: usize,
    pub es: usize,
    pub cs: usize,
    pub ss: usize,
    pub ds: usize,
    pub fs: usize,
    pub gs: usize,
    pub ldt: usize,
    pub trap: u16,
    pub iomap_base: u16,
}

#[cfg(target_arch = "x86_64")]
#[repr(packed)]
pub struct TSS {
    pub reserved1: u32,
	pub sp0: usize,
	pub sp1: usize,
	pub sp2: usize,
	pub reserved2: u32,
	pub reserved3: u32,
	pub ist1: usize,
	pub ist2: usize,
	pub ist3: usize,
	pub ist4: usize,
	pub ist5: usize,
	pub ist6: usize,
	pub ist7: usize,
	pub reserved4: u32,
	pub reserved5: u32,
	pub reserved6: u16,
    pub iomap_base: u16,
}
