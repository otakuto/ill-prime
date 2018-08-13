
extern crate rand;
extern crate num_bigint as bigint;
extern crate tiny_keccak;
use std::mem;
use std::fmt;
use std::time::{Instant};
use std::time::{SystemTime, UNIX_EPOCH};

use bigint::{BigUint, ToBigUint, RandBigInt};


#[derive (Debug)]
struct Block
{
    timestamp: [u8; 8],
    prev_hash: [u8; 32],
    data: Vec<u8>,
    nonce: Vec<u8>,
}

impl Block
{
    fn hash(&self) -> [u8; 32]
    {
        let mut v: Vec<u8> = Vec::new();
        v.extend_from_slice(&self.timestamp);
        v.extend_from_slice(&self.prev_hash);
        v.extend_from_slice(&self.data.as_slice());
        v.extend_from_slice(&self.nonce.as_slice());

        return tiny_keccak::sha3_256(v.as_slice());
    }
}

fn hash_to_string(hash: & [u8; 32]) -> String
{
    let s = hash.into_iter().map(|x| format!("{:02X}", x)).collect::<Vec<String>>().join("");
    return s;
}

impl fmt::Display for Block
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let ts = unsafe { mem::transmute::<[u8; 8], u64>(self.timestamp) };
        let prev_hash = hash_to_string(&self.prev_hash);
        let hash = hash_to_string(&self.hash());
        write!(f, "timestamp={}\nprev_hash={}\nhash     ={}\ndata     ={:?}\nnonce    ={:?}", ts, prev_hash, hash, &self.data, &self.nonce)
    }
}

fn is_prime(n :BigUint) -> Result<bool, String>
{
    let zero = 0.to_biguint().unwrap();
    let one = 1.to_biguint().unwrap();
    let two = 2.to_biguint().unwrap();

    if n == two
    {
        return Ok(true);
    }
    if n == one || (&n & &one) == zero
    {
        return Ok(false);
    }

    let mut d = (&n - &one) >> 1;
    while (&d & &one) == zero
    {
        d >>= 1;
    }

    let mut rng = rand::thread_rng();

    for _ in 0..100
    {
        //println!("{}", i);
        let a = rng.gen_biguint_range(&one, &(&n - &one));
        let mut t = d.clone();
        let mut y = a.modpow(&t, &n);

        while (&t != &(&n - &one)) && (&y != &one) && (&y != &(&n - &one))
        {
            y = y.modpow(&two, &n);
            t <<= 1;
        }

        if (&y != &(&n - &one)) && ((&t & &one) == zero)
        {
            return Ok(false)
        }
    }

    return Ok(true);
}

fn mine(timestamp: & [u8; 8], prev_hash: & [u8; 32], data: & Vec<u8>) -> Vec<u8>
{
    let n;
    {
        let mut v: Vec<u8> = Vec::new();
        v.extend_from_slice(timestamp);
        v.extend_from_slice(prev_hash);
        v.extend_from_slice(data);

        n = BigUint::from_bytes_be(v.as_slice());
    }

    let mut i :usize = 1;
    loop
    {
        let m = &n << (&i * 8);
        let nonce_max = (1 << (i * 8)).to_biguint().unwrap();
        let mut nonce = 0.to_biguint().unwrap();
        while nonce < nonce_max
        {
            //println!("{}/{}", nonce, nonce_max);
            //if nonce > 0xFFF.to_biguint().unwrap() && is_prime(&m + &nonce).unwrap()
            if is_prime(&m + &nonce).unwrap()
            {
                println!("{} is prime num", &m + &nonce);
                let nonce = &nonce.to_bytes_be();
                let mut v = vec![0u8; i - nonce.len()];
                v.extend_from_slice(nonce);
                return v;
            }
            nonce += 1.to_biguint().unwrap();
        }
        i += 1;
    }
}

fn main()
{
    let mut s = std::iter::repeat("a").take(200).collect::<String>();
    let mut chain: Vec<Block> = Vec::new();
    chain.push(Block{ timestamp: [0u8; 8], prev_hash: [0u8; 32], data: vec![1,23], nonce: vec![]});
    let mut prev_hash = chain.last().unwrap().hash();

    loop
    {
        let now = Instant::now();
        let timestamp = unsafe { mem::transmute::<u64, [u8; 8]>(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()) };
        s.push('a');
        let i = BigUint::from_bytes_be(s.as_bytes());
        //let i = BigUint::parse_bytes(b"1145141919810", 16).unwrap();
        //let i = 222222222222222222.to_biguint().unwrap();
        println!("{}", i);
        println!("length={}", s.len());
        let nonce = mine(&timestamp, &prev_hash, &s.as_bytes().to_vec());
        println!("nonce={:?}", &nonce);
        {
            let mut v: Vec<u8> = Vec::new();
            v.extend_from_slice(&timestamp);
            v.extend_from_slice(&prev_hash);
            v.extend_from_slice(&s.as_bytes().to_vec());
            v.extend_from_slice(&nonce.as_slice());
            println!("{:?}", is_prime(BigUint::from_bytes_be(v.as_slice())));
            println!("block.len()={}",&v.len());
        }
        println!("{} sec", now.elapsed().as_secs());
        let block = Block
        {
            timestamp: timestamp,
            prev_hash: prev_hash,
            data: i.to_bytes_be().clone(),
            nonce: nonce,
        };
        prev_hash = block.hash();
        chain.push(block);
        println!("== Block ==\n{}\n", &chain.last().unwrap());
    }
}

