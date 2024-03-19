use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
};

use crate::processor::SologData;


pub fn instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    content: Vec<u8>,
) -> ProgramResult {
    let [author, comment_info, system_program] = arrayref::array_ref![accounts, 0, 3];

    if !author.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !(comment_info.owner == program_id) {
        return Err(ProgramError::InvalidAccountData);
    }

    let comment_data = SologData::try_from_slice(&comment_info.data.borrow())?;

    let mut comment_data = match comment_data {
        SologData::Comment(comment) => comment,
        _ => return Err(ProgramError::InvalidInstructionData),
    };

    if comment_data.author != *author.key {
        return Err(ProgramError::InvalidAccountData);
    }

    comment_data.content = content;
    let comment_data = SologData::Comment(comment_data);

    let new_comment_data_len = to_vec(&comment_data)?.len();
    let new_rent = Rent::default().minimum_balance(new_comment_data_len);
    let diff_rent = new_rent.saturating_sub(comment_info.lamports());


    invoke(
        &system_instruction::transfer(author.key, comment_info.key, diff_rent),
        &[author.clone(), comment_info.clone(), system_program.clone()],
    )?;

    comment_info.realloc(new_comment_data_len, false)?;
    comment_data.serialize(&mut &mut comment_info.try_borrow_mut_data()?[..])?;

    msg!("edit comment success");
    Ok(())
}
