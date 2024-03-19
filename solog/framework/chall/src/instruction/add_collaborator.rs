use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey
};

use crate::processor::{SologData, MAX_COLLABORATORS};

pub fn instruction(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [author, collaborator, post_info] = arrayref::array_ref![accounts, 0, 3];

    if !author.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !post_info.is_writable || post_info.owner != program_id {
        return Err(ProgramError::InvalidAccountData);
    }

    let post = SologData::try_from_slice(&post_info.try_borrow_data()?)?;
    let mut post = match post {
        SologData::Post(post) => post,
        _ => return Err(ProgramError::InvalidInstructionData),
    };

    if post.author != *author.key {
        msg!("Author mismatch");
        return Err(ProgramError::InvalidAccountData);
    }

    if post.collaborator_count >= MAX_COLLABORATORS as u8 {
        msg!("Too many collaborators");
        return Err(ProgramError::InvalidInstructionData);
    }

    if post.collaborators.contains(&*collaborator.key) {
        msg!("Collaborator already added");
        return Err(ProgramError::InvalidInstructionData);
    }

    post.collaborators[post.collaborator_count as usize] = *collaborator.key;
    post.collaborator_count += 1;

    SologData::Post(post).serialize(&mut &mut post_info.try_borrow_mut_data()?[..])?;
    Ok(())
}
