use borsh::BorshDeserialize;
use solana_program::{entrypoint::ProgramResult, msg};

use crate::{entrypoint::AllocatorHandler, processor::{HeapCommand, HeapKit, MAX_HEAP_DEV_COUNT}};


pub fn heap_kit(instruction_data: &[u8]) -> ProgramResult {
    let HeapKit { commands } = HeapKit::try_from_slice(instruction_data)?;

    if commands.len() > MAX_HEAP_DEV_COUNT {
        msg!("Too many commands, skip");
        return Ok(());
    }

    for command in commands {
        match command {
            HeapCommand::Allocate { size } => {
                let vec_ptr: Vec<u8> = Vec::with_capacity(size as usize);
                msg!("Allocated: {:p}", vec_ptr.as_ptr());
            }
            HeapCommand::Edit { index, data, resize } => {
                unsafe {
                    let (_, ptr) =  AllocatorHandler::search(index as usize);
                    AllocatorHandler::edit(ptr, &data, resize);
                }
            }
            HeapCommand::Search { index } => {
                unsafe {
                    let (len, ptr) = AllocatorHandler::search(index as usize);
                    msg!("Index: {}, Length: {}, Ptr: {:p}", index, len, ptr);
                }
            }
        }
    }

    Ok(())
}