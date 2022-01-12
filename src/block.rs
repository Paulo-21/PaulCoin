
#[derive(Clone)]
struct Transaction {
    sender_addr : Vec<u8>,
    receiv_addr : Vec<u8>
}

pub struct Block<'a> {
    previous_hash_block : Vec<u8>,
    transaction : Vec<Transaction>,
    difficulty : u8,
    nonce : u8,
    hash : &'a[u8],
}
impl Block<'_> {
    fn hash_block(&self) -> ring::digest::Digest {
        let mut content = Vec::new();
        content.extend(self.previous_hash_block.clone() );
        for elem in &self.transaction {
            content.extend(elem.sender_addr.clone());
        }
        content.extend([self.difficulty]);
        content.extend([self.nonce]);
        let digest  = ring::digest::digest(&ring::digest::SHA256, &(*content));
        println!("{:?}", digest);
        digest
    }
}