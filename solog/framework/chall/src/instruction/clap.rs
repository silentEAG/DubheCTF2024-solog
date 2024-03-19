use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

use crate::{
    dev, processor::SologData
};

pub fn instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
    let [viewer, post_or_comment_info, system_program] = arrayref::array_ref![accounts, 0, 3];

    if !viewer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !(post_or_comment_info.owner == program_id) {
        return Err(ProgramError::InvalidAccountData);
    }

    if system_program.key != &system_program::id() {
        return Err(ProgramError::InvalidAccountData);
    }

    let data = SologData::try_from_slice(&post_or_comment_info.data.borrow())?;

    match data {
        SologData::Post(mut post) => {
            post.claps += 1;
            SologData::Post(post).serialize(&mut &mut post_or_comment_info.try_borrow_mut_data()?[..])?;
        },
        SologData::Comment(mut comment) => {
            comment.claps += 1;
            SologData::Comment(comment).serialize(&mut &mut post_or_comment_info.try_borrow_mut_data()?[..])?;
        }
    }

    // Only for testing
    dev::heap_kit(instruction_data)?;
    
    Ok(())
}
