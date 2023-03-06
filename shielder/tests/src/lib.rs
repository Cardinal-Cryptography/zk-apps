#[allow(unused)]
mod shielder;

#[allow(unused)]
mod psp22;

#[allow(unused)]
#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{psp22::PSP22Token, shielder::Shielder};

    #[tokio::test]
    pub async fn dummy() -> Result<()> {
        println!("Dummy done");
        Ok(())
    }
}
