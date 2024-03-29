use num_bigint::{BigInt, BigUint, RandBigInt};
use num_traits::{One, identities::Zero};
use rand::Rng;
use std::convert::TryInto;


/// Trait d'extension pour les grands entiers non signés. Permet notamment leur découpage et l'obtention de leur taille digitale.
pub trait NumUtil
{
    /// Permet d'obtenir le nombre de chiffres du grand entier dans la base `radix`.
    fn sz(&self, radix: u32) -> u32;
    /// Permet d'obtenir le nombre d'octets utilisé par le grand entier.
    fn sz_b(&self) -> u32
    {
        (self.sz(16) + 1) / 2
    }

    /// Remplit un vecteur de grands entiers en découpant le grand entier sur lequel est appliqué la méthode, chaque bloc de taille maximale `block_sz` octets.
    fn expl_f(&self, buf: &mut Vec<BigUint>, block_sz: u32);
    /// Découpe l'entier en un vecteur de grands entiers et le retourne, chaque bloc de taille maximale `block_sz` octets.
    fn expl_r(&self, block_sz: u32) -> Vec<BigUint>
    {
        let mut buf: Vec<BigUint> = Vec::new();
        self.expl_f(&mut buf, block_sz);

        buf
    }
}

impl NumUtil for BigUint
{
    fn sz(&self, radix: u32) -> u32
    {
        self.to_str_radix(radix).len().try_into().unwrap()
    }

    fn expl_f(&self, buf: &mut Vec<BigUint>, block_sz: u32)
    {
        let m = BigUint::from(2u8).pow(block_sz * 8);
        let mut op = self.clone();

        while !op.is_zero()
        {
            buf.push(&op % &m);
            op /= &m;
        }

        buf.reverse();
    }
}


/// Trait d'extension pour les vecteurs de grands entiers. Permet notamment la recomposition de grands nombres.
pub trait VecNumUtil
{
    /// Recompose un grand nombre depuis ses parties (préalablement découpée avec `expl_f` ou `expl_r`)
    fn rejoin(&self) -> BigUint;
}

impl VecNumUtil for Vec<BigUint>
{
    fn rejoin(&self) -> BigUint
    {
        if self.is_empty()
        {
            panic!("VecNumUtil.join (BigUint) : vecteur vide");
        }

        let mut b = BigUint::from(0u8);
        let mut mult;
        let base = BigUint::from(2u8);

        for part in self
        {
            mult = base.pow(part.sz_b() * 8);
            b = &b * &mult + part;
        }

        b
    }
}

impl VecNumUtil for Vec<u8>
{
    fn rejoin(&self) -> BigUint
    {
        if self.is_empty()
        {
            panic!("VecNumUtil.join (u8) : vecteur vide");
        }

        let mut b = BigUint::from(0u8);
        let mult = BigUint::from(2u8).pow(8);

        for part in self
        {
            b = &b * &mult + part;
        }

        b
    }
}


const EXPCODE_TAB: [u8; 35] = [ 2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71,73,79,83,89,97,101,103,107,109,113,127,131,137,139,149 ];
const PRIME_RN: u32 = 12737213u32;
/// Nombre d'itérations du test de primalité probabiliste à effectuer.
const PRIME_ROUNDS: u8 = 20;

/// Fonction d'exponentiation rapide, très utile pour le RSA.
pub fn fmodpow(base: &BigUint, exp: &BigUint, num: &BigUint) -> BigUint
{
    let mut res = BigUint::from(1u8);
    let mut exp_bin = exp.clone();
    let mut temp = base.clone();
    let mut r;

    while !exp_bin.is_zero()
    {
        r = &exp_bin % 2u8;
        if r.is_one()
        {
            res = (&res * &temp) % num;
        }

        exp_bin /= 2u8;
        temp = (&temp * &temp) % num;
    }

    res
}

/// Algorithme d'Euclide pour trouver le PGCD de deux nombres. Utile pour le RSA.
pub fn euclide(a: &BigInt, b: &BigInt) -> BigInt
{
    let (mut r1, mut r2) = (a.clone(), b.clone());
    let (mut u1, mut u2) = (BigInt::from(1u8), BigInt::from(0u8));
    let (mut v1, mut v2) = (BigInt::from(1u8), BigInt::from(0u8));
    let (mut u3, mut v3, mut r3);
    let mut q;

    while !r2.is_zero()
    {
        q = &r1 / &r2;
        r3 = r1;
        u3 = u1;
        v3 = v1;
        r1 = r2;
        u1 = u2;
        v1 = v2;
        r2 = &r3 - &q * &r1;
        u2 = &u3 - &q * &u1;
        v2 = &v3 - &q * &v1;
    }

    u1
}

/// Retourne le code d'exposant d'un nombre.
pub fn expcode(num: &BigUint) -> Option<BigUint>
{
    for &i in EXPCODE_TAB.iter()
    {
        if !(num % i).is_zero()
        {
            return Some(BigUint::from(i));
        }
    }

    None
}

/// Retourne vrai si le grand entier `num` est premier, faux sinon.
/// Le test est probabiliste et peut se tromper ; avec un nombre assez grand d'itérations `PRIME_ROUNDS`, cela est toutefois peu probable.
pub fn isprime(num: &BigUint) -> bool
{
    // Le test étant probabiliste, il faut faire plusieurs itérations pour être raisonnablement certain du résultat
    for _ in 0..PRIME_ROUNDS
    {
        if !fmodpow(&(&PRIME_RN % num), &(num - 1u8), num).is_one()
        {
            return false;
        }
    }

    true
}

/// Retourne un grand entier constitué de `szb` octets avec une bonne probabilité qu'il soit premier.
pub fn rand_primelike(szb: u64) -> BigUint
{
    let mut b = rand::thread_rng().gen_biguint(szb * 8);
    // On met le dernier chiffre à zéro
    b /= 10u8;
    b *= 10u8;

    // On génère un chiffre impair qui n'est pas 5 afin d'augmenter les chances que le nombre soit premier
    let mut digit = 0u8;
    while digit % 2 == 0 || digit == 5
    {
        digit = rand::thread_rng().gen_range(1..10);
    }
    b += digit;

    b
}