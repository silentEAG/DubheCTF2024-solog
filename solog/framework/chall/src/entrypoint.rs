use {
    crate::{instruction, processor::SologInstruction},
    borsh::BorshDeserialize,
    solana_program::{
        account_info::AccountInfo, entrypoint::{ProgramResult, HEAP_LENGTH, HEAP_START_ADDRESS}, msg, pubkey::Pubkey
    },
    std::{alloc::Layout, mem::size_of, ptr::null_mut, usize},
};

pub struct AllocatorHandler;

impl AllocatorHandler {
    const BOTTOM_ADDRESS: usize = HEAP_START_ADDRESS as usize + size_of::<*mut u8>();

    pub unsafe fn edit(pos: *mut u8, data: &[u8], resize: bool) {

        let idx_length = *(pos.sub(8) as *const usize);
        let mut idx_data_ptr = pos;
        let idx_next_ptr = pos.add(idx_length) as *mut usize;
        let data_len = data.len();

        if idx_length > HEAP_LENGTH {
            msg!("Data length is too long");
            return;
        }

        if idx_length == 0 {
            msg!("Data length is 0");
            return;
        }

        if data_len > idx_length + 8 || data_len > 40 {
            msg!("Data length is too long");
            return;
        }

        // write data to idx_data_ptr
        for i in 0..data_len {
            idx_data_ptr.write_volatile(data[i]);
            idx_data_ptr = idx_data_ptr.add(1);
        }

        if data_len < idx_length && resize {
            // update next pointer
            *(pos.add(data_len) as *mut usize) = *idx_next_ptr;
            // update data length
            *(pos.sub(8) as *mut usize) = data_len;
        }
    }

    pub unsafe fn search(idx: usize) -> (usize, *mut u8) {

        let mut current_idx = 1;

        let mut pos = *(Self::BOTTOM_ADDRESS as *mut usize) as *mut u8;
        let mut idx_length = *(pos as *mut usize);
        let mut idx_ptr = pos.add(8);

        while current_idx < idx {

            if pos.is_null() {
                return (0, null_mut());
            }

            let offset_ptr = pos.add(idx_length as usize + 8);

            pos = *(offset_ptr as *const usize) as *mut u8;

            idx_length = *(pos as *mut usize);
            idx_ptr = pos.add(8);

            current_idx += 1;
        };

        (idx_length, idx_ptr)
        
    }

}

struct SologAllocator;

impl SologAllocator {
    const BOTTOM_ADDRESS: usize = HEAP_START_ADDRESS as usize + size_of::<*mut u8>();
    const TOP_ADDRESS: usize = HEAP_START_ADDRESS as usize + HEAP_LENGTH;
    const POS_PTR: *mut usize = HEAP_START_ADDRESS as usize as *mut usize;
}

unsafe impl std::alloc::GlobalAlloc for SologAllocator {

    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {

        let mut pos = *Self::POS_PTR;
        if pos == 0 {
            pos = Self::BOTTOM_ADDRESS;
        }

        // 8 bytes current pointer
        *(pos as *mut usize) = pos.saturating_add(8);
        pos = pos.saturating_add(8);

        // 8 bytes data length
        let layout_size = layout.size();
        *(pos as *mut usize) = layout_size;
        pos = pos.saturating_add(8);

        // x bytes data
        let return_pos = pos as *mut u8;
        pos = pos.saturating_add(layout_size as usize);


        let next_ptr_pos = pos;

        if next_ptr_pos > Self::TOP_ADDRESS {
            return null_mut();
        }

        *Self::POS_PTR = pos;
        return_pos
    }

    #[inline]
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        // no deallocation :>
    }
}

#[cfg(not(feature = "no-entrypoint"))]
#[cfg(target_os = "solana")]
#[global_allocator]
static A: SologAllocator = SologAllocator;

solana_program::entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match SologInstruction::try_from_slice(instruction_data)? {
        SologInstruction::CreatePost { title, content } => {
            instruction::create_post(program_id, accounts, title, content)?
        }
        SologInstruction::Clap {data} => {
            instruction::clap(program_id, accounts, &data)?
        },
        SologInstruction::AddComment { content } => {
            instruction::add_comment(program_id, accounts, content)?
        },
        SologInstruction::AddCollaborator => {
            instruction::add_collaborator(program_id, accounts)?
        },
        SologInstruction::EditComment { content } => {
            instruction::edit_comment(program_id, accounts, content)?
        }
    };
    Ok(())
}
