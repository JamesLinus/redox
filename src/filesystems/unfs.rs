use core::clone::Clone;
use core::mem::size_of;
use core::ptr;

use common::memory::*;
use common::string::*;
use common::vec::*;

use drivers::disk::*;

#[derive(Copy, Clone)]
pub struct Block {
    pub address: u64
}

#[derive(Copy, Clone)]
pub struct Extent{
    pub block: Block,
    pub length: u64
}

pub struct Header {
    pub signature: [u8; 4],
    pub version: u32,
    pub root_sector_list: Block,
    pub free_sector_list: Block,
    pub name: [u8; 256],
    pub reserved: [u8; 232]
}

pub struct Node {
    pub parent_collection: Block,
    pub data_sector_list: Block,
    pub data_size: u64,
    pub user_id: u64,
    pub group_id: u64,
    pub mode: u64,
    pub modify_time: u64,
    pub access_time: u64,
    pub create_time: u64,
    pub name: [u8; 256],
    pub reserved: [u8; 184]
}

pub struct SectorList {
    pub parent_node: Block,
    pub fragment_number: u64,
    pub next_fragment: Block,
    pub last_fragment: Block,
    pub extents: [Extent; 30]
}

pub struct UnFS {
    pub disk: Disk,
    pub header: Header
}

impl UnFS {
    pub fn from_disk(disk: Disk) -> UnFS{
        unsafe{
            let header_ptr = alloc(size_of::<Header>()) as *const Header;
            disk.read(1, 1, header_ptr as usize);
            let ret = UnFS { disk:disk, header: ptr::read(header_ptr) };
            unalloc(header_ptr as usize);
            return ret;
        }
    }

    pub fn valid(&self) -> bool {
        return self.header.signature[0] == 'U' as u8
            && self.header.signature[1] == 'n' as u8
            && self.header.signature[2] == 'F' as u8
            && self.header.signature[3] == 'S' as u8;
    }

    pub unsafe fn node(&self, filename: String) -> *const Node{
        let mut ret: *const Node = 0 as *const Node;
        let mut node_matches = false;

        let root_sector_list_ptr = alloc(size_of::<SectorList>()) as *mut SectorList;
        if root_sector_list_ptr as usize > 0 {
            let root_sector_list = &mut *root_sector_list_ptr;
            let mut root_sector_list_address = self.header.root_sector_list.address;
            while root_sector_list_address > 0 {
                self.disk.read(root_sector_list_address, 1, root_sector_list_ptr as usize);

                for extent_i in 0..30 {
                    let extent = root_sector_list.extents[extent_i];
                    if extent.block.address > 0 {
                        for node_address in extent.block.address..extent.block.address + extent.length {
                            let node = alloc(size_of::<Node>()) as *const Node;
                            self.disk.read(node_address, 1, node as usize);

                            node_matches = true;
                            let mut i = 0;
                            for c in filename.chars()  {
                                if !(i < 256 && (*node).name[i] == c as u8) {
                                    node_matches = false;
                                    break;
                                }
                                i += 1;
                            }
                            if !(i < 256 && (*node).name[i] == 0) {
                                node_matches = false;
                            }

                            if node_matches {
                                ret = node;
                                break;
                            }else{
                                unalloc(node as usize);
                            }
                        }
                    }

                    if node_matches {
                        break;
                    }
                }

                root_sector_list_address = root_sector_list.next_fragment.address;

                if node_matches{
                    break;
                }
            }
            unalloc(root_sector_list_ptr as usize);
        }
        ret
    }

    pub fn list(&self, directory: String) -> Vec<String> {
        let mut ret = Vec::<String>::new();

        unsafe{
            let root_sector_list_ptr = alloc(size_of::<SectorList>()) as *mut SectorList;
            if root_sector_list_ptr as usize > 0 {
                let root_sector_list = &mut *root_sector_list_ptr;
                let mut root_sector_list_address = self.header.root_sector_list.address;
                while root_sector_list_address > 0 {
                    self.disk.read(root_sector_list_address, 1, root_sector_list_ptr as usize);

                    for extent_i in 0..30 {
                        let extent = root_sector_list.extents[extent_i];
                        if extent.block.address > 0 {
                            for node_address in extent.block.address..extent.block.address + extent.length {
                                let node = alloc(size_of::<Node>()) as *const Node;
                                self.disk.read(node_address, 1, node as usize);

                                let node_name = String::from_c_slice(&(*node).name);
                                if node_name.starts_with(directory.clone()) {
                                    ret.push(node_name);
                                }

                                unalloc(node as usize);
                            }
                        }
                    }

                    root_sector_list_address = root_sector_list.next_fragment.address;
                }
                unalloc(root_sector_list_ptr as usize);
            }
        }

        ret
    }

    pub unsafe fn load(&self, filename: String) -> usize {
        let mut destination = 0;

        let node = self.node(filename);

        if node as usize > 0{
            if (*node).data_sector_list.address > 0 {
                let sector_list_ptr = alloc(size_of::<SectorList>()) as *mut SectorList;
                if sector_list_ptr as usize > 0 {
                    let sector_list = &mut *sector_list_ptr;
                    self.disk.read((*node).data_sector_list.address, 1, sector_list_ptr as usize);

                    //TODO: More than one extent, extent sector count > 64K
                    let mut size = 0;
                    for i in 0..1 {
                        if sector_list.extents[i].block.address > 0 && sector_list.extents[i].length > 0{
                            size += sector_list.extents[i].length * 512;
                        }
                    }

                    destination = alloc(size as usize);
                    if destination > 0 {
                        for i in 0..1 {
                            if sector_list.extents[i].block.address > 0 && sector_list.extents[i].length > 0{
                                self.disk.read(sector_list.extents[i].block.address, sector_list.extents[i].length as u16, destination);
                            }
                        }
                    }
                    unalloc(sector_list_ptr as usize);
                }
            }

            unalloc(node as usize);
        }

        return destination;
    }

    // TODO: Support realloc of LBAs
    pub unsafe fn save(&self, filename: String, source: usize){
        let node = self.node(filename);

        if node as usize > 0{
            if (*node).data_sector_list.address > 0 {
                let sector_list_ptr = alloc(size_of::<SectorList>()) as *mut SectorList;
                if sector_list_ptr as usize > 0 {
                    let sector_list = &mut *sector_list_ptr;
                    self.disk.read((*node).data_sector_list.address, 1, sector_list_ptr as usize);

                    if source > 0 {
                        for i in 0..1 {
                            if sector_list.extents[i].block.address > 0 && sector_list.extents[i].length > 0{
                                self.disk.write(sector_list.extents[i].block.address, sector_list.extents[i].length as u16, source);
                            }
                        }
                    }
                    unalloc(sector_list_ptr as usize);
                }
            }

            unalloc(node as usize);
        }
    }
}
