use std::str::FromStr;

use cosmrs::{self, bank::MsgSend, AccountId, Denom, tx::Msg, ErrorReport};

use super::{Module, TxMetadata, UnsignedTx};

#[derive(Clone, Debug)]
pub enum Bank<'m> {
    Send {
        from: &'m str,
        to: &'m str,
        amount: u64,
        denom: &'m str,
    },
}

impl Module<'_> for Bank<'_> {
    // fix
    type Error = ErrorReport;

    fn try_into_tx(self) -> Result<UnsignedTx, Self::Error> {
        match self {
            Bank::Send {
                from,
                to,
                amount,
                denom,
            } => {
                let amount = cosmrs::Coin {
                    amount: amount.into(),
                    denom: Denom::from_str(denom)?,
                };
                let msg = MsgSend {
                    from_address: AccountId::from_str(from)?,
                    to_address: AccountId::from_str(to)?,
                    amount: vec![amount.clone()],
                };

                Ok(UnsignedTx {
                    messages: vec![msg.to_any()?],
                    metadata: TxMetadata::default(),
                })
            }
        }
    }
}
